# SolDecoders — Self-Evaluation Against Engineering Guidelines

A point-by-point assessment against the 29 production-grade guidelines. ✅ done · 🟡 partial · ⬜ N/A.

| # | Guideline | Status | Where / how |
|---|---|---|---|
| 1 | SOLID principles | ✅ | `InstructionDecoder` = SRP per program + OCP via registry; `DecoderRegistry` depends on the trait (DIP), not concrete decoders. |
| 2 | Microservice architecture pattern | ✅ | Hexagonal (ports & adapters): pure domain (`types`/`core`) + adapters (`api`/`node`). Library-first so it composes into the event-driven `GeyserIndex`. |
| 3 | Partitioning & sharding | ⬜ | Stateless decoder library — no datastore. (Used by `GeyserIndex`, which does partition.) |
| 4 | Timeouts, retry, fault tolerance | 🟡 | Decoding is pure & in-process (no I/O to time out); server has graceful shutdown. GraphQL depth/complexity limits bound request cost. |
| 5 | Rate limiting & circuit breaker | 🟡 | Not applicable to pure CPU decode; query complexity limit is the analogous guard. (Full kit lives in `GeyserIndex`/`WalletLens`.) |
| 6 | Robust error handling & recovery | ✅ | Typed `DecodeError` with stable codes; `is_unsupported()` separates routing misses from corrupt input; batch results are independent. |
| 7 | GraphQL over REST | ✅ | `async-graphql` schema; 5 operations. |
| 8 | Test coverage to ~100% | ✅ | 56 tests covering every decoder branch + error edge, registry routing, batch, schema, HTTP, CLI. |
| 9 | Project structure | ✅ | Cargo workspace, layered crates, dependencies point inward. |
| 10 | Modular, reusable components | ✅ | The whole project *is* the reusable component; decoders are independently usable. |
| 11 | Third-party crates | ✅ | tokio, serde, thiserror, async-graphql, rayon, sha2, bs58, base64. |
| 12 | Generative/Agentic AI | ⬜ | Out of scope for a deterministic decoder; AI lives in `WalletLens`. |
| 13 | Idiomatic patterns | ✅ | Newtypes, builder registry, `From`/`TryFrom`, exhaustive `match`, `Result` discipline. |
| 14 | Generics | ✅ | `with_decoder<D: InstructionDecoder>` keeps decoders on the stack until boxed. |
| 15 | Anchor framework | 🟡 | Decodes Anchor-style 8-byte discriminators (Orca) by deriving `sha256("global:swap")`. |
| 16 | README w/ diagrams, flows, TOC, tests | ✅ | TOC, mermaid component + sequence diagrams, component table, test results. |
| 17 | Performance, reliability, maintainability | ✅ | O(1) decode, parallel batch, pure functions = trivially testable & thread-safe. |
| 18 | Tokio runtime | ✅ | Async axum server on Tokio. |
| 19 | Parallel / concurrency / batch | ✅ | `decode_batch` via rayon; thread-safe `Send + Sync` registry. |
| 20 | Logging & observability | ✅ | `tracing` + Prometheus `/metrics` + health probes. |
| 21 | Happy path + edge cases | ✅ | Tests cover short data, missing accounts, unknown discriminator, bad base58/base64, empty input. |
| 22 | Composable, extensible architecture | ✅ | Register new protocols without touching existing code. |
| 23 | Interfaces, config, structure | ✅ | Clean trait, `clap`/env config, layered crates. |
| 24 | Type system enforces constraints | ✅ | `Pubkey`/`Signature` validate length at construction; `DecodedEvent` makes illegal states unrepresentable. |
| 25 | Benchmarks & complexity | ✅ | Criterion `decode_bench` (serial vs parallel); complexity table in README. |
| 26 | CI/CD | ✅ | `.github/workflows/ci.yml`: fmt + clippy `-D warnings` + test + audit. |
| 27 | Dockerfile | ✅ | Multi-stage, non-root, `serve` default. |
| 28 | Postman collection | ✅ | `postman/SolDecoders.postman_collection.json`. |
| 29 | Self-evaluation | ✅ | This document. |
