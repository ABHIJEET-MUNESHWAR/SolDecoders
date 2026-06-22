//! Built-in program decoders.

mod orca;
mod raydium;
mod spl_token;
mod system;

pub use orca::OrcaWhirlpoolDecoder;
pub use raydium::RaydiumAmmV4Decoder;
pub use spl_token::SplTokenDecoder;
pub use system::SystemDecoder;

use soldecoders_types::{DecodeError, RawInstruction};

/// Read a little-endian `u64` from instruction data at `offset`, mapping a
/// short buffer to a [`DecodeError::TooShort`].
pub(crate) fn read_u64(ix: &RawInstruction, offset: usize) -> Result<u64, DecodeError> {
    ix.read_u64_le(offset).ok_or(DecodeError::TooShort {
        have: ix.data.len(),
        need: offset + 8,
    })
}

/// Fetch the account at `index`, mapping absence to [`DecodeError::MissingAccounts`].
pub(crate) fn account(
    ix: &RawInstruction,
    index: usize,
    needed: usize,
) -> Result<soldecoders_types::Pubkey, DecodeError> {
    ix.account(index)
        .cloned()
        .ok_or(DecodeError::MissingAccounts {
            expected: needed,
            found: ix.accounts.len(),
        })
}
