//! # soldecoders-core
//!
//! The decoding engine. A [`DecoderRegistry`] maps program ids to
//! [`InstructionDecoder`] implementations; built-in decoders cover the SPL
//! Token, System, Raydium AMM v4 and Orca Whirlpool programs. Decoding is pure
//! and side-effect free, so a batch of instructions can be decoded in parallel
//! with [`decode_batch`].

#![forbid(unsafe_code)]

mod batch;
mod decoder;
mod programs;
mod registry;

pub use batch::decode_batch;
pub use decoder::InstructionDecoder;
pub use programs::{OrcaWhirlpoolDecoder, RaydiumAmmV4Decoder, SplTokenDecoder, SystemDecoder};
pub use registry::DecoderRegistry;
