//! The core decoder abstraction.

use soldecoders_types::{DecodeError, DecodedEvent, Pubkey, RawInstruction};

/// A decoder for a single Solana program.
///
/// Implementations are stateless and `Send + Sync` so the registry can be
/// shared across threads and used from [`crate::decode_batch`]. This is the
/// open/closed seam of the library: new protocols are added by implementing
/// this trait and registering the decoder — no existing code changes.
pub trait InstructionDecoder: Send + Sync {
    /// The program id this decoder handles.
    fn program_id(&self) -> &Pubkey;

    /// A stable, human-readable protocol name (e.g. `"spl_token"`).
    fn name(&self) -> &'static str;

    /// Decode a single raw instruction into a typed event.
    ///
    /// # Errors
    /// Returns a [`DecodeError`] if the data is too short, the discriminator is
    /// unknown, or required accounts are missing.
    fn decode(&self, ix: &RawInstruction) -> Result<DecodedEvent, DecodeError>;
}
