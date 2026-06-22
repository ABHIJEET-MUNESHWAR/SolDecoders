//! SPL Token program decoder.
//!
//! Decodes the most analytics-relevant instructions: `Transfer` (3),
//! `MintTo` (7), `Burn` (8) and their `*Checked` variants (12/14/15) which carry
//! the mint and declared decimals.

use soldecoders_types::{DecodeError, DecodedEvent, Pubkey, RawInstruction, TokenAmount};

use crate::decoder::InstructionDecoder;
use crate::programs::{account, read_u64};

/// The canonical SPL Token program id.
const SPL_TOKEN_PROGRAM: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

const TAG_TRANSFER: u8 = 3;
const TAG_MINT_TO: u8 = 7;
const TAG_BURN: u8 = 8;
const TAG_TRANSFER_CHECKED: u8 = 12;
const TAG_MINT_TO_CHECKED: u8 = 14;
const TAG_BURN_CHECKED: u8 = 15;

/// Decoder for the SPL Token program.
pub struct SplTokenDecoder {
    program_id: Pubkey,
}

impl Default for SplTokenDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl SplTokenDecoder {
    /// Construct the decoder with the canonical program id.
    pub fn new() -> Self {
        Self {
            // SAFETY-of-invariant: a hard-coded, well-known program id; parsed
            // once at construction (init time), never on a request path.
            program_id: Pubkey::from_base58(SPL_TOKEN_PROGRAM).expect("valid SPL Token program id"),
        }
    }

    fn tag(ix: &RawInstruction) -> Result<u8, DecodeError> {
        ix.data
            .first()
            .copied()
            .ok_or(DecodeError::TooShort { have: 0, need: 1 })
    }
}

impl InstructionDecoder for SplTokenDecoder {
    fn program_id(&self) -> &Pubkey {
        &self.program_id
    }

