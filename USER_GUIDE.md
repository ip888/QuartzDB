# QuartzDB User Guide

**Version:** 0.1.0  
**Last Updated:** January 2, 2026  
**Platform:** Cloudflare Workers + Durable Objects

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Project Organization](#project-organization)
4. [How It Works](#how-it-works)
5. [API Reference](#api-reference)
6. [Vector Search with HNSW](#vector-search-with-hnsw)
7. [Development Guide](#development-guide)
8. [Deployment](#deployment)

---

## Overview

QuartzDB is a **serverless vector database** running on Cloudflare Workers edge network. It provides:

- ✅ **Key-Value Storage** - Fast, persistent storage with Durable Objects
- ✅ **Vector Search** - HNSW (Hierarchical Navigable Small World) algorithm for similarity search
- ✅ **Edge Computing** - Deployed to 300+ locations worldwide
- ✅ **Zero Ops** - No servers to manage, auto-scaling
- ✅ **Analytics** - Built-in monitoring with Analytics Engine

### Key Features

| Feature | Technology | Purpose |
|---------|------------|---------|
| **Storage** | Durable Objects + SQLite | Persistent key-value store |
| **Vector Search** | HNSW Algorithm | O(log n) nearest neighbor search |
| **Runtime** | WASM on V8 | Fast, secure, portable |
| **Edge Network** | Cloudflare Workers | Low latency globally |
| **Analytics** | Analytics Engine | Real-time metrics |

---

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Cloudflare Edge Network                  │
│                      (300+ Locations)                        │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      QuartzDB Worker                         │
│                      (WASM Runtime)                          │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  Router (worker-rs)                                  │   │
│  │  - /health, /api/*, /vector/*                        │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
           │                              │
           ▼                              ▼
┌─────────────────────┐      ┌──────────────────────────┐
│  StorageObject      │      │  VectorIndexObject       │
│  (Durable Object)   │      │  (Durable Object)        │
│  ┌────────────────┐ │      │  ┌──────────────────┐   │
│  │ HashMap Cache  │ │      │  │ HNSW Index       │   │
│  │ + SQLite       │ │      │  │ Multi-layer      │   │
│  └────────────────┘ │      │  │ Graph            │   │
└─────────────────────┘      │  └──────────────────┘   │
                             └──────────────────────────┘
                                         │
                                         ▼
                             ┌──────────────────────────┐
                             │   Analytics Engine       │
                             │   (Metrics Storage)      │
                             └──────────────────────────┘
```

### Component Breakdown

#### 1. **Worker (Main Entry Point)**
- **Location:** `quartz-faas/src/lib.rs`
- **Responsibility:** HTTP routing, request handling, health checks
- **Technology:** Rust + worker-rs + WASM
- **Execution:** Runs on every incoming request

#### 2. **Durable Objects (State Management)**

**StorageObject** - Key-Value Store
- **Location:** `quartz-faas/src/durable.rs`
- **Persistence:** SQLite backend (automatic)
- **Cache:** In-memory HashMap for fast reads
- **Operations:** PUT, GET, DELETE, LIST

**VectorIndexObject** - Vector Search Engine
- **Location:** `quartz-faas/src/durable.rs`
- **Algorithm:** HNSW (Hierarchical Navigable Small World)
- **Persistence:** Serialized HNSW graph to SQLite
- **Operations:** INSERT, SEARCH, STATS, CONFIG

#### 3. **HNSW Vector Search**
- **Location:** `quartz-faas/src/vector/hnsw.rs`
- **Algorithm:** Multi-layer proximity graph
- **Complexity:** O(log n) search, O(log n) insert
- **WASM-Compatible:** Uses `js_sys` for random numbers

#### 4. **Analytics Engine**
- **Location:** `quartz-faas/src/monitoring.rs`
- **Metrics:** Request latency, success/failure rates
- **Storage:** Cloudflare Analytics Engine
- **Retention:** Real-time + historical data

---

## Project Organization

### Directory Structure

```
QuartzDB/
├── quartz-faas/              # Main application (WASM-based)
│   ├── src/
│   │   ├── lib.rs            # Worker entry point + router
│   │   ├── api.rs            # Request/response types
│   │   ├── error.rs          # Error handling
│   │   ├── durable.rs        # Durable Objects (Storage + Vector)
│   │   ├── monitoring.rs     # Analytics Engine integration
│   │   └── vector/           # Vector search module
│   │       ├── mod.rs        # Module exports
│   │       └── hnsw.rs       # HNSW algorithm implementation
│   │
│   ├── wrangler.toml         # Cloudflare Workers configuration
│   └── Cargo.toml            # Rust dependencies
│
├── docs/
│   └── strategy/             # Strategic planning documents
│       ├── PRODUCTION_ROADMAP.md
│       ├── PASSIVE_INCOME_STRATEGY.md
│       ├── RESTART_PLAN.md
│       └── WEEK_1_ACTION_PLAN.md
│
├── Cargo.toml                # Workspace configuration
└── README.md                 # Project overview
```

### Module Responsibilities

| Module | File | Responsibility |
|--------|------|----------------|
| **Router** | `lib.rs` | HTTP routing, middleware, health checks |
| **API** | `api.rs` | Request/response data structures |
| **Error** | `error.rs` | Error types and conversions |
| **Storage** | `durable.rs` | Key-value store (StorageObject) |
| **Vector** | `durable.rs` | Vector index (VectorIndexObject) |
| **HNSW** | `vector/hnsw.rs` | HNSW algorithm implementation |
| **Monitoring** | `monitoring.rs` | Metrics collection and reporting |

---

## How It Works

### Request Flow

#### 1. **Client Request**
```
Client → Cloudflare Edge → QuartzDB Worker
```

#### 2. **Router Processing**
```rust
// Worker receives request
Router::new()
    .post_async("/api/put", handler)
    .get_async("/api/get/:key", handler)
    .post_async("/vector/insert", handler)
    .post_async("/vector/search", handler)
```

#### 3. **Durable Object Interaction**
```
Worker → Get Durable Object Stub → Forward Request → Process → Return Response
```

#### 4. **Analytics Tracking**
```rust
// Track metrics for every request
RequestMetrics::new(method, path)
    .with_duration(elapsed_ms)
    .with_status(status_code)
    .send_to_analytics(env)
```

### Data Flow Examples

#### Key-Value Storage

```
1. Client sends: POST /api/put {"key": "user:123", "value": "John Doe"}
2. Worker routes to StorageObject
3. StorageObject:
   - Updates in-memory cache
   - Writes to Durable Storage (SQLite)
4. Response: {"success": true, "key": "user:123"}
```

#### Vector Search

```
1. Client sends: POST /vector/search {"query": [0.1, 0.2, ...], "k": 10}
2. Worker routes to VectorIndexObject
3. VectorIndexObject:
   - Loads HNSW index from cache/storage
   - Runs HNSW search algorithm:
     a. Start at entry point (highest layer)
     b. Greedily navigate to nearest neighbors per layer
     c. Descend to layer 0
     d. Return k nearest neighbors
4. Response: {"results": [{id, score, metadata}...]}
```

### HNSW Algorithm Flow

```
Insert Vector:
1. Generate random level (exponential distribution)
2. Create node with connections at each layer
3. Find entry point (top layer node)
4. For each layer (top to bottom):
   - Search for nearest neighbors
   - Connect to M nearest nodes
   - Add bidirectional edges
   - Prune overconnected neighbors
5. Update entry point if new node is highest

Search Query:
1. Start at entry point (top layer)
2. For each layer (top to 1):
   - Greedy search for single nearest neighbor
   - Move to that neighbor
3. At layer 0:
   - Search for k nearest neighbors
   - Use ef_search parameter for quality
4. Return top k results by distance
```

---

## API Reference

### Health Check

**GET /health**

```bash
curl https://your-worker.workers.dev/health
```

Response:
```json
{
  "status": "healthy",
  "service": "quartz-faas",
  "version": "0.1.0",
  "uptime_seconds": 123456,
  "checks": {
    "storage": "ok",
    "vector_index": "ok"
  }
}
```

### Key-Value Storage

#### Store Value

**POST /api/put**

```bash
curl -X POST https://your-worker.workers.dev/api/put \
  -H "Content-Type: application/json" \
  -d '{"key": "user:123", "value": "John Doe"}'
```

Response:
```json
{
  "success": true,
  "key": "user:123",
  "message": "Value stored successfully"
}
```

#### Retrieve Value

**GET /api/get/:key**

```bash
curl https://your-worker.workers.dev/api/get/user:123
```

Response:
```json
{
  "success": true,
  "key": "user:123",
  "value": "John Doe",
  "source": "cache"
}
```

#### Delete Value

**DELETE /api/delete/:key**

```bash
curl -X DELETE https://your-worker.workers.dev/api/delete/user:123
```

### Vector Search

#### Insert Vector

**POST /vector/insert**

```bash
curl -X POST https://your-worker.workers.dev/vector/insert \
  -H "Content-Type: application/json" \
  -d '{
    "id": 123,
    "vector": [0.1, 0.2, 0.3, ...],
    "metadata": {"title": "Document 1", "category": "tech"}
  }'
```

Response:
```json
{
  "success": true,
  "id": 123,
  "message": "Vector inserted successfully"
}
```

#### Search Vectors

**POST /vector/search**

```bash
curl -X POST https://your-worker.workers.dev/vector/search \
  -H "Content-Type: application/json" \
  -d '{
    "query": [0.1, 0.2, 0.3, ...],
    "k": 10
  }'
```

Response:
```json
{
  "success": true,
  "algorithm": "HNSW",
  "results": [
    {
      "id": 123,
      "distance": 0.05,
      "score": 0.95,
      "metadata": {"title": "Document 1"}
    }
  ]
}
```

#### Get Statistics

**GET /vector/stats**

```bash
curl https://your-worker.workers.dev/vector/stats
```

Response:
```json
{
  "algorithm": "HNSW",
  "num_vectors": 10000,
  "num_nodes": 10000,
  "dimension": 384,
  "entry_point_level": 4,
  "connections_per_layer": [32000, 8000, 2000, 500, 125]
}
```

#### Configure HNSW

**POST /vector/config**

```bash
curl -X POST https://your-worker.workers.dev/vector/config \
  -H "Content-Type: application/json" \
  -d '{
    "dimension": 384,
    "metric": "cosine",
    "max_connections": 16,
    "ef_construction": 200,
    "ef_search": 100
  }'
```

---

## Vector Search with HNSW

### What is HNSW?

**Hierarchical Navigable Small World** is a graph-based algorithm for approximate nearest neighbor search.

**Key Characteristics:**
- **Multi-layer Graph:** Higher layers for coarse navigation, layer 0 for fine-grained search
- **Greedy Search:** Navigate to nearest neighbor at each step
- **Complexity:** O(log n) for both insert and search
- **Accuracy:** Highly accurate with tunable parameters

### Configuration Parameters

| Parameter | Description | Default | Tuning |
|-----------|-------------|---------|--------|
| `M` | Connections per node (layers 1+) | 16 | Higher = better recall, slower |
| `M₀` | Connections per node (layer 0) | 32 | Usually 2×M |
| `ef_construction` | Neighbors explored during insert | 200 | Higher = better graph, slower insert |
| `ef_search` | Neighbors explored during search | 100 | Higher = better recall, slower search |

### Performance Tuning

**For Speed:**
```json
{
  "max_connections": 8,
  "ef_construction": 100,
  "ef_search": 50
}
```

**For Accuracy:**
```json
{
  "max_connections": 32,
  "ef_construction": 400,
  "ef_search": 200
}
```

**Balanced (Default):**
```json
{
  "max_connections": 16,
  "ef_construction": 200,
  "ef_search": 100
}
```

### Distance Metrics

- **Cosine Similarity** - Best for normalized embeddings (default)
- **Euclidean (L2)** - Geometric distance
- **Dot Product** - Raw similarity score

---

## Development Guide

### Prerequisites

- Rust 1.70+ (`rustup install stable`)
- Node.js 18+ (`node --version`)
- Wrangler CLI (`npm install -g wrangler`)

### Local Development

```bash
# Clone repository
git clone <repo-url>
cd QuartzDB/quartz-faas

# Install dependencies
cargo build

# Run locally with Miniflare
wrangler dev

# Test endpoints
curl http://localhost:8787/health
```

### Running Tests

```bash
# Unit tests
cargo test

# Integration tests
wrangler dev &
python examples/simple_vector_demo.py
```

### Building for Production

```bash
# Optimize build
wrangler build

# Check bundle size
ls -lh build/
# Expected: ~700KB total, ~270KB gzipped
```

---

## Deployment

### Deploy to Cloudflare Workers

```bash
# Login to Cloudflare
wrangler login

# Deploy to production
cd quartz-faas
wrangler deploy

# Output:
# ✨ Published quartz-faas
# https://quartz-faas.<your-subdomain>.workers.dev
```

### Configure Durable Objects

Ensure `wrangler.toml` has:

```toml
[[durable_objects.bindings]]
name = "STORAGE"
class_name = "StorageObject"
script_name = "quartz-faas"

[[durable_objects.bindings]]
name = "VECTOR_INDEX"
class_name = "VectorIndexObject"
script_name = "quartz-faas"
```

### Monitor Performance

View analytics at:
```
https://dash.cloudflare.com → Workers → quartz-faas → Analytics
```

---

## Advanced Topics

### Scaling Considerations

- **Durable Objects:** Each DO instance handles up to ~1000 RPS
- **Multiple Instances:** Use different DO names for sharding
- **Vector Index Size:** ~100MB per 100K vectors (384-dim)
- **Cold Start:** First request ~50-100ms, subsequent <1ms

### Backup and Recovery

Durable Objects automatically persist to disk:
- **SQLite Storage:** Replicated to 3+ regions
- **No Manual Backups:** Handled by Cloudflare
- **Export Data:** Use LIST endpoint for manual export

### Security

- **Authentication:** Add Cloudflare Access or custom middleware
- **Rate Limiting:** Use Cloudflare Rate Limiting rules
- **CORS:** Configure in worker router if needed

---

## Troubleshooting

### Common Issues

**"CPU time limit exceeded"**
- HNSW search is CPU-intensive
- Solution: Lower `ef_search` or upgrade to Workers Paid plan

**"Durable Object not found"**
- Check `wrangler.toml` bindings
- Ensure DO migrations are published

**"Vector dimension mismatch"**
- All vectors must have same dimension
- Reconfigure index with correct dimension

### Debug Mode

```bash
# Run with verbose logging
RUST_LOG=debug wrangler dev
```

---

## Support

- **Documentation:** See `docs/` folder
- **Issues:** GitHub Issues
- **Community:** Cloudflare Workers Discord

---

**Happy Building! 🚀**
