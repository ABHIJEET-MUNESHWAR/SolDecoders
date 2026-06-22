//! The raw, undecoded instruction/transaction model.
//!
//! This mirrors the shape of a Solana instruction as seen by a Geyser/RPC
//! consumer: a program id, the ordered list of account keys it touches, and an
//! opaque data blob. Decoders in `soldecoders-core` turn these into
//! [`DecodedEvent`](crate::DecodedEvent)s.

use serde::{Deserialize, Serialize};

use crate::ids::{Pubkey, Signature};

/// A single raw instruction within a transaction.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawInstruction {
    /// The program this instruction invokes.
    pub program_id: Pubkey,
    /// The ordered account keys referenced by the instruction.
    pub accounts: Vec<Pubkey>,
    /// The opaque instruction data (little-endian, program-specific layout).
    pub data: Vec<u8>,
}

impl RawInstruction {
    /// Convenience constructor.
    pub fn new(program_id: Pubkey, accounts: Vec<Pubkey>, data: Vec<u8>) -> Self {
        Self {
            program_id,
            accounts,
            data,
        }
    }

    /// The first `n` bytes of the data, or `None` if shorter.
    pub fn discriminator(&self, n: usize) -> Option<&[u8]> {
        self.data.get(..n)
    }

    /// Read a little-endian `u64` starting at `offset`.
    ///
    /// # Errors
    /// Returns `None` if fewer than 8 bytes remain at `offset`.
    pub fn read_u64_le(&self, offset: usize) -> Option<u64> {
        let end = offset.checked_add(8)?;
        let slice = self.data.get(offset..end)?;
        let arr: [u8; 8] = slice.try_into().ok()?;
        Some(u64::from_le_bytes(arr))
    }

    /// The account at `index`, if present.
    pub fn account(&self, index: usize) -> Option<&Pubkey> {
        self.accounts.get(index)
    }
}

/// A raw transaction: its signatures and the ordered instructions it carries.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawTransaction {
    /// Transaction signatures (the first is the canonical id).
    pub signatures: Vec<Signature>,
    /// The ordered instructions to decode.
    pub instructions: Vec<RawInstruction>,
}

impl RawTransaction {
    /// Convenience constructor.
    pub fn new(signatures: Vec<Signature>, instructions: Vec<RawInstruction>) -> Self {
        Self {
            signatures,
            instructions,
        }
    }

    /// The canonical signature (first), if any.
    pub fn id(&self) -> Option<&Signature> {
        self.signatures.first()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(byte: u8) -> Pubkey {
        Pubkey::new([byte; 32])
    }

    #[test]
    fn reads_le_u64() {
        let ix = RawInstruction::new(key(1), vec![], vec![3, 0, 1, 0, 0, 0, 0, 0, 0]);
        // tag byte (3) then u64 at offset 1 = 256
        assert_eq!(ix.read_u64_le(1), Some(256));
    }

    #[test]
    fn read_u64_guards_bounds() {
        let ix = RawInstruction::new(key(1), vec![], vec![1, 2, 3]);
        assert_eq!(ix.read_u64_le(0), None);
        assert_eq!(ix.read_u64_le(100), None);
    }

    #[test]
    fn discriminator_slices() {
        let ix = RawInstruction::new(key(1), vec![], vec![9, 9, 9, 9]);
        assert_eq!(ix.discriminator(2), Some(&[9u8, 9][..]));
        assert_eq!(ix.discriminator(10), None);
    }

    #[test]
    fn accounts_and_tx_id() {
        let ix = RawInstruction::new(key(1), vec![key(2), key(3)], vec![]);
        assert_eq!(ix.account(1), Some(&key(3)));
        assert_eq!(ix.account(5), None);
        let tx = RawTransaction::new(vec![], vec![ix]);
        assert!(tx.id().is_none());
    }
}
