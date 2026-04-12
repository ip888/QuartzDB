# QuartzDB User Guide

**Version:** 0.1.0  
**Last Updated:** April 12, 2026  
**Platform:** Cloudflare Workers + Durable Objects

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Project Organization](#project-organization)
4. [Authentication](#authentication)
5. [API Reference](#api-reference)
6. [Vector Search with HNSW](#vector-search-with-hnsw)
7. [Multi-Tenant SaaS](#multi-tenant-saas)
8. [Development Guide](#development-guide)
9. [Deployment](#deployment)
10. [Security](#security)

---

## Overview

QuartzDB is a **serverless vector database** running on the Cloudflare Workers edge network. It provides:

- ✅ **Key-Value Storage** — Fast, persistent storage with Durable Objects
- ✅ **Vector Search** — HNSW (Hierarchical Navigable Small World) algorithm for similarity search
- ✅ **Edge Computing** — Deployed to 300+ locations worldwide
- ✅ **Zero Ops** — No servers to manage, auto-scaling
- ✅ **Multi-Tenant SaaS** — Signup, API key auth, usage tracking, Stripe billing
- ✅ **Sharded Architecture** — Fan-out search across multiple Durable Objects
- ✅ **Analytics** — Request metrics tracked in Analytics Engine

### Key Features

| Feature | Technology | Purpose |
|---------|------------|---------|
| **Storage** | Durable Objects + SQLite | Persistent key-value store |
| **Vector Search** | HNSW Algorithm | O(log n) nearest neighbor search |
| **Runtime** | Rust → WASM on V8 | Fast, secure, portable |
| **Edge Network** | Cloudflare Workers | Low latency globally |
| **Analytics** | Analytics Engine | Real-time request metrics |
| **Auth** | API keys (SHA-256 hashed in KV) | Multi-tenant isolation |
| **Billing** | Stripe webhooks + KV usage records | Per-tenant usage tracking |
| **Sharding** | Consistent-hash shard router | Horizontal scaling |

---

## Architecture

### High-Level Architecture

```
                        ┌─────────────────────────────────┐
                        │   Next.js Dashboard              │
                        │   (quartz-dashboard)             │
                        │   - Signup / Pricing / Playground│
                        └────────────┬────────────────────┘
                                     │
                        ┌────────────▼────────────────────┐
                        │  Cloudflare Edge (300+ DCs)      │
                        │  ┌───────────────────────────┐  │
                        │  │  Rate Limiting (CF native) │  │
                        │  └───────────┬───────────────┘  │
                        └──────────────┼──────────────────┘
                                       │
         ┌─────────────────────────────▼──────────────────────────┐
         │                   QuartzDB Worker (WASM)               │
         │  ┌─────────┐ ┌───────────┐ ┌───────────┐ ┌─────────┐ │
         │  │  Auth    │ │  Router   │ │ Validation│ │ Timeout │ │
         │  │(api key) │ │ (lib.rs)  │ │           │ │  Guard  │ │
         │  └────┬────┘ └─────┬─────┘ └───────────┘ └─────────┘ │
         │       │      ┌─────┴─────┐                             │
         │       │      │  Shard    │                             │
         │       │      │  Router   │                             │
         │       │      └──┬──┬──┬──┘                             │
         └───────┼─────────┼──┼──┼────────────────────────────────┘
                 │         │  │  │
      ┌──────────┘    ┌────┘  │  └────┐
      │               │       │       │
      ▼               ▼       ▼       ▼
┌──────────┐  ┌──────────┐  ...  ┌──────────┐
│ TENANT_KV│  │ Shard 0  │       │ Shard N  │
│ (KV      │  │ (Durable │       │ (Durable │
│  Store)  │  │  Object) │       │  Object) │
│          │  │  ┌──────┐│       │  ┌──────┐│
│ apikey:… │  │  │ HNSW ││       │  │ HNSW ││
│ tenant:… │  │  │ Index ││       │  │ Index ││
│ usage:…  │  │  └──────┘│       │  └──────┘│
│ stripe:… │  │  + SQLite │       │  + SQLite │
└──────────┘  └──────────┘       └──────────┘
```

### Request Flow

1. **Ingress**: Client → Cloudflare edge (nearest datacenter)
2. **Rate Limit**: Cloudflare native rate limiting (per-IP)
3. **Auth**: Extract API key → validate against env secret (legacy) or KV (multi-tenant)
4. **Usage Check**: For `qdb_*` keys, check monthly query/insert limits
5. **Validation**: Validate request body (dimensions, k value, etc.)
6. **Shard Routing**: Consistent hash routes vectors to the correct Durable Object
7. **Processing**: DO runs HNSW insert/search/delete
8. **Timeout Guard**: Elapsed-time check after DO call; fan-out breaks early if budget exceeded
9. **Billing**: Track query/insert count for multi-tenant keys
10. **Analytics**: Log request metrics to Analytics Engine

---

## Project Organization

### Directory Structure

```
QuartzDB/
├── quartz-faas/                   # Rust backend (Cloudflare Worker)
│   ├── src/
│   │   ├── lib.rs                 # Worker entry point, HTTP router
│   │   ├── api.rs                 # Request/response type definitions
│   │   ├── auth.rs                # API key extraction + validation
│   │   ├── billing.rs             # Stripe webhooks, usage tracking
│   │   ├── durable.rs             # Durable Objects (StorageObject + VectorIndexObject)
│   │   ├── error.rs               # FaasError enum
│   │   ├── monitoring.rs          # RequestMetrics, Timer, Analytics Engine
│   │   ├── platform.rs            # Cross-platform abstraction (time, random, crypto)
│   │   ├── ratelimit.rs           # Rate limiting (CF headers + token bucket)
│   │   ├── sharding.rs            # Consistent-hash shard router
│   │   ├── tenant.rs              # Multi-tenant CRUD, plan limits
│   │   ├── timeout.rs             # TimeoutGuard for elapsed-time checking
│   │   ├── validation.rs          # Input validation for all endpoints
│   │   └── vector/                # Vector search module
│   │       ├── mod.rs             # Module exports
│   │       ├── hnsw.rs            # HNSW algorithm implementation
│   │       └── simd.rs            # WASM SIMD distance functions
│   ├── wrangler.toml              # Cloudflare Workers configuration
│   └── Cargo.toml                 # Rust dependencies
│
├── quartz-dashboard/              # Next.js frontend
│   └── src/
│       ├── app/
│       │   ├── page.tsx           # Landing page
│       │   ├── dashboard/page.tsx # Dashboard (stats, playground)
│       │   ├── signup/page.tsx    # Tenant signup flow
│       │   ├── pricing/page.tsx   # Pricing tiers
│       │   ├── playground/page.tsx# Interactive API playground
│       │   └── docs/page.tsx      # Documentation page
│       └── lib/
│           ├── api.ts             # TypeScript API client
│           └── auth.tsx           # Auth context provider
│
├── tests/                         # Shell-based integration tests
│   ├── smoke_test.sh
│   ├── load_test.sh
│   ├── scenario_test.sh
│   └── quick_test.sh
│
├── docs/                          # Technical documentation
│   ├── HNSW_EXPLAINED.md
│   └── VECTOR_SEARCH_EXPLAINED.md
│
├── Cargo.toml                     # Workspace configuration
└── USER_GUIDE.md                  # This file
```

### Module Responsibilities

| Module | File | Responsibility |
|--------|------|----------------|
| **Router** | `lib.rs` | HTTP routing, middleware orchestration |
| **API** | `api.rs` | Request/response data structures |
| **Auth** | `auth.rs` | API key extraction, legacy + tenant validation |
| **Billing** | `billing.rs` | Stripe webhook handler, per-tenant usage counters |
| **Durable** | `durable.rs` | StorageObject (KV) + VectorIndexObject (HNSW) |
| **Error** | `error.rs` | `FaasError` enum with `thiserror` |
| **Monitoring** | `monitoring.rs` | RequestMetrics, Timer, Analytics Engine |
| **Platform** | `platform.rs` | Cross-platform: `now_ms()`, `random_f64()`, `sha256()`, `hmac_sha256()` |
| **Rate Limit** | `ratelimit.rs` | CF header check + token bucket (for DOs) |
| **Sharding** | `sharding.rs` | Consistent-hash router, fan-out merge |
| **Tenant** | `tenant.rs` | TenantManager CRUD, plan limits, key generation |
| **Timeout** | `timeout.rs` | TimeoutGuard for elapsed-time checking |
| **Validation** | `validation.rs` | Input validation for all API endpoints |
| **HNSW** | `vector/hnsw.rs` | Core HNSW algorithm (insert, search, delete) |
| **SIMD** | `vector/simd.rs` | WASM SIMD128 accelerated distance functions |

---

## Authentication

QuartzDB supports two authentication modes:

### Legacy Mode (Admin / Dev)

Set the `QUARTZ_API_KEY` secret via Wrangler:

```bash
wrangler secret put QUARTZ_API_KEY
```

Then include in requests:

```bash
curl -H "Authorization: Bearer YOUR_KEY" https://api.quartzdb.io/api/vector/stats
# or
curl -H "X-API-Key: YOUR_KEY" https://api.quartzdb.io/api/vector/stats
```

Legacy keys bypass tenant-level billing and usage limits.

### Multi-Tenant Mode (SaaS)

1. **Sign up** via `POST /api/tenants/signup` to get a `qdb_*` API key
2. Include the key in requests (same headers as above)
3. Each request is authenticated against KV, with billing and limits enforced

**Public endpoints** (no auth required): `/`, `/health`  
**Protected endpoints**: `/api/*` (except `/api/tenants/signup`)  
**Webhook endpoints**: `/webhooks/stripe` (verified by Stripe signature)

---

## API Reference

> **Base URL**: `https://api.quartzdb.io` (or your custom domain)  
> All protected endpoints require an `Authorization: Bearer <key>` or `X-API-Key: <key>` header.

### Health Check

**GET /health**

```bash
curl https://api.quartzdb.io/health
```

```json
{
  "status": "healthy",
  "service": "quartz-faas",
  "version": "0.1.0",
  "uptime_seconds": 12345,
  "checks": {
    "storage": "ok",
    "vector_index": "ok"
  }
}
```

### Key-Value Storage

#### Store Value — `POST /api/put`

```bash
curl -X POST https://api.quartzdb.io/api/put \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"key": "user:123", "value": "John Doe"}'
```

#### Retrieve Value — `GET /api/get/:key`

```bash
curl -H "Authorization: Bearer $API_KEY" \
  https://api.quartzdb.io/api/get/user:123
```

#### Delete Value — `DELETE /api/delete/:key`

```bash
curl -X DELETE -H "Authorization: Bearer $API_KEY" \
  https://api.quartzdb.io/api/delete/user:123
```

### Vector Operations

#### Insert Vector — `POST /api/vector/insert`

```bash
curl -X POST https://api.quartzdb.io/api/vector/insert \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "id": "doc-001",
    "vector": [0.1, 0.2, 0.3, 0.4],
    "metadata": {"title": "Document 1", "category": "tech"}
  }'
```

```json
{"success": true, "id": "doc-001", "message": "Vector inserted successfully"}
```

#### Batch Insert — `POST /api/vector/batch-insert`

```bash
curl -X POST https://api.quartzdb.io/api/vector/batch-insert \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "vectors": [
      {"id": "doc-001", "vector": [0.1, 0.2, 0.3, 0.4]},
      {"id": "doc-002", "vector": [0.5, 0.6, 0.7, 0.8]}
    ]
  }'
```

```json
{"success": true, "inserted": 2}
```

Maximum batch size: **100 vectors**.

#### Search Vectors — `POST /api/vector/search`

```bash
curl -X POST https://api.quartzdb.io/api/vector/search \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"vector": [0.1, 0.2, 0.3, 0.4], "k": 10}'
```

```json
{
  "results": [
    {"id": "doc-001", "score": 0.95, "metadata": {"title": "Document 1"}}
  ]
}
```

The `k` field is optional (defaults to 10). Results are merged from all shards and ranked by score.

#### Get Vector — `GET /api/vector/get/:id`

```bash
curl -H "Authorization: Bearer $API_KEY" \
  https://api.quartzdb.io/api/vector/get/doc-001
```

#### Delete Vector (soft-delete) — `DELETE /api/vector/delete/:id`

```bash
curl -X DELETE -H "Authorization: Bearer $API_KEY" \
  https://api.quartzdb.io/api/vector/delete/doc-001
```

#### Index Statistics — `GET /api/vector/stats`

```bash
curl -H "Authorization: Bearer $API_KEY" \
  https://api.quartzdb.io/api/vector/stats
```

```json
{
  "total_document_count": 5000,
  "total_vector_count": 5000,
  "total_storage_bytes": 2048000,
  "shard_count": 4,
  "shards": [
    {"shard_id": 0, "shard_name": "vector-index-0", "stats": {...}}
  ]
}
```

### Tenant & Billing

#### Sign Up — `POST /api/tenants/signup`

```bash
curl -X POST https://api.quartzdb.io/api/tenants/signup \
  -H "Content-Type: application/json" \
  -d '{"tenant_id": "my-project"}'
```

```json
{
  "success": true,
  "tenant_id": "my-project",
  "api_key": "qdb_a1b2c3d4e5f6...",
  "plan": "free",
  "vector_limit": 1000,
  "query_limit_per_month": 10000,
  "message": "Save your API key - it will not be shown again!"
}
```

> **Important**: The API key is shown only once. It is stored as a SHA-256 hash.

#### Usage — `GET /api/usage`

Requires a `qdb_*` API key.

```bash
curl -H "Authorization: Bearer qdb_..." \
  https://api.quartzdb.io/api/usage
```

```json
{
  "success": true,
  "tenant_id": "my-project",
  "plan": "Free",
  "usage": {"queries": 42, "inserts": 100, "vectors_stored": 100, "month": "2026-04"},
  "limits": {"vector_limit": 1000, "query_limit_per_month": 10000}
}
```

#### Stripe Webhook — `POST /webhooks/stripe`

Handles `checkout.session.completed`, `customer.subscription.updated`, and `customer.subscription.deleted` events. Verified via HMAC-SHA256 signature.

---

## Vector Search with HNSW

### What is HNSW?

**Hierarchical Navigable Small World** is a graph-based algorithm for approximate nearest neighbor search.

- **Multi-layer graph**: Higher layers for coarse navigation, layer 0 for fine-grained search
- **Greedy search**: Navigate to nearest neighbor at each step
- **Complexity**: O(log n) for both insert and search
- **Accuracy**: Highly accurate with tunable parameters

### Configuration Parameters

| Parameter | Description | Default | Tuning |
|-----------|-------------|---------|--------|
| `M` | Connections per node (layers 1+) | 16 | Higher = better recall, slower |
| `M₀` | Connections per node (layer 0) | 32 | Usually 2×M |
| `ef_construction` | Neighbors explored during insert | 200 | Higher = better graph quality |
| `ef_search` | Neighbors explored during search | 100 | Higher = better recall |

### Distance Metrics

| Metric | Best For | Range |
|--------|----------|-------|
| **Cosine** (default) | Normalized embeddings (OpenAI, Cohere) | [0, 2] |
| **Euclidean (L2)** | Geometric distance | [0, ∞) |
| **Dot Product** | Raw similarity score | (-∞, ∞) |

### Performance Characteristics

- **Insert**: O(log n) — ~1ms per vector
- **Search**: O(log n) — 1-5ms for 100K vectors
- **Memory**: ~1KB per vector (384-dim) including graph edges
- **Soft Delete**: O(1) — marks node as deleted, excluded from search results

---

## Multi-Tenant SaaS

### Pricing Plans

| Plan | Price | Vectors | Queries/Month |
|------|-------|---------|---------------|
| **Free** | $0 | 1,000 | 10,000 |
| **Pro** | $29/mo | 100,000 | 100,000 |
| **Scale** | $99/mo | 1,000,000 | Unlimited |

### Billing Flow

1. User signs up via `POST /api/tenants/signup` → Free tier
2. User upgrades via Stripe Checkout (set `client_reference_id` to tenant ID)
3. Stripe sends `checkout.session.completed` webhook → plan upgraded to Pro/Scale
4. Subscription changes (`updated`/`deleted`) propagate via webhook

### Usage Tracking

- Per-tenant monthly counters stored in KV (`usage:{tenant_id}:{YYYY-MM}`)
- Query limits checked **before** search (returns 429 if exceeded)
- Insert counts tracked **after** successful insertion
- Records auto-expire after 90 days

### KV Schema

| Key Pattern | Value | Purpose |
|-------------|-------|---------|
| `apikey:{sha256_hash}` | TenantInfo JSON | API key → tenant lookup |
| `tenant:{tenant_id}` | TenantInfo JSON | Tenant ID → tenant lookup |
| `stripe:{customer_id}` | `tenant_id` string | Stripe → tenant reverse mapping |
| `usage:{tenant_id}:{YYYY-MM}` | UsageRecord JSON | Monthly counters |

---

## Development Guide

### Prerequisites

- Rust 1.89+ with `wasm32-unknown-unknown` target
- Node.js 18+ (for dashboard)
- Wrangler CLI (`npm install -g wrangler`)

### Local Development

```bash
# Backend
cd quartz-faas
cargo build --target wasm32-unknown-unknown
wrangler dev

# Dashboard
cd quartz-dashboard
npm install
npm run dev
```

### Running Tests

```bash
# All Rust tests (cross-platform, no WASM target needed)
cargo test

# Integration tests (requires running worker)
wrangler dev &
bash tests/smoke_test.sh
bash tests/quick_test.sh
```

The test suite includes **98+ tests** covering:
- HNSW algorithm (insert, search, delete, serialization, all metrics)
- Billing (usage keys, monthly formatting, HMAC verification)
- Validation (all input validators, edge cases)
- Monitoring (timer, metrics lifecycle)
- Platform (SHA-256, HMAC-SHA256, time, random)
- Tenant (key generation, plan limits)
- Timeout (creation, expiry, check)
- Auth (key extraction, path protection)

---

## Deployment

### 1. Configure Secrets

```bash
cd quartz-faas

# Set API key for admin access
wrangler secret put QUARTZ_API_KEY

# Set Stripe webhook secret
wrangler secret put STRIPE_WEBHOOK_SECRET
```

### 2. Create KV Namespace

```bash
# Development
wrangler kv:namespace create "TENANT_KV"
# → Copy the id into wrangler.toml [[kv_namespaces]] id = "..."

# Production
wrangler kv:namespace create "TENANT_KV" --env production
# → Copy the id into [[env.production.kv_namespaces]] id = "..."
```

### 3. Deploy

```bash
# Development
wrangler deploy

# Production
wrangler deploy --env production
```

### 4. Verify

```bash
curl https://api.quartzdb.io/health
```

### wrangler.toml Key Settings

```toml
# Production CORS (restricts to dashboard origin)
[env.production.vars]
CORS_ALLOWED_ORIGIN = "https://dashboard.quartzdb.io"
```

---

## Security

### Authentication

- **API keys** sent via `Authorization: Bearer` or `X-API-Key` header
- Multi-tenant keys (`qdb_*`) are **SHA-256 hashed** before KV storage — raw keys are never persisted
- Stripe webhook signatures verified with **HMAC-SHA256** (RFC 2104) + **constant-time comparison**
- Webhook timestamps checked for **5-minute freshness** to prevent replay attacks

### Rate Limiting

- **Cloudflare native rate limiting** at the edge (per-IP, configured in dashboard)
- **Per-tenant monthly usage limits** enforced in the Worker (returns 429 when exceeded)

### CORS

- Development: `Access-Control-Allow-Origin: *`
- Production: Restricted to `CORS_ALLOWED_ORIGIN` env var (e.g. `https://dashboard.quartzdb.io`)

### Input Validation

All user inputs are validated before processing:
- Vector dimensions (1–4096, must be consistent)
- Search `k` parameter (1–1000)
- Batch size (1–100 vectors)
- Tenant ID format (alphanumeric + `_` `-`, max 64 chars)

---

## Troubleshooting

### Common Issues

**"CPU time limit exceeded"**  
HNSW search is CPU-intensive. Lower `ef_search` or upgrade to Workers Paid plan.

**"Usage limit exceeded" (429)**  
Tenant has hit their monthly query quota. Upgrade plan at `/pricing`.

**"Unauthorized: Invalid or missing API key" (401)**  
Check that your API key is included in headers and is valid.

**"Vector dimension mismatch"**  
All vectors in an index must have the same dimension. Check your embedding model output.

**"Batch too large"**  
Maximum 100 vectors per batch-insert request.

---

**Happy Building!**
