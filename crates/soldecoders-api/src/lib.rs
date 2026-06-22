//! # soldecoders-api
//!
//! The GraphQL surface for SolDecoders. Decoding is pure, so every operation is
//! a query: decode a single instruction, a whole transaction, or a parallel
//! batch, and enumerate the supported programs.

#![forbid(unsafe_code)]

mod query;
mod schema;
mod types;

pub use query::QueryRoot;
pub use schema::{build_schema, ApiContext, SolDecodersSchema};
pub use types::{
    DecodeResultObject, DecodedEventObject, InstructionInput, SupportedProgramObject,
    TransactionInput,
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use base64::Engine;
    use soldecoders_core::DecoderRegistry;

    fn test_schema() -> SolDecodersSchema {
        build_schema(ApiContext::new(Arc::new(DecoderRegistry::builtin())))
    }

    #[tokio::test]
    async fn api_version_resolves() {
        let schema = test_schema();
        let resp = schema.execute("{ apiVersion }").await;
        assert!(resp.errors.is_empty());
        let json = resp.data.into_json().unwrap();
        assert_eq!(json["apiVersion"], env!("CARGO_PKG_VERSION"));
    }

    #[tokio::test]
    async fn lists_supported_programs() {
        let schema = test_schema();
        let resp = schema.execute("{ supportedPrograms { name } }").await;
        assert!(resp.errors.is_empty());
        let json = resp.data.into_json().unwrap();
        assert_eq!(json["supportedPrograms"].as_array().unwrap().len(), 4);
    }

    #[tokio::test]
    async fn decodes_system_transfer_instruction() {
        let schema = test_schema();
        let mut data = 2u32.to_le_bytes().to_vec();
        data.extend_from_slice(&1_000u64.to_le_bytes());
        let data_b64 = base64::engine::general_purpose::STANDARD.encode(&data);
        let q = format!(
            r#"{{ decodeInstruction(input: {{
                programId: "11111111111111111111111111111111",
                accounts: ["{a}", "{b}"],
                dataBase64: "{d}"
            }}) {{ success errorCode event {{ eventType lamports from to }} }} }}"#,
            a = "11111111111111111111111111111112",
            b = "11111111111111111111111111111113",
            d = data_b64,
        );
        let resp = schema.execute(q).await;
        assert!(resp.errors.is_empty(), "errors: {:?}", resp.errors);
        let json = resp.data.into_json().unwrap();
        let r = &json["decodeInstruction"];
        assert_eq!(r["success"], true);
        assert_eq!(r["event"]["eventType"], "system_transfer");
        assert_eq!(r["event"]["lamports"], "1000");
    }

    #[tokio::test]
    async fn reports_decode_error_without_failing_query() {
        let schema = test_schema();
        let q = r#"{ decodeInstruction(input: {
            programId: "11111111111111111111111111111111",
            accounts: [],
            dataBase64: ""
        }) { success errorCode } }"#;
        let resp = schema.execute(q).await;
        assert!(resp.errors.is_empty());
        let json = resp.data.into_json().unwrap();
        assert_eq!(json["decodeInstruction"]["success"], false);
        assert_eq!(json["decodeInstruction"]["errorCode"], "too_short");
    }

    #[tokio::test]
    async fn rejects_bad_base58_program_id() {
        let schema = test_schema();
        let q = r#"{ decodeInstruction(input: {
            programId: "not-base58!",
            accounts: [],
            dataBase64: ""
        }) { success } }"#;
        let resp = schema.execute(q).await;
        assert!(!resp.errors.is_empty());
    }
}
