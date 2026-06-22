//! GraphQL input/output types and conversions from the domain model.

use async_graphql::{InputObject, SimpleObject};
use base64::Engine;

use soldecoders_types::{
    DecodeError, DecodedEvent, Pubkey, RawInstruction, RawTransaction, Signature,
};

/// A program registered in the decoder registry.
#[derive(SimpleObject)]
pub struct SupportedProgramObject {
    /// Base58 program id.
    pub program_id: String,
    /// Protocol name (e.g. `spl_token`).
    pub name: String,
}

/// A raw instruction supplied for decoding.
#[derive(InputObject)]
pub struct InstructionInput {
    /// Base58 program id.
    pub program_id: String,
    /// Base58 account keys in instruction order.
    pub accounts: Vec<String>,
    /// Standard-base64 instruction data.
    pub data_base64: String,
}

impl InstructionInput {
    /// Convert to the domain [`RawInstruction`], validating ids and base64.
    pub fn into_raw(self) -> Result<RawInstruction, String> {
        let program_id = Pubkey::from_base58(&self.program_id)?;
        let accounts = self
            .accounts
            .iter()
            .map(|a| Pubkey::from_base58(a))
            .collect::<Result<Vec<_>, _>>()?;
        let data = base64::engine::general_purpose::STANDARD
            .decode(&self.data_base64)
            .map_err(|e| format!("invalid base64 data: {e}"))?;
        Ok(RawInstruction::new(program_id, accounts, data))
    }
}

/// A raw transaction supplied for decoding.
#[derive(InputObject)]
pub struct TransactionInput {
    /// Base58 signatures (first is canonical).
    pub signatures: Vec<String>,
    /// Instructions to decode, in order.
    pub instructions: Vec<InstructionInput>,
}

impl TransactionInput {
    /// Convert to the domain [`RawTransaction`].
    pub fn into_raw(self) -> Result<RawTransaction, String> {
        let signatures = self
            .signatures
            .iter()
            .map(|s| Signature::from_base58(s))
            .collect::<Result<Vec<_>, _>>()?;
        let instructions = self
            .instructions
            .into_iter()
            .map(InstructionInput::into_raw)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(RawTransaction::new(signatures, instructions))
    }
}

/// A flattened, GraphQL-friendly view of a [`DecodedEvent`].
///
/// All variant-specific fields are optional; `event_type` and `program_kind`
/// are always present so clients can branch deterministically.
#[derive(SimpleObject, Default)]
pub struct DecodedEventObject {
    /// Stable event type (e.g. `spl_transfer`).
    pub event_type: String,
    /// Program family (e.g. `spl_token`).
    pub program_kind: String,
    /// Source account (transfers/burns).
    pub source: Option<String>,
    /// Destination account (transfers/mints).
    pub destination: Option<String>,
    /// Signing authority / owner.
    pub authority: Option<String>,
    /// Token mint, when carried.
    pub mint: Option<String>,
    /// Raw token amount.
    pub amount_raw: Option<String>,
    /// Declared decimals, when known.
    pub amount_decimals: Option<u8>,
    /// System transfer funding account.
    pub from: Option<String>,
    /// System transfer recipient.
    pub to: Option<String>,
    /// Lamports moved (system transfer).
    pub lamports: Option<String>,
    /// AMM/pool program (swap).
    pub program: Option<String>,
    /// User authority (swap).
    pub user: Option<String>,
    /// Amount in (swap).
    pub amount_in: Option<String>,
    /// Minimum amount out (swap).
    pub min_amount_out: Option<String>,
    /// Protocol label (swap).
    pub protocol: Option<String>,
}

impl From<DecodedEvent> for DecodedEventObject {
    fn from(e: DecodedEvent) -> Self {
        let mut o = DecodedEventObject {
            event_type: e.event_type().to_string(),
            program_kind: e.kind().label().to_string(),
            ..Default::default()
        };
        match e {
            DecodedEvent::SplTransfer {
                source,
                destination,
                authority,
                mint,
                amount,
            } => {
                o.source = Some(source.to_base58());
                o.destination = Some(destination.to_base58());
                o.authority = Some(authority.to_base58());
                o.mint = mint.map(|m| m.to_base58());
                o.amount_raw = Some(amount.raw.to_string());
                o.amount_decimals = amount.decimals;
            }
            DecodedEvent::SplMint {
                mint,
                destination,
                authority,
                amount,
            } => {
                o.mint = Some(mint.to_base58());
                o.destination = Some(destination.to_base58());
                o.authority = Some(authority.to_base58());
                o.amount_raw = Some(amount.raw.to_string());
                o.amount_decimals = amount.decimals;
            }
            DecodedEvent::SplBurn {
                source,
                mint,
                authority,
                amount,
            } => {
                o.source = Some(source.to_base58());
                o.mint = Some(mint.to_base58());
                o.authority = Some(authority.to_base58());
                o.amount_raw = Some(amount.raw.to_string());
                o.amount_decimals = amount.decimals;
            }
            DecodedEvent::SystemTransfer { from, to, lamports } => {
                o.from = Some(from.to_base58());
                o.to = Some(to.to_base58());
                o.lamports = Some(lamports.to_string());
            }
            DecodedEvent::DexSwap {
                program,
                user,
                amount_in,
                min_amount_out,
                protocol,
            } => {
                o.program = Some(program.to_base58());
                o.user = Some(user.to_base58());
                o.amount_in = Some(amount_in.to_string());
                o.min_amount_out = min_amount_out.map(|v| v.to_string());
                o.protocol = Some(protocol);
            }
        }
        o
    }
}

/// The outcome of decoding one instruction: either an event, or an error code.
#[derive(SimpleObject)]
pub struct DecodeResultObject {
    /// Whether decoding succeeded.
    pub success: bool,
    /// The decoded event, when successful.
    pub event: Option<DecodedEventObject>,
    /// Stable error code, when unsuccessful.
    pub error_code: Option<String>,
    /// Human-readable error message, when unsuccessful.
    pub error: Option<String>,
}

impl From<Result<DecodedEvent, DecodeError>> for DecodeResultObject {
    fn from(r: Result<DecodedEvent, DecodeError>) -> Self {
        match r {
            Ok(event) => DecodeResultObject {
                success: true,
                event: Some(event.into()),
                error_code: None,
                error: None,
            },
            Err(e) => DecodeResultObject {
                success: false,
                event: None,
                error_code: Some(e.code().to_string()),
                error: Some(e.to_string()),
            },
        }
    }
}
