//! System program decoder — native SOL transfers.

use soldecoders_types::{DecodeError, DecodedEvent, Pubkey, RawInstruction};

use crate::decoder::InstructionDecoder;
use crate::programs::{account, read_u64};

/// The native System program id (the all-zero pubkey).
const SYSTEM_PROGRAM: &str = "11111111111111111111111111111111";

/// The `Transfer` instruction discriminator (a little-endian `u32`).
const TAG_TRANSFER: u32 = 2;

/// Decoder for the System program.
pub struct SystemDecoder {
    program_id: Pubkey,
}

impl Default for SystemDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemDecoder {
    /// Construct the decoder with the canonical program id.
    pub fn new() -> Self {
        Self {
            program_id: Pubkey::from_base58(SYSTEM_PROGRAM).expect("valid System program id"),
        }
    }
}

impl InstructionDecoder for SystemDecoder {
    fn program_id(&self) -> &Pubkey {
        &self.program_id
    }

    fn name(&self) -> &'static str {
        "system"
    }

    fn decode(&self, ix: &RawInstruction) -> Result<DecodedEvent, DecodeError> {
        let disc = ix.data.get(..4).ok_or(DecodeError::TooShort {
            have: ix.data.len(),
            need: 4,
        })?;
        let arr: [u8; 4] = disc.try_into().map_err(|_| DecodeError::MalformedField {
            field: "discriminator",
            reason: "expected 4 bytes".into(),
        })?;
        let tag = u32::from_le_bytes(arr);
        match tag {
            TAG_TRANSFER => Ok(DecodedEvent::SystemTransfer {
                from: account(ix, 0, 2)?,
                to: account(ix, 1, 2)?,
                lamports: read_u64(ix, 4)?,
            }),
            other => Err(DecodeError::UnknownDiscriminator {
                program: self.program_id.to_base58(),
                discriminator: other.to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(byte: u8) -> Pubkey {
        Pubkey::new([byte; 32])
    }

    fn transfer_ix(lamports: u64) -> RawInstruction {
        let mut data = TAG_TRANSFER.to_le_bytes().to_vec();
        data.extend_from_slice(&lamports.to_le_bytes());
        RawInstruction::new(
            SystemDecoder::new().program_id().clone(),
            vec![key(1), key(2)],
            data,
        )
    }

    #[test]
    fn program_id_is_all_zero() {
        assert_eq!(SystemDecoder::new().program_id().as_bytes(), &[0u8; 32]);
    }

    #[test]
    fn decodes_transfer() {
        let event = SystemDecoder::new()
            .decode(&transfer_ix(1_000_000_000))
            .unwrap();
        match event {
            DecodedEvent::SystemTransfer { from, to, lamports } => {
                assert_eq!(from, key(1));
                assert_eq!(to, key(2));
                assert_eq!(lamports, 1_000_000_000);
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn rejects_short_discriminator() {
        let d = SystemDecoder::new();
        let ix = RawInstruction::new(d.program_id().clone(), vec![key(1), key(2)], vec![2, 0]);
        assert!(matches!(
            d.decode(&ix).unwrap_err(),
            DecodeError::TooShort { need: 4, .. }
        ));
    }

    #[test]
    fn rejects_unknown_discriminator() {
        let d = SystemDecoder::new();
        let mut data = 7u32.to_le_bytes().to_vec();
        data.extend_from_slice(&1u64.to_le_bytes());
        let ix = RawInstruction::new(d.program_id().clone(), vec![key(1), key(2)], data);
        assert!(matches!(
            d.decode(&ix).unwrap_err(),
            DecodeError::UnknownDiscriminator { .. }
        ));
    }

    #[test]
    fn rejects_missing_lamports() {
        let d = SystemDecoder::new();
        let ix = RawInstruction::new(
            d.program_id().clone(),
            vec![key(1), key(2)],
            TAG_TRANSFER.to_le_bytes().to_vec(),
        );
        assert!(matches!(
            d.decode(&ix).unwrap_err(),
            DecodeError::TooShort { .. }
        ));
    }
}
