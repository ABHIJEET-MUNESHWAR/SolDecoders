//! Schema assembly and shared request context.

use std::sync::Arc;

use async_graphql::{EmptyMutation, EmptySubscription, Error, Schema};

use soldecoders_core::DecoderRegistry;

use crate::query::QueryRoot;

/// Per-process context shared with every resolver.
#[derive(Clone)]
pub struct ApiContext {
    /// The shared decoder registry.
    pub registry: Arc<DecoderRegistry>,
}

impl ApiContext {
    /// Create a context around a shared registry.
    pub fn new(registry: Arc<DecoderRegistry>) -> Self {
        Self { registry }
    }
}

/// The concrete schema type — read-only (decoding is pure).
pub type SolDecodersSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

/// Build the executable schema with depth/complexity limits.
pub fn build_schema(ctx: ApiContext) -> SolDecodersSchema {
    Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(ctx)
        .limit_depth(12)
        .limit_complexity(512)
        .finish()
}

/// Convert any displayable error into an `async-graphql` error.
pub(crate) fn to_err<E: std::fmt::Display>(e: E) -> Error {
    Error::new(e.to_string())
}
