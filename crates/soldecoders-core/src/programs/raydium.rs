//! Raydium AMM v4 decoder — `swapBaseIn` / `swapBaseOut`.
//!
//! Raydium's AMM v4 uses a single-byte discriminator. The two swap variants
//! carry `amount_in`/`min_amount_out` (base-in) or `max_amount_in`/`amount_out`
//! (base-out) as little-endian `u64`s. The user authority is the final account
//! in the layout.

use soldecoders_types::{DecodeError, DecodedEvent, Pubkey, RawInstruction};

use crate::decoder::InstructionDecoder;
use crate::programs::read_u64;

/// The Raydium AMM v4 program id.
const RAYDIUM_AMM_V4: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";

const TAG_SWAP_BASE_IN: u8 = 9;
const TAG_SWAP_BASE_OUT: u8 = 11;

/// Minimum accounts a Raydium swap references (program-specific layout).
const MIN_SWAP_ACCOUNTS: usize = 3;

/// Decoder for the Raydium AMM v4 program.
pub struct RaydiumAmmV4Decoder {
    program_id: Pubkey,
}

impl Default for RaydiumAmmV4Decoder {
    fn default() -> Self {
        Self::new()
    }
}

impl RaydiumAmmV4Decoder {
    /// Construct the decoder with the canonical program id.
    pub fn new() -> Self {
        Self {
            program_id: Pubkey::from_base58(RAYDIUM_AMM_V4).expect("valid Raydium AMM v4 id"),
        }
    }

    fn user(ix: &RawInstruction) -> Result<Pubkey, DecodeError> {
        ix.accounts
            .last()
            .cloned()
            .ok_or(DecodeError::MissingAccounts {
                expected: MIN_SWAP_ACCOUNTS,
                found: ix.accounts.len(),
            })
    }
}

impl InstructionDecoder for RaydiumAmmV4Decoder {
    fn program_id(&self) -> &Pubkey {
        &self.program_id
    }

    fn name(&self) -> &'static str {
        "raydium_amm_v4"
    }

    fn decode(&self, ix: &RawInstruction) -> Result<DecodedEvent, DecodeError> {
        let tag = ix
            .data
            .first()
            .copied()
            .ok_or(DecodeError::TooShort { have: 0, need: 1 })?;
        match tag {
            TAG_SWAP_BASE_IN => Ok(DecodedEvent::DexSwap {
                program: self.program_id.clone(),
                user: Self::user(ix)?,
                amount_in: read_u64(ix, 1)?,
                min_amount_out: Some(read_u64(ix, 9)?),
                protocol: "raydium_amm_v4".into(),
            }),
            TAG_SWAP_BASE_OUT => Ok(DecodedEvent::DexSwap {
                program: self.program_id.clone(),
                user: Self::user(ix)?,
                // base-out: first field is max_amount_in (the offered cap).
                amount_in: read_u64(ix, 1)?,
                min_amount_out: Some(read_u64(ix, 9)?),
                protocol: "raydium_amm_v4".into(),
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

    fn swap_ix(tag: u8) -> RawInstruction {
        let mut data = vec![tag];
        data.extend_from_slice(&5_000u64.to_le_bytes());
        data.extend_from_slice(&4_500u64.to_le_bytes());
        RawInstruction::new(
            RaydiumAmmV4Decoder::new().program_id().clone(),
            vec![key(1), key(2), key(7)],
            data,
        )
    }

    #[test]
    fn decodes_swap_base_in() {
        let event = RaydiumAmmV4Decoder::new()
            .decode(&swap_ix(TAG_SWAP_BASE_IN))
            .unwrap();
        match event {
            DecodedEvent::DexSwap {
                user,
                amount_in,
                min_amount_out,
                protocol,
                ..
            } => {
                assert_eq!(user, key(7));
                assert_eq!(amount_in, 5_000);
                assert_eq!(min_amount_out, Some(4_500));
                assert_eq!(protocol, "raydium_amm_v4");
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn decodes_swap_base_out() {
        let event = RaydiumAmmV4Decoder::new()
            .decode(&swap_ix(TAG_SWAP_BASE_OUT))
            .unwrap();
        assert!(matches!(event, DecodedEvent::DexSwap { .. }));
    }

    #[test]
    fn rejects_unknown_and_short() {
        let d = RaydiumAmmV4Decoder::new();
        let bad = RawInstruction::new(d.program_id().clone(), vec![key(1)], vec![100]);
        assert!(matches!(
            d.decode(&bad).unwrap_err(),
            DecodeError::UnknownDiscriminator { .. }
        ));
        let short = RawInstruction::new(d.program_id().clone(), vec![key(1)], vec![]);
        assert!(matches!(
            d.decode(&short).unwrap_err(),
            DecodeError::TooShort { .. }
        ));
    }

    #[test]
    fn rejects_no_accounts() {
        let d = RaydiumAmmV4Decoder::new();
        let mut data = vec![TAG_SWAP_BASE_IN];
        data.extend_from_slice(&1u64.to_le_bytes());
        data.extend_from_slice(&1u64.to_le_bytes());
        let ix = RawInstruction::new(d.program_id().clone(), vec![], data);
        assert!(matches!(
            d.decode(&ix).unwrap_err(),
            DecodeError::MissingAccounts { .. }
        ));
    }
}
