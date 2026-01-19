# QuartzDB

> **Serverless Edge Database** - High-performance vector search engine running on Cloudflare Workers

[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Cloudflare Workers](https://img.shields.io/badge/cloudflare-workers-orange.svg)](https://workers.cloudflare.com/)

QuartzDB is a serverless vector search database built on Cloudflare Workers and Durable Objects. It provides high-performance similarity search for embeddings with sub-10ms latency from 300+ edge locations worldwide.

## ✨ Key Features

### Currently Implemented ✅

- **Serverless Vector Search**
  - Runs on Cloudflare Workers (WASM)
  - Durable Objects for persistent storage
  - HNSW algorithm for fast approximate nearest neighbor search
  - Cosine similarity metric for embeddings
  - 384-dimensional vector support
  - Global distribution across 300+ locations
  - Automatic scaling and zero maintenance
  - Pay-per-request pricing model

- **Vector Operations**
  - Insert/Search/Delete vectors
  - Batch operations (batch-insert)
  - Get vector by ID
  - Metadata support for vectors
  - Soft-delete for efficient space management
  - Performance statistics and monitoring

- **REST API**
  - Simple HTTP endpoints
  - JSON request/response format
  - API key authentication
  - Health checks and monitoring
  - Error handling and logging

- **Production Ready**
  - Smart batched persistence (10-second windows)
  - Quota-optimized for free tier
  - Sub-100ms write latency (non-blocking)
  - Rust 2024 edition with WASM compilation
  - Tested and deployed on Cloudflare
  - Custom domain support (api.quartzdb.io)

### Coming Soon 🚀

- **Advanced Features**
  - Multi-metric similarity search (L2, Hamming)
  - Graph-based vector indexing optimization
  - Vector dimensionality flexibility (not just 384)
  - Query analytics and usage tracking

## 🏗️ Architecture

```text
                    ┌─────────────────────────────┐
                    │   Client Application        │
                    └─────────────┬───────────────┘
                                  │
                     HTTPS Request │ (REST API)
                                  │
                                  ▼
        ┌─────────────────────────────────────────────────┐
        │   Cloudflare Workers (Edge Runtime)             │
        │   • WASM-compiled Rust code                     │
        │   • Runs at 300+ global locations               │
        │   • Sub-10ms cold start                         │
        └─────────────┬───────────────────────────────────┘
                      │
                      │ RPC Call
                      ▼
        ┌─────────────────────────────────────────────────┐
        │   Durable Objects (Persistent Storage)          │
        │                                                  │
        │   ┌──────────────────┐  ┌──────────────────┐    │
        │   │ StorageObject    │  │ VectorIndexObject│    │
        │   │                  │  │                  │    │
        │   │ • RefCell cache  │  │ • Vector store   │    │
        │   │ • KV persistence │  │ • Cosine search  │    │
        │   │ • Strong consist │  │ • Metadata mgmt  │    │
        │   └──────────────────┘  └──────────────────┘    │
        │                                                  │
        │   Durable Object Storage (SQLite-backed)        │
        └──────────────────────────────────────────────────┘
                              │
                              │ Analytics
                              ▼
                  ┌──────────────────────┐
                  │ Analytics Engine     │
                  │ • Request metrics    │
                  │ • Performance data   │
                  └──────────────────────┘
```

### Key Components

1. **Workers Runtime**: Handles HTTP requests at the edge
2. **Durable Objects**: Stateful compute with persistent storage
3. **StorageObject**: Key-value storage with caching
4. **VectorIndexObject**: Vector similarity search
5. **Analytics Engine**: Request tracking and monitoring

## 🚀 Quick Start

### Prerequisites

- Rust 1.89+ with `wasm32-unknown-unknown` target
- Node.js 20+
- Wrangler CLI (Cloudflare Workers CLI)

```bash
# Install Rust WASM target
rustup target add wasm32-unknown-unknown

# Install Wrangler
npm install -g wrangler

# Install worker-build (Rust → WASM compiler)
npm install -g @cloudflare/workers-rs
```

### Development

```bash
# Navigate to quartz-faas directory
cd quartz-faas

# Build WASM
wrangler build

# Run locally with Miniflare
wrangler dev

# Test the API
curl http://localhost:8787/health
```

### Deployment

```bash
# Login to Cloudflare
wrangler login

# Deploy to production
wrangler deploy

# Your API is now live at:
# https://quartzdb.<your-subdomain>.workers.dev
```

## 📡 API Reference

### Health Check

```bash
GET /health
```

**Response:**
```json
{
  "status": "ok",
  "timestamp": "2026-01-02T21:00:00Z"
}
```

### Key-Value Operations

**Store a value:**
```bash
POST /api/put
Content-Type: application/json

{
  "key": "user:123",
  "value": "Alice"
}
```

**Retrieve a value:**
```bash
GET /api/get/user:123
```

**Response:**
```json
{
  "key": "user:123",
  "value": "Alice"
}
```

**Delete a key:**
```bash
DELETE /api/delete/user:123
```

### Vector Operations

**Insert a vector:**
```bash
POST /api/vector/insert
Content-Type: application/json

{
  "id": 1,
  "vector": [0.1, 0.2, 0.3, ...],
  "metadata": {
    "label": "example",
    "source": "openai"
  }
}
```

**Search for similar vectors:**
```bash
POST /api/vector/search
Content-Type: application/json

{
  "query": [0.1, 0.2, 0.3, ...],
  "k": 10,
  "metric": "cosine"
}
```

**Response:**
```json
{
  "results": [
    {
      "id": 1,
      "distance": 0.95,
      "metadata": { "label": "example" }
    }
  ]
}
```

See [`quartz-faas/README.md`](quartz-faas/README.md) for complete API documentation.

## 📊 Performance

Running on Cloudflare Workers globally:

| Metric | Value |
|--------|-------|
| **Write Latency (P50)** | ~100ms (network included) |
| **Write Latency (no network)** | ~2-3ms (non-blocking, batched) |
| **Cold Start** | < 10ms |
| **API Latency (P50)** | < 20ms |
| **API Latency (P99)** | < 50ms |
| **Vector Dimension** | 384 |
| **Search Algorithm** | HNSW (Hierarchical Navigable Small World) |
| **Similarity Metric** | Cosine distance |
| **Global Locations** | 300+ |
| **Scaling** | Automatic |
| **Max Request Size** | 100 MB |
| **Durable Objects** | Millisecond persistence |
| **Persistence Strategy** | Smart batched (10-second windows) |

*Performance varies by region and workload. Write latency includes network roundtrip; actual DB operation is 2-3ms.*

## 📦 Project Structure

```text
QuartzDB/
├── quartz-faas/          # ⭐ Main Application (Cloudflare Workers)
│   ├── src/
│   │   ├── lib.rs        # Worker entry point
│   │   ├── api.rs        # REST API handlers
│   │   ├── durable.rs    # Durable Objects implementation
│   │   ├── monitoring.rs # Analytics tracking
│   │   └── error.rs      # Error handling
│   ├── wrangler.toml     # Cloudflare Workers config
│   ├── Cargo.toml        # Rust dependencies
│   └── README.md         # API documentation
│
├── docs/                 # Technical documentation
│   ├── HNSW_EXPLAINED.md
│   └── VECTOR_SEARCH_EXPLAINED.md
│
├── Cargo.toml            # Workspace configuration
├── README.md             # This file
├── LICENSE               # MIT license
├── DEPLOYMENT_STATUS.md  # Current deployment status
└── PRODUCTION_ROADMAP.md # Future plans
```
│
├── quartz-client/        # Client SDK (✅ Complete)

## 🔒 Security & Compliance

- **Data Persistence**: Durable Objects provide automatic persistence
- **Edge Security**: Cloudflare's global security infrastructure
- **HTTPS Only**: All traffic encrypted in transit
- **DDoS Protection**: Built-in Cloudflare protection

## 💰 Pricing

QuartzDB leverages Cloudflare Workers' pay-per-request model:

- **Workers Requests**: $0.50 per million requests
- **Durable Objects**: $0.15 per million requests + $0.20 per GB-month storage
- **Analytics Engine**: Included in Workers pricing

[Calculate your costs](https://developers.cloudflare.com/workers/platform/pricing/)

## 🤝 Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run `cargo test` and `wrangler build`
5. Submit a pull request

## 📄 License

MIT License - see [LICENSE](LICENSE) for details

## 🔗 Links

- **Documentation**: [quartz-faas/README.md](quartz-faas/README.md)
- **Deployment Status**: [DEPLOYMENT_STATUS.md](DEPLOYMENT_STATUS.md)
- **Roadmap**: [PRODUCTION_ROADMAP.md](PRODUCTION_ROADMAP.md)
- **Cloudflare Workers**: https://workers.cloudflare.com/
- **Durable Objects**: https://developers.cloudflare.com/durable-objects/

## 🚀 Next Steps

1. **[Read the deployment guide](DEPLOYMENT_STATUS.md)** - Learn about production deployment
2. **[Explore the API](quartz-faas/README.md)** - Complete API documentation
3. **[Deploy your own](https://workers.cloudflare.com/)** - Get started with Cloudflare Workers
4. **[Check the roadmap](PRODUCTION_ROADMAP.md)** - See what's coming next

---

Built with ❤️ using Rust and Cloudflare Workers
