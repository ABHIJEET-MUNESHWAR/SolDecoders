//! Command-line configuration.

use clap::{Args, Parser, Subcommand};

/// SolDecoders — a Solana instruction decoder library, server, and CLI.
#[derive(Debug, Parser)]
#[command(name = "soldecoders-node", version, about)]
pub struct Cli {
    /// Emit logs as structured JSON.
    #[arg(long, global = true, env = "SOLDECODERS_LOG_JSON")]
    pub log_json: bool,

    /// The subcommand to run.
    #[command(subcommand)]
    pub command: Command,
}

/// Available subcommands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Run the GraphQL server.
    Serve(ServeArgs),
    /// Decode a single instruction and print the result as JSON.
    Decode(DecodeArgs),
}

/// Arguments for the `serve` subcommand.
#[derive(Debug, Args, Clone)]
pub struct ServeArgs {
    /// Bind host.
    #[arg(long, default_value = "0.0.0.0", env = "SOLDECODERS_HOST")]
    pub host: String,
    /// Bind port.
    #[arg(long, default_value_t = 8080, env = "SOLDECODERS_PORT")]
    pub port: u16,
}

impl Default for ServeArgs {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".into(),
            port: 8080,
        }
    }
}

/// Arguments for the `decode` subcommand.
#[derive(Debug, Args, Clone)]
pub struct DecodeArgs {
    /// Base58 program id.
    #[arg(long)]
    pub program_id: String,
    /// Comma-separated base58 account keys, in instruction order.
    #[arg(long, default_value = "")]
    pub accounts: String,
    /// Standard-base64 instruction data.
    #[arg(long, default_value = "")]
    pub data_base64: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_serve_defaults() {
        let cli = Cli::parse_from(["soldecoders-node", "serve"]);
        match cli.command {
            Command::Serve(args) => {
                assert_eq!(args.port, 8080);
                assert_eq!(args.host, "0.0.0.0");
            }
            _ => panic!("expected serve"),
        }
    }

    #[test]
    fn parses_decode_args() {
        let cli = Cli::parse_from([
            "soldecoders-node",
            "decode",
            "--program-id",
            "11111111111111111111111111111111",
            "--accounts",
            "a,b",
            "--data-base64",
            "AgAAAA==",
        ]);
        match cli.command {
            Command::Decode(args) => {
                assert_eq!(args.program_id, "11111111111111111111111111111111");
                assert_eq!(args.accounts, "a,b");
            }
            _ => panic!("expected decode"),
        }
    }

    #[test]
    fn serve_default_impl() {
        assert_eq!(ServeArgs::default().port, 8080);
    }
}
