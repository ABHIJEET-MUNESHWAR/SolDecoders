//! Orca Whirlpool decoder — Anchor `swap` instruction.
//!
//! Whirlpool is an Anchor program, so instructions are prefixed with an 8-byte
//! discriminator: the first 8 bytes of `sha256("global:<ix_name>")`. The `swap`
//! payload is `amount: u64`, `other_amount_threshold: u64`, then a `u128` price
//! limit and two `bool` flags. We surface `amount` and the threshold as the
//! swap's in/out floor. The token authority is account index 1.

use sha2::{Digest, Sha256};

use soldecoders_types::{DecodeError, DecodedEvent, Pubkey, RawInstruction};

use crate::decoder::InstructionDecoder;
use crate::programs::{account, read_u64};

/// The Orca Whirlpool program id.
const ORCA_WHIRLPOOL: &str = "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc";

/// Token authority account index in the Whirlpool `swap` layout.
const AUTHORITY_INDEX: usize = 1;

/// Compute the 8-byte Anchor discriminator for a global instruction name.
fn anchor_discriminator(ix_name: &str) -> [u8; 8] {
    let mut hasher = Sha256::new();
    hasher.update(format!("global:{ix_name}").as_bytes());
    let digest = hasher.finalize();
    let mut out = [0u8; 8];
    out.copy_from_slice(&digest[..8]);
    out
}

/// Decoder for the Orca Whirlpool program.
pub struct OrcaWhirlpoolDecoder {
    program_id: Pubkey,
    swap_discriminator: [u8; 8],
}

impl Default for OrcaWhirlpoolDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl OrcaWhirlpoolDecoder {
    /// Construct the decoder, precomputing the `swap` discriminator.
    pub fn new() -> Self {
        Self {
            program_id: Pubkey::from_base58(ORCA_WHIRLPOOL).expect("valid Orca Whirlpool id"),
            swap_discriminator: anchor_discriminator("swap"),
        }
    }
}

impl InstructionDecoder for OrcaWhirlpoolDecoder {
    fn program_id(&self) -> &Pubkey {
        &self.program_id
    }

    fn name(&self) -> &'static str {
        "orca_whirlpool"
    }

    fn decode(&self, ix: &RawInstruction) -> Result<DecodedEvent, DecodeError> {
        let disc = ix.data.get(..8).ok_or(DecodeError::TooShort {
            have: ix.data.len(),
            need: 8,
        })?;
        if disc != self.swap_discriminator {
            return Err(DecodeError::UnknownDiscriminator {
                program: self.program_id.to_base58(),
                discriminator: format!("0x{}", hex8(disc)),
            });
        }
        Ok(DecodedEvent::DexSwap {
            program: self.program_id.clone(),
            user: account(ix, AUTHORITY_INDEX, AUTHORITY_INDEX + 1)?,
            amount_in: read_u64(ix, 8)?,
            min_amount_out: Some(read_u64(ix, 16)?),
            protocol: "orca_whirlpool".into(),
        })
    }
}

/// Render up to 8 bytes as hex for diagnostics.
fn hex8(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(byte: u8) -> Pubkey {
        Pubkey::new([byte; 32])
    }

    fn swap_ix() -> RawInstruction {
        let d = OrcaWhirlpoolDecoder::new();
        let mut data = d.swap_discriminator.to_vec();
        data.extend_from_slice(&2_000u64.to_le_bytes()); // amount
        data.extend_from_slice(&1_800u64.to_le_bytes()); // threshold
        RawInstruction::new(d.program_id().clone(), vec![key(0), key(5)], data)
    }

    #[test]
    fn discriminator_is_deterministic() {
        assert_eq!(anchor_discriminator("swap"), anchor_discriminator("swap"));
        assert_ne!(anchor_discriminator("swap"), anchor_discriminator("burn"));
    }

    #[test]
    fn decodes_swap() {
        let event = OrcaWhirlpoolDecoder::new().decode(&swap_ix()).unwrap();
        match event {
            DecodedEvent::DexSwap {
                user,
                amount_in,
                min_amount_out,
                protocol,
                ..
            } => {
                assert_eq!(user, key(5));
                assert_eq!(amount_in, 2_000);
                assert_eq!(min_amount_out, Some(1_800));
                assert_eq!(protocol, "orca_whirlpool");
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn rejects_wrong_discriminator() {
        let d = OrcaWhirlpoolDecoder::new();
        let mut data = vec![0u8; 8];
        data.extend_from_slice(&[0u8; 16]);
        let ix = RawInstruction::new(d.program_id().clone(), vec![key(0), key(5)], data);
        assert!(matches!(
            d.decode(&ix).unwrap_err(),
            DecodeError::UnknownDiscriminator { .. }
        ));
    }

    #[test]
    fn rejects_short_data() {
        let d = OrcaWhirlpoolDecoder::new();
        let ix = RawInstruction::new(d.program_id().clone(), vec![key(0), key(5)], vec![1, 2, 3]);
        assert!(matches!(
            d.decode(&ix).unwrap_err(),
            DecodeError::TooShort { need: 8, .. }
        ));
    }
}
