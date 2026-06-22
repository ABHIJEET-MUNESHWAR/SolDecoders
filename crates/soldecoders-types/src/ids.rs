//! Base58 identifier newtypes: [`Pubkey`] and [`Signature`].
//!
//! Solana public keys are 32 bytes and signatures are 64 bytes, both rendered
//! in base58 on the wire. These newtypes validate length at construction so an
//! invalid key cannot be represented downstream (compile-time-adjacent safety
//! via the type system).

use std::fmt;

use serde::{Deserialize, Serialize};

/// A 32-byte Solana public key / program id.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Pubkey([u8; 32]);

impl Pubkey {
    /// Construct from raw bytes.
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Parse from a base58 string, validating the decoded length is 32 bytes.
    ///
    /// # Errors
    /// Returns a message if the string is not valid base58 or not 32 bytes.
    pub fn from_base58(s: &str) -> Result<Self, String> {
        let raw = bs58::decode(s)
            .into_vec()
            .map_err(|e| format!("invalid base58: {e}"))?;
        let bytes: [u8; 32] = raw
            .try_into()
            .map_err(|v: Vec<u8>| format!("expected 32 bytes, got {}", v.len()))?;
        Ok(Self(bytes))
    }

    /// The raw 32 bytes.
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Render as a base58 string.
    pub fn to_base58(&self) -> String {
        bs58::encode(self.0).into_string()
    }
}

impl fmt::Display for Pubkey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_base58())
    }
}

impl fmt::Debug for Pubkey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Pubkey({})", self.to_base58())
    }
}

impl TryFrom<String> for Pubkey {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::from_base58(&s)
    }
}

impl From<Pubkey> for String {
    fn from(p: Pubkey) -> Self {
        p.to_base58()
    }
}

/// A 64-byte transaction signature, rendered in base58.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Signature(Vec<u8>);

impl Signature {
    /// Parse from a base58 string, validating the decoded length is 64 bytes.
    ///
    /// # Errors
    /// Returns a message if the string is not valid base58 or not 64 bytes.
    pub fn from_base58(s: &str) -> Result<Self, String> {
        let raw = bs58::decode(s)
            .into_vec()
            .map_err(|e| format!("invalid base58: {e}"))?;
        if raw.len() != 64 {
            return Err(format!("expected 64 bytes, got {}", raw.len()));
        }
        Ok(Self(raw))
    }

    /// Render as a base58 string.
    pub fn to_base58(&self) -> String {
        bs58::encode(&self.0).into_string()
    }
}

impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_base58())
    }
}

impl fmt::Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Signature({})", self.to_base58())
    }
}

impl TryFrom<String> for Signature {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::from_base58(&s)
    }
}

impl From<Signature> for String {
    fn from(s: Signature) -> Self {
        s.to_base58()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_key() -> Pubkey {
        // SPL Token program id.
        Pubkey::from_base58("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap()
    }

    #[test]
    fn pubkey_roundtrip_base58() {
        let k = sample_key();
        let s = k.to_base58();
        let k2 = Pubkey::from_base58(&s).unwrap();
        assert_eq!(k, k2);
        assert_eq!(s, "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
    }

    #[test]
    fn pubkey_rejects_wrong_length() {
        let err = Pubkey::from_base58("abc").unwrap_err();
        assert!(err.contains("expected 32 bytes"));
    }

    #[test]
    fn pubkey_rejects_bad_base58() {
        assert!(Pubkey::from_base58("0OIl").is_err());
    }

    #[test]
    fn pubkey_serde_is_base58_string() {
        let k = sample_key();
        let json = serde_json::to_string(&k).unwrap();
        assert_eq!(json, "\"TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA\"");
        let back: Pubkey = serde_json::from_str(&json).unwrap();
        assert_eq!(k, back);
    }

    #[test]
    fn signature_validates_length() {
        let sig = Signature(vec![7u8; 64]);
        let s = sig.to_base58();
        let back = Signature::from_base58(&s).unwrap();
        assert_eq!(sig, back);
        assert!(Signature::from_base58("abc").is_err());
    }

    #[test]
    fn debug_renders_base58() {
        assert!(format!("{:?}", sample_key()).starts_with("Pubkey(Tokenkeg"));
    }
}
