//! The `decode` subcommand: decode one instruction and print JSON.

use anyhow::{anyhow, Result};
use base64::Engine;

use soldecoders_core::DecoderRegistry;
use soldecoders_types::{Pubkey, RawInstruction};

use crate::config::DecodeArgs;

/// Decode a single instruction described on the command line.
pub fn run_decode(args: DecodeArgs) -> Result<()> {
    let registry = DecoderRegistry::builtin();
    let ix = build_instruction(&args)?;
    let result = registry.decode_instruction(&ix);
    let json = match result {
        Ok(event) => serde_json::to_string_pretty(&event)?,
        Err(e) => serde_json::to_string_pretty(&serde_json::json!({
            "error": e.to_string(),
            "code": e.code(),
        }))?,
    };
    println!("{json}");
    Ok(())
}

/// Build a [`RawInstruction`] from parsed CLI arguments.
fn build_instruction(args: &DecodeArgs) -> Result<RawInstruction> {
    let program_id = Pubkey::from_base58(&args.program_id).map_err(|e| anyhow!(e))?;
    let accounts = if args.accounts.trim().is_empty() {
        Vec::new()
    } else {
        args.accounts
            .split(',')
            .map(|a| Pubkey::from_base58(a.trim()).map_err(|e| anyhow!(e)))
            .collect::<Result<Vec<_>>>()?
    };
    let data = base64::engine::general_purpose::STANDARD
        .decode(args.data_base64.trim())
        .map_err(|e| anyhow!("invalid base64 data: {e}"))?;
    Ok(RawInstruction::new(program_id, accounts, data))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_system_transfer_instruction() {
        let mut data = 2u32.to_le_bytes().to_vec();
        data.extend_from_slice(&1_000u64.to_le_bytes());
        let args = DecodeArgs {
            program_id: "11111111111111111111111111111111".into(),
            accounts: "11111111111111111111111111111112,11111111111111111111111111111113".into(),
            data_base64: base64::engine::general_purpose::STANDARD.encode(&data),
        };
        let ix = build_instruction(&args).unwrap();
        assert_eq!(ix.accounts.len(), 2);
        let event = DecoderRegistry::builtin().decode_instruction(&ix).unwrap();
        assert_eq!(event.event_type(), "system_transfer");
    }

    #[test]
    fn empty_accounts_yields_empty_vec() {
        let args = DecodeArgs {
            program_id: "11111111111111111111111111111111".into(),
            accounts: "  ".into(),
            data_base64: String::new(),
        };
        let ix = build_instruction(&args).unwrap();
        assert!(ix.accounts.is_empty());
        assert!(ix.data.is_empty());
    }

    #[test]
    fn rejects_bad_program_id() {
        let args = DecodeArgs {
            program_id: "bad!".into(),
            accounts: String::new(),
            data_base64: String::new(),
        };
        assert!(build_instruction(&args).is_err());
    }

    #[test]
    fn run_decode_handles_unknown_program_gracefully() {
        // A valid-but-unregistered program id should print an error JSON, not panic.
        let args = DecodeArgs {
            program_id: "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin".into(),
            accounts: String::new(),
            data_base64: String::new(),
        };
        assert!(run_decode(args).is_ok());
    }
}
