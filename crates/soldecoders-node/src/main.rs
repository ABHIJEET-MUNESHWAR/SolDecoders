//! Binary entry point.

#![forbid(unsafe_code)]

use anyhow::Result;
use clap::Parser;

use soldecoders_node::config::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
    let cli = Cli::parse();
    soldecoders_node::run(cli).await
}
