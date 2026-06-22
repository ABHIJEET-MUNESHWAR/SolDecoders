//! # soldecoders-types
//!
//! Pure domain types for the SolDecoders library — no I/O, no framework
//! dependencies. Everything here is `serde`-serialisable so the same model can
//! flow from the decoder core through the GraphQL API and into downstream
//! streaming systems unchanged.

#![forbid(unsafe_code)]

mod error;
mod event;
mod ids;
mod instruction;

pub use error::DecodeError;
pub use event::{DecodedEvent, ProgramKind, TokenAmount};
pub use ids::{Pubkey, Signature};
pub use instruction::{RawInstruction, RawTransaction};
