//! # soldecoders-node
//!
//! The composition root: wires the decoder registry into a GraphQL server and a
//! one-shot decode CLI, with structured tracing and Prometheus metrics.

#![forbid(unsafe_code)]

pub mod config;
pub mod decode;
pub mod startup;
pub mod telemetry;

use anyhow::Result;

use crate::config::{Cli, Command};

/// Dispatch the parsed CLI to the chosen subcommand.
pub async fn run(cli: Cli) -> Result<()> {
    telemetry::init_tracing(cli.log_json);
    match cli.command {
        Command::Serve(args) => startup::run_server(args).await,
        Command::Decode(args) => decode::run_decode(args),
    }
}
