# Changelog

All notable changes to QuartzDB are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added

- **Multi-tenant SaaS architecture** — API key authentication (`qdb_` prefix), per-tenant usage tracking, and tiered rate limits (Free / Pro / Enterprise).
- **Stripe billing integration** — Webhook-driven subscription management, usage-based metering, and HMAC-SHA256 signature verification for webhook security.
- **Signup and pricing pages** in the Next.js dashboard (`/signup`, `/pricing`).
- **Platform abstraction layer** (`platform.rs`) — Cross-platform implementations for `now_ms()`, `random_f64()`, `sha256()`, `hmac_sha256()`, `constant_time_eq()`, `hex_encode()`, and `current_year_month()`. Centralises all JS/WASM interop.
- **Shard-based vector routing** — Consistent hashing distributes vectors across Durable Object shards; search fans out to all shards and merges results.
- **Request validation** — Dimension checks, ID length limits, batch-size caps, and `k` range enforcement (1-100).
- **Timeout guards** — Configurable per-operation timeouts for vector insert, search, batch-insert, get, and delete.
- **Rate limiting** — Token-bucket rate limiter via Durable Objects for per-tenant request throttling.
- **Monitoring** — Analytics Engine integration for request metrics (latency, status codes, operation types).
- **CORS support** — Permissive CORS headers on all responses for browser-based clients.
- **cargo-deny configuration** (`deny.toml`) — License allowlist, advisory database checks, and source restrictions.
- **rust-toolchain.toml** — Pins Rust 1.94.1 with `wasm32-unknown-unknown` target and `rustfmt`/`clippy` components.
- **CI/CD overhaul** — Replaced deprecated `actions-rs` with `dtolnay/rust-toolchain`, added WASM build verification, `cargo-deny` audit, and MSRV check.
- **Comprehensive test suite** — 108+ unit tests covering HNSW operations, billing, auth, validation, platform, sharding, and SIMD.
- **Documentation** — `USER_GUIDE.md`, rewritten `VECTOR_SEARCH_EXPLAINED.md`, `HNSW_EXPLAINED.md`, inline doc comments on all public items.
- `SECURITY.md` — Responsible disclosure policy.

### Changed

- Upgraded Rust edition from 2021 to **2024** and minimum supported version to **1.94.1**.
- Updated `worker` crate from 0.7.2 to **0.7.5**, `wasm-bindgen` to **0.2.117**, `js-sys` to **0.3.91**.
- Replaced `unsafe { std::mem::transmute(tid) }` in `platform.rs` with safe `tid.as_u64().unwrap_or(1)`.
- Replaced `top_k` search parameter with `k` across API and validation.
- `rustfmt.toml` edition updated to match Cargo edition (2024).
- Added `#![deny(warnings)]` and `#![warn(clippy::pedantic)]` crate-level lint attributes.

### Removed

- Dead workspace dependencies: `tokio`, `tracing`, `tracing-subscriber`, `tracing-appender`, `async-trait`, `proptest`, `config`, `validator`, `futures`.
- Stale TODO comments referencing completed HMAC and CORS work.
- CI jobs referencing non-existent `quartz-server` binary and Python test runners.

## [0.1.0] - 2025-10-17

### Added

- Initial release.
- HNSW vector index with cosine similarity (384 dimensions).
- Cloudflare Workers deployment with Durable Objects storage.
- Key-value CRUD operations (`/api/put`, `/api/get/:key`, `/api/delete/:key`).
- Vector CRUD operations (`/api/vector/insert`, `/api/vector/search`, `/api/vector/get/:id`, `/api/vector/delete/:id`, `/api/vector/batch-insert`).
- Health check endpoint (`/health`).
- Next.js dashboard with playground and docs pages.