    fn name(&self) -> &'static str {
        "spl_token"
    }

    fn decode(&self, ix: &RawInstruction) -> Result<DecodedEvent, DecodeError> {
        let tag = Self::tag(ix)?;
        match tag {
            TAG_TRANSFER => Ok(DecodedEvent::SplTransfer {
                source: account(ix, 0, 3)?,
                destination: account(ix, 1, 3)?,
                authority: account(ix, 2, 3)?,
                mint: None,
                amount: TokenAmount::raw(read_u64(ix, 1)?),
            }),
            TAG_TRANSFER_CHECKED => {
                let decimals = *ix.data.get(9).ok_or(DecodeError::TooShort {
                    have: ix.data.len(),
                    need: 10,
                })?;
                Ok(DecodedEvent::SplTransfer {
                    source: account(ix, 0, 4)?,
                    mint: Some(account(ix, 1, 4)?),
                    destination: account(ix, 2, 4)?,
                    authority: account(ix, 3, 4)?,
                    amount: TokenAmount::checked(read_u64(ix, 1)?, decimals),
                })
            }
            TAG_MINT_TO => Ok(DecodedEvent::SplMint {
                mint: account(ix, 0, 3)?,
                destination: account(ix, 1, 3)?,
                authority: account(ix, 2, 3)?,
                amount: TokenAmount::raw(read_u64(ix, 1)?),
            }),
            TAG_MINT_TO_CHECKED => {
                let decimals = *ix.data.get(9).ok_or(DecodeError::TooShort {
                    have: ix.data.len(),
                    need: 10,
                })?;
                Ok(DecodedEvent::SplMint {
                    mint: account(ix, 0, 3)?,
                    destination: account(ix, 1, 3)?,
                    authority: account(ix, 2, 3)?,
                    amount: TokenAmount::checked(read_u64(ix, 1)?, decimals),
                })
            }
            TAG_BURN => Ok(DecodedEvent::SplBurn {
                source: account(ix, 0, 3)?,
                mint: account(ix, 1, 3)?,
                authority: account(ix, 2, 3)?,
                amount: TokenAmount::raw(read_u64(ix, 1)?),
            }),
            TAG_BURN_CHECKED => {
                let decimals = *ix.data.get(9).ok_or(DecodeError::TooShort {
                    have: ix.data.len(),
                    need: 10,
                })?;
                Ok(DecodedEvent::SplBurn {
                    source: account(ix, 0, 3)?,
                    mint: account(ix, 1, 3)?,
                    authority: account(ix, 2, 3)?,
                    amount: TokenAmount::checked(read_u64(ix, 1)?, decimals),
                })
            }
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

    fn transfer_ix() -> RawInstruction {
        let mut data = vec![TAG_TRANSFER];
        data.extend_from_slice(&500u64.to_le_bytes());
        RawInstruction::new(
            SplTokenDecoder::new().program_id().clone(),
            vec![key(1), key(2), key(3)],
            data,
        )
    }

    #[test]
    fn decodes_transfer() {
        let d = SplTokenDecoder::new();
        let event = d.decode(&transfer_ix()).unwrap();
        match event {
            DecodedEvent::SplTransfer {
                source,
                destination,
                authority,
                mint,
                amount,
            } => {
                assert_eq!(source, key(1));
                assert_eq!(destination, key(2));
                assert_eq!(authority, key(3));
                assert_eq!(mint, None);
                assert_eq!(amount, TokenAmount::raw(500));
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn decodes_transfer_checked_with_decimals() {
        let mut data = vec![TAG_TRANSFER_CHECKED];
        data.extend_from_slice(&1_500_000u64.to_le_bytes());
        data.push(6); // decimals
        let ix = RawInstruction::new(
            SplTokenDecoder::new().program_id().clone(),
            vec![key(1), key(9), key(2), key(3)],
            data,
        );
        let event = SplTokenDecoder::new().decode(&ix).unwrap();
        match event {
            DecodedEvent::SplTransfer { mint, amount, .. } => {
                assert_eq!(mint, Some(key(9)));
                assert_eq!(amount, TokenAmount::checked(1_500_000, 6));
                assert_eq!(amount.ui_amount(), Some(1.5));
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn decodes_mint_and_burn() {
        let d = SplTokenDecoder::new();
        let mut mint_data = vec![TAG_MINT_TO];
        mint_data.extend_from_slice(&10u64.to_le_bytes());
        let mint_ix = RawInstruction::new(
            d.program_id().clone(),
            vec![key(1), key(2), key(3)],
            mint_data,
        );
        assert!(matches!(
            d.decode(&mint_ix).unwrap(),
            DecodedEvent::SplMint { .. }
        ));

        let mut burn_data = vec![TAG_BURN];
        burn_data.extend_from_slice(&7u64.to_le_bytes());
        let burn_ix = RawInstruction::new(
            d.program_id().clone(),
            vec![key(1), key(2), key(3)],
            burn_data,
        );
        assert!(matches!(
            d.decode(&burn_ix).unwrap(),
            DecodedEvent::SplBurn { .. }
        ));
    }

    #[test]
    fn rejects_unknown_discriminator() {
        let d = SplTokenDecoder::new();
        let ix = RawInstruction::new(d.program_id().clone(), vec![], vec![99]);
        assert!(matches!(
            d.decode(&ix).unwrap_err(),
            DecodeError::UnknownDiscriminator { .. }
        ));
    }

    #[test]
    fn rejects_missing_accounts() {
        let d = SplTokenDecoder::new();
        let mut data = vec![TAG_TRANSFER];
        data.extend_from_slice(&1u64.to_le_bytes());
        let ix = RawInstruction::new(d.program_id().clone(), vec![key(1)], data);
        assert!(matches!(
            d.decode(&ix).unwrap_err(),
            DecodeError::MissingAccounts { .. }
        ));
    }

    #[test]
    fn rejects_short_data() {
        let d = SplTokenDecoder::new();
        let ix = RawInstruction::new(
            d.program_id().clone(),
            vec![key(1), key(2), key(3)],
            vec![TAG_TRANSFER, 0, 0],
        );
        assert!(matches!(
            d.decode(&ix).unwrap_err(),
            DecodeError::TooShort { .. }
        ));
    }

    #[test]
    fn rejects_empty_data() {
        let d = SplTokenDecoder::new();
        let ix = RawInstruction::new(d.program_id().clone(), vec![], vec![]);
        assert!(matches!(
            d.decode(&ix).unwrap_err(),
            DecodeError::TooShort { need: 1, .. }
        ));
    }
}
