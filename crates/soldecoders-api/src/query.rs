//! The GraphQL query root.

use async_graphql::{Context, Object, Result};

use soldecoders_core::decode_batch;

use crate::schema::{to_err, ApiContext};
use crate::types::{
    DecodeResultObject, InstructionInput, SupportedProgramObject, TransactionInput,
};

/// Read-only query root: all decoding operations are pure functions.
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// The running API version (crate version).
    async fn api_version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    /// The programs this server can decode.
    async fn supported_programs(&self, ctx: &Context<'_>) -> Result<Vec<SupportedProgramObject>> {
        let api = ctx.data::<ApiContext>()?;
        Ok(api
            .registry
            .supported_programs()
            .into_iter()
            .map(|(program_id, name)| SupportedProgramObject {
                program_id,
                name: name.to_string(),
            })
            .collect())
    }

    /// Decode a single instruction. Validation failures (bad base58/base64) are
    /// hard errors; decode failures are returned in the result's `errorCode`.
    async fn decode_instruction(
        &self,
        ctx: &Context<'_>,
        input: InstructionInput,
    ) -> Result<DecodeResultObject> {
        let api = ctx.data::<ApiContext>()?;
        let raw = input.into_raw().map_err(to_err)?;
        Ok(api.registry.decode_instruction(&raw).into())
    }

    /// Decode every instruction in a transaction, preserving order.
    async fn decode_transaction(
        &self,
        ctx: &Context<'_>,
        input: TransactionInput,
    ) -> Result<Vec<DecodeResultObject>> {
        let api = ctx.data::<ApiContext>()?;
        let raw = input.into_raw().map_err(to_err)?;
        Ok(api
            .registry
            .decode_transaction(&raw)
            .into_iter()
            .map(Into::into)
            .collect())
    }

    /// Decode a batch of instructions in parallel (rayon). Order is preserved.
    async fn decode_batch(
        &self,
        ctx: &Context<'_>,
        instructions: Vec<InstructionInput>,
    ) -> Result<Vec<DecodeResultObject>> {
        let api = ctx.data::<ApiContext>()?;
        let raws = instructions
            .into_iter()
            .map(InstructionInput::into_raw)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(to_err)?;
        Ok(decode_batch(&api.registry, &raws)
            .into_iter()
            .map(Into::into)
            .collect())
    }
}
