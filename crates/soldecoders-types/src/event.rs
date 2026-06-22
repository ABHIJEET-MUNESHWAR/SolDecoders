//! The typed, analytics-ready event hierarchy produced by decoders.

use serde::{Deserialize, Serialize};

use crate::ids::Pubkey;

/// A token amount in base units (no decimal scaling applied), with the optional
/// declared decimals when the instruction carries them (e.g. `transferChecked`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenAmount {
    /// Raw amount in the smallest unit.
    pub raw: u64,
    /// Declared decimals, when known.
    pub decimals: Option<u8>,
}

impl TokenAmount {
    /// A raw amount with unknown decimals.
    pub const fn raw(raw: u64) -> Self {
        Self {
            raw,
            decimals: None,
        }
    }

    /// A raw amount with declared decimals.
    pub const fn checked(raw: u64, decimals: u8) -> Self {
        Self {
            raw,
            decimals: Some(decimals),
        }
    }

    /// The human-scaled value (`raw / 10^decimals`) when decimals are known.
    pub fn ui_amount(&self) -> Option<f64> {
        self.decimals
            .map(|d| self.raw as f64 / 10f64.powi(i32::from(d)))
    }
}

/// The family a decoded instruction belongs to — useful for routing and metrics.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgramKind {
    /// The SPL Token (or Token-2022) program.
    SplToken,
    /// The native System program.
    System,
    /// A decentralised-exchange / AMM program.
    Dex,
    /// Anything else.
    Other,
}

impl ProgramKind {
    /// A stable lowercase label.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::SplToken => "spl_token",
            Self::System => "system",
            Self::Dex => "dex",
            Self::Other => "other",
        }
    }
}

/// A typed event decoded from a single raw instruction.
///
/// New variants can be added without breaking existing consumers thanks to the
/// `kind` tag and snake_case wire format; this is the composable seam that lets
/// downstream analytics (GeyserIndex, WalletLens) match on a stable schema.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DecodedEvent {
    /// An SPL token transfer (`Transfer` / `TransferChecked`).
    SplTransfer {
        /// Source token account.
        source: Pubkey,
        /// Destination token account.
        destination: Pubkey,
        /// The signing authority / owner.
        authority: Pubkey,
        /// The token mint, when the instruction carries it (`TransferChecked`).
        mint: Option<Pubkey>,
        /// Amount moved.
        amount: TokenAmount,
    },

    /// An SPL token mint (`MintTo` / `MintToChecked`).
    SplMint {
        /// The mint being increased.
        mint: Pubkey,
        /// The destination token account.
        destination: Pubkey,
        /// The mint authority.
        authority: Pubkey,
        /// Amount minted.
        amount: TokenAmount,
    },

    /// An SPL token burn (`Burn` / `BurnChecked`).
    SplBurn {
        /// The token account burned from.
        source: Pubkey,
        /// The mint being decreased.
        mint: Pubkey,
        /// The signing authority / owner.
        authority: Pubkey,
        /// Amount burned.
        amount: TokenAmount,
    },

    /// A native SOL transfer via the System program.
    SystemTransfer {
        /// Funding account.
        from: Pubkey,
        /// Recipient account.
        to: Pubkey,
        /// Lamports moved.
        lamports: u64,
    },

    /// A swap on a DEX/AMM program.
    DexSwap {
        /// The AMM/pool program.
        program: Pubkey,
        /// The user-authority initiating the swap.
        user: Pubkey,
        /// Amount offered in (base units of the input token).
        amount_in: u64,
        /// Minimum acceptable output (slippage floor), when present.
        min_amount_out: Option<u64>,
        /// A human label for the protocol (e.g. "raydium_amm_v4").
        protocol: String,
    },
}

impl DecodedEvent {
    /// The program family this event belongs to.
    pub const fn kind(&self) -> ProgramKind {
        match self {
            Self::SplTransfer { .. } | Self::SplMint { .. } | Self::SplBurn { .. } => {
                ProgramKind::SplToken
            }
            Self::SystemTransfer { .. } => ProgramKind::System,
            Self::DexSwap { .. } => ProgramKind::Dex,
        }
    }

    /// A stable, lowercase event-type label for metrics and dashboards.
    pub const fn event_type(&self) -> &'static str {
        match self {
            Self::SplTransfer { .. } => "spl_transfer",
            Self::SplMint { .. } => "spl_mint",
            Self::SplBurn { .. } => "spl_burn",
            Self::SystemTransfer { .. } => "system_transfer",
            Self::DexSwap { .. } => "dex_swap",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_amount_ui_scaling() {
        assert_eq!(TokenAmount::checked(1_500_000, 6).ui_amount(), Some(1.5));
        assert_eq!(TokenAmount::raw(10).ui_amount(), None);
    }

    #[test]
    fn event_kind_and_type() {
        let e = DecodedEvent::SystemTransfer {
            from: Pubkey::new([1; 32]),
            to: Pubkey::new([2; 32]),
            lamports: 100,
        };
        assert_eq!(e.kind(), ProgramKind::System);
        assert_eq!(e.event_type(), "system_transfer");
        assert_eq!(e.kind().label(), "system");
    }

    #[test]
    fn event_serialises_with_kind_tag() {
        let e = DecodedEvent::SystemTransfer {
            from: Pubkey::new([1; 32]),
            to: Pubkey::new([2; 32]),
            lamports: 100,
        };
        let json = serde_json::to_value(&e).unwrap();
        assert_eq!(json["kind"], "system_transfer");
        assert_eq!(json["lamports"], 100);
    }

    #[test]
    fn dex_swap_kind() {
        let e = DecodedEvent::DexSwap {
            program: Pubkey::new([9; 32]),
            user: Pubkey::new([8; 32]),
            amount_in: 1000,
            min_amount_out: Some(900),
            protocol: "raydium_amm_v4".into(),
        };
        assert_eq!(e.kind(), ProgramKind::Dex);
        assert_eq!(e.event_type(), "dex_swap");
    }
}
