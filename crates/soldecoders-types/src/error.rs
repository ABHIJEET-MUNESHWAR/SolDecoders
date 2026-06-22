//! Errors raised while decoding raw Solana instructions.

use thiserror::Error;

/// A failure encountered while turning a [`RawInstruction`](crate::RawInstruction)
/// into a typed [`DecodedEvent`](crate::DecodedEvent).
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DecodeError {
    /// No decoder is registered for the instruction's program id.
    #[error("no decoder registered for program {program}")]
    UnknownProgram {
        /// The base58 program id that had no registered decoder.
        program: String,
    },

    /// The instruction data was too short to contain the expected fields.
    #[error("instruction data too short: have {have} bytes, need {need}")]
    TooShort {
        /// Bytes actually present.
        have: usize,
        /// Bytes required to decode the discriminator and payload.
        need: usize,
    },

    /// The discriminator/tag did not match any known instruction for the program.
    #[error("unknown instruction discriminator {discriminator} for {program}")]
    UnknownDiscriminator {
        /// Program the instruction targeted.
        program: String,
        /// The unrecognised discriminator (hex or decimal rendering).
        discriminator: String,
    },

    /// The instruction referenced fewer accounts than the layout requires.
    #[error("expected at least {expected} accounts, found {found}")]
    MissingAccounts {
        /// Minimum accounts the layout requires.
        expected: usize,
        /// Accounts actually present.
        found: usize,
    },

    /// A field could not be parsed from raw bytes.
    #[error("malformed field `{field}`: {reason}")]
    MalformedField {
        /// The field that failed to parse.
        field: &'static str,
        /// Human-readable reason.
        reason: String,
    },
}

impl DecodeError {
    /// A stable, machine-readable code for logs, metrics, and API responses.
    pub const fn code(&self) -> &'static str {
        match self {
            Self::UnknownProgram { .. } => "unknown_program",
            Self::TooShort { .. } => "too_short",
            Self::UnknownDiscriminator { .. } => "unknown_discriminator",
            Self::MissingAccounts { .. } => "missing_accounts",
            Self::MalformedField { .. } => "malformed_field",
        }
    }

    /// Whether this represents "I don't know how to decode this" (a routing miss)
    /// rather than corrupt input. Routing misses are normal in a mixed stream.
    pub const fn is_unsupported(&self) -> bool {
        matches!(
            self,
            Self::UnknownProgram { .. } | Self::UnknownDiscriminator { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_are_stable() {
        assert_eq!(
            DecodeError::TooShort { have: 1, need: 9 }.code(),
            "too_short"
        );
        assert_eq!(
            DecodeError::MissingAccounts {
                expected: 3,
                found: 1
            }
            .code(),
            "missing_accounts"
        );
    }

    #[test]
    fn unsupported_classification() {
        assert!(DecodeError::UnknownProgram {
            program: "p".into()
        }
        .is_unsupported());
        assert!(!DecodeError::TooShort { have: 0, need: 1 }.is_unsupported());
    }
}
