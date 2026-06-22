//! The decoder registry — routes instructions to the right program decoder.

use std::collections::HashMap;

use soldecoders_types::{DecodeError, DecodedEvent, Pubkey, RawInstruction, RawTransaction};

use crate::decoder::InstructionDecoder;
use crate::programs::{OrcaWhirlpoolDecoder, RaydiumAmmV4Decoder, SplTokenDecoder, SystemDecoder};

/// A routing table from program id to its [`InstructionDecoder`].
///
/// Build one with [`DecoderRegistry::builtin`] for the standard protocol set,
/// or compose a custom registry with [`DecoderRegistry::with_decoder`].
#[derive(Default)]
pub struct DecoderRegistry {
    decoders: HashMap<Pubkey, Box<dyn InstructionDecoder>>,
}

impl DecoderRegistry {
    /// An empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// A registry pre-loaded with all built-in decoders (SPL Token, System,
    /// Raydium AMM v4, Orca Whirlpool).
    pub fn builtin() -> Self {
        Self::new()
            .with_decoder(SplTokenDecoder::new())
            .with_decoder(SystemDecoder::new())
            .with_decoder(RaydiumAmmV4Decoder::new())
            .with_decoder(OrcaWhirlpoolDecoder::new())
    }

    /// Register a decoder, consuming and returning `self` for builder-style use.
    ///
    /// Generic over the concrete decoder type so callers keep their decoders on
    /// the stack until they are boxed here.
    pub fn with_decoder<D: InstructionDecoder + 'static>(mut self, decoder: D) -> Self {
        self.register(Box::new(decoder));
        self
    }

    /// Register a boxed decoder. A later registration for the same program id
    /// replaces the earlier one.
    pub fn register(&mut self, decoder: Box<dyn InstructionDecoder>) {
        self.decoders.insert(decoder.program_id().clone(), decoder);
    }

    /// Decode a single instruction.
    ///
    /// # Errors
    /// [`DecodeError::UnknownProgram`] if no decoder is registered, otherwise
    /// whatever the matched decoder returns.
    pub fn decode_instruction(&self, ix: &RawInstruction) -> Result<DecodedEvent, DecodeError> {
        match self.decoders.get(&ix.program_id) {
            Some(decoder) => decoder.decode(ix),
            None => Err(DecodeError::UnknownProgram {
                program: ix.program_id.to_base58(),
            }),
        }
    }

    /// Decode every instruction in a transaction, preserving order. Each result
    /// is independent so unsupported instructions do not abort the others.
    pub fn decode_transaction(
        &self,
        tx: &RawTransaction,
    ) -> Vec<Result<DecodedEvent, DecodeError>> {
        tx.instructions
            .iter()
            .map(|ix| self.decode_instruction(ix))
            .collect()
    }

    /// The registered programs as `(base58_program_id, protocol_name)` pairs,
    /// sorted by name for stable output.
    pub fn supported_programs(&self) -> Vec<(String, &'static str)> {
        let mut out: Vec<_> = self
            .decoders
            .values()
            .map(|d| (d.program_id().to_base58(), d.name()))
            .collect();
        out.sort_by_key(|(_, name)| *name);
        out
    }

    /// Number of registered decoders.
    pub fn len(&self) -> usize {
        self.decoders.len()
    }

    /// Whether any decoder is registered.
    pub fn is_empty(&self) -> bool {
        self.decoders.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::InstructionDecoder;

    fn key(byte: u8) -> Pubkey {
        Pubkey::new([byte; 32])
    }

    #[test]
    fn builtin_registers_four_programs() {
        let reg = DecoderRegistry::builtin();
        assert_eq!(reg.len(), 4);
        assert!(!reg.is_empty());
        let names: Vec<_> = reg
            .supported_programs()
            .into_iter()
            .map(|(_, n)| n)
            .collect();
        assert_eq!(
            names,
            vec!["orca_whirlpool", "raydium_amm_v4", "spl_token", "system"]
        );
    }

    #[test]
    fn unknown_program_routes_to_error() {
        let reg = DecoderRegistry::builtin();
        let ix = RawInstruction::new(key(200), vec![], vec![0]);
        let err = reg.decode_instruction(&ix).unwrap_err();
        assert!(matches!(err, DecodeError::UnknownProgram { .. }));
        assert!(err.is_unsupported());
    }

    #[test]
    fn empty_registry_has_no_decoders() {
        let reg = DecoderRegistry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
    }

    #[test]
    fn decode_transaction_preserves_order_and_independence() {
        let reg = DecoderRegistry::builtin();
        let system = SystemDecoder::new().program_id().clone();
        // one decodable system transfer + one unknown program
        let mut data = vec![2, 0, 0, 0];
        data.extend_from_slice(&1_000u64.to_le_bytes());
        let ok = RawInstruction::new(system, vec![key(1), key(2)], data);
        let bad = RawInstruction::new(key(250), vec![], vec![]);
        let tx = RawTransaction::new(vec![], vec![ok, bad]);
        let results = reg.decode_transaction(&tx);
        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_err());
    }
}
