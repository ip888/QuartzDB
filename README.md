# QuartzDB

> **AI-First Edge Database** - High-performance distributed database optimized for AI/ML workloads at the edge

[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com)

QuartzDB is a high-performance distributed edge database designed for modern cloud-native and AI applications. It provides automatic data locality optimization, built-in edge caching with smart replication, and specialized support for AI/ML workloads including vector search for embeddings.

## ✨ Key Features

### Currently Implemented ✅

- **Integrated Storage Engine**
  - LSM Tree with multi-level compaction
  - Write-Ahead Logging (WAL) for durability
  - High-performance cache with configurable size
  - Background compaction management
  - ~2µs read latency (cache hits)
  - ~3-6µs write latency

- **Client SDK**
  - Connection pooling with configurable limits
  - Automatic retry with exponential backoff
  - Query result caching
  - Performance metrics tracking
  - Thread-safe operations

- **Production Ready**
  - Comprehensive test suite (100+ tests)
  - Performance benchmarks
  - Example code and documentation
  - Rust 2024 edition

- **HTTP API Server**
  - RESTful endpoints for CRUD operations
  - Health checks and statistics
  - Request/response logging
  - CORS and compression middleware
  - JSON-based API
  - Comprehensive documentation with examples

### Coming Soon 🚀

- **Vector Search Module** (Week 4)
  - HNSW indexing for AI embeddings
  - Similarity search (cosine, euclidean, dot product)
  - Integration with OpenAI, Cohere, Hugging Face
  
- **gRPC API Server** (Week 3)
  - High-performance gRPC interface
  - Authentication & authorization
  - Streaming support
  
- **Edge Computing**
  - Automatic data locality
  - Multi-region consistency
  - Smart replication

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     Client Application                   │
└─────────────────┬───────────────────────────────────────┘
                  │
                  ↓
┌─────────────────────────────────────────────────────────┐
│                    QuartzDB Client SDK                   │
│  • Connection Pooling  • Retry Logic  • Metrics         │
└─────────────────┬───────────────────────────────────────┘
                  │
                  ↓
┌─────────────────────────────────────────────────────────┐
│                   Storage Engine (Core)                  │
│  ┌──────────┐  ┌───────────┐  ┌──────────┐             │
│  │  Cache   │  │  LSM Tree │  │   WAL    │             │
│  │ Manager  │  │ (Levels)  │  │  (Log)   │             │
│  └──────────┘  └───────────┘  └──────────┘             │
│                        │                                 │
│                        ↓                                 │
│                  ┌─────────────┐                        │
│                  │  RocksDB    │                        │
│                  │  (Backend)  │                        │
│                  └─────────────┘                        │
└─────────────────────────────────────────────────────────┘
```

### Storage Layer Components

1. **Cache Manager**: In-memory LRU cache for hot data
2. **LSM Tree**: Log-Structured Merge-tree for efficient writes
3. **Write-Ahead Log**: Ensures durability and crash recovery
4. **Compaction Manager**: Background task for level compaction
5. **RocksDB**: Proven storage backend

## 🚀 Quick Start

### Option 1: GitHub Codespaces (Recommended for M1 Mac)

**No local installation needed!** Develop in the cloud with full Docker support:

1. Click the green **Code** button on GitHub
2. Select **Codespaces** → **Create codespace on main**
3. Wait 2-3 minutes for setup
4. Start coding immediately!

**Free tier**: 60 hours/month on 2-core machine

[📖 Detailed Codespaces Guide](.devcontainer/README.md)

### Option 2: Local Development

#### Prerequisites

- Rust 1.75 or higher
- Cargo
- (Optional) Docker for containerized deployment

#### Installation

Add QuartzDB to your `Cargo.toml`:

```toml
[dependencies]
quartz-client = { path = "path/to/QuartzDB/quartz-client" }
quartz-storage = { path = "path/to/QuartzDB/quartz-storage" }
```

### Basic Usage

```rust
use quartz_storage::{StorageEngine, StorageConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create storage engine
    let storage = StorageEngine::new("./data/my_db")?;

    // Write data
    storage.put(b"user:1", b"Alice").await?;
    
    // Read data
    if let Some(value) = storage.get(b"user:1").await? {
        println!("Value: {}", String::from_utf8_lossy(&value));
    }
    
    // Delete data
    storage.delete(b"user:1").await?;

    Ok(())
}
```

### Custom Configuration

```rust
use quartz_storage::{StorageEngine, StorageConfig};

let config = StorageConfig {
    cache_size: 5000,           // 5000 entries
    compaction_threshold: 4,    // Compact at 4 levels
    max_level_size: 20,         // Max 20 SSTables per level
    enable_wal: true,           // Enable durability
};

let storage = StorageEngine::with_config("./data/custom_db", config)?;
```

### Background Compaction

```rust
// Start background compaction
storage.start_compaction().await;

// Your application logic here...

// Stop compaction on shutdown
storage.stop_compaction().await;
```

### HTTP API Server

QuartzDB includes a production-ready HTTP API server built with Axum:

```bash
# Start the server with defaults (port 3000)
cargo run -p quartz-server

# Custom configuration
QUARTZ_PORT=8080 QUARTZ_CACHE_SIZE=50000 cargo run -p quartz-server
```

**API Endpoints:**

```bash
# Health check
curl http://localhost:3000/api/v1/health

# Get storage statistics
curl http://localhost:3000/api/v1/stats

# Store a value
curl -X POST http://localhost:3000/api/v1/kv/user:1 \
  -H "Content-Type: application/json" \
  -d '{"value": "Alice"}'

# Retrieve a value
curl http://localhost:3000/api/v1/kv/user:1

# Delete a key
curl -X DELETE http://localhost:3000/api/v1/kv/user:1
```

**Try the example client:**

```bash
cargo run -p quartz-server --example simple_client
```

See [`quartz-server/API.md`](quartz-server/API.md) for complete API documentation.

## 📊 Performance

Based on our benchmarks (M1 Mac, SSD):

| Operation | Latency | Throughput |
|-----------|---------|------------|
| **Cache Hit Read** | ~2µs | 500K ops/sec |
| **Cache Miss Read** | ~2µs | 490K ops/sec |
| **Write (WAL disabled)** | ~3µs | 340K ops/sec |
| **Write (WAL enabled)** | ~6µs | 160K ops/sec |
| **Delete** | ~5µs | 200K ops/sec |
| **Concurrent Writes (8)** | ~50µs | 160K ops/sec |

Run your own benchmarks:

```bash
cargo bench -p quartz-storage
```

## 📦 Project Structure

```text
QuartzDB/
├── quartz-core/          # Core database types and logic
│   ├── src/
│   │   ├── types.rs      # Core data types
│   │   ├── query.rs      # Query engine
│   │   └── transaction.rs # Transaction support
│   └── tests/
│
├── quartz-storage/       # Storage engine (✅ Complete)
│   ├── src/
│   │   ├── engine.rs     # Integrated storage engine
│   │   ├── lsm.rs        # LSM tree implementation
│   │   ├── wal.rs        # Write-ahead log
│   │   ├── cache.rs      # Cache manager
│   │   └── compaction.rs # Compaction manager
│   ├── tests/            # Integration tests
│   ├── benches/          # Performance benchmarks
│   └── examples/         # Usage examples
│
├── quartz-network/       # Networking layer
│   └── src/lib.rs
│
├── quartz-edge/          # Edge computing components
│   └── src/lib.rs
│
├── quartz-client/        # Client SDK (✅ Complete)
│   ├── src/
│   │   ├── lib.rs        # Client implementation
│   │   └── metrics.rs    # Performance metrics
│   └── tests/
│
├── quartz-server/        # HTTP API Server (✅ Complete)
│   ├── src/
│   │   ├── lib.rs        # API handlers and router
│   │   └── main.rs       # Server binary
│   ├── tests/            # API integration tests (13)
│   ├── examples/         # Client usage examples
│   └── API.md            # Complete API documentation
│
└── docs/                 # Documentation
    ├── VECTOR_SEARCH_EXPLAINED.md
    └── PRODUCT_STRATEGY.md
```

## 🧪 Testing

### Run All Tests

```bash
cargo test
```

### Run Storage Tests Only

```bash
cargo test -p quartz-storage
```

### Run with Output

```bash
cargo test -- --nocapture
```

### Run Examples

```bash
# Storage demo
cargo run -p quartz-storage --example storage_demo

# HTTP API client example
cargo run -p quartz-server --example simple_client
```

### Run HTTP API Tests

```bash
# Run all server tests
cargo test -p quartz-server

# Test specific endpoint
cargo test -p quartz-server test_put_and_get
```

## 🔧 Building

### Development Build

```bash
cargo build
```

### Release Build (Optimized)

```bash
cargo build --release
```

### Check Code Quality

```bash
# Run clippy for lints
cargo clippy

# Format code
cargo fmt

# Check formatting
cargo fmt -- --check
```

## 📈 Benchmarking

The project includes comprehensive benchmarks:

```bash
# Run all benchmarks
cargo bench -p quartz-storage

# Run specific benchmark
cargo bench -p quartz-storage -- write_operations

# Quick benchmark (faster)
cargo bench -p quartz-storage -- --quick
```

Benchmark categories:

- **Write operations**: Single writes with various cache sizes
- **Batch writes**: Bulk write performance (10, 100, 1000 entries)
- **Read operations**: Cache hits vs misses
- **Delete operations**: Deletion performance
- **WAL comparison**: WAL enabled vs disabled
- **Concurrent operations**: Multi-threaded performance
- **Mixed workload**: 70/30 read/write ratio

## 🗺️ Roadmap

### Phase 1: Core Storage ✅ (Completed)

- [x] LSM tree implementation
- [x] Write-ahead logging
- [x] Cache management
- [x] Background compaction
- [x] Integration tests
- [x] Performance benchmarks

### Phase 2: API Server (Weeks 2-3)

- [ ] HTTP REST API
- [ ] gRPC interface
- [ ] Authentication/Authorization
- [ ] API documentation
- [ ] Rate limiting

### Phase 3: Vector Search (Week 4)

- [ ] Vector storage format
- [ ] HNSW index implementation
- [ ] Similarity algorithms
- [ ] Vector API endpoints
- [ ] Integration with AI platforms

### Phase 4: Edge Computing (Weeks 5-8)

- [ ] Multi-region support
- [ ] Data locality optimization
- [ ] Smart replication
- [ ] Conflict resolution
- [ ] Edge node management

### Phase 5: Production Ready (Weeks 9-12)

- [ ] Monitoring & observability
- [ ] Distributed tracing
- [ ] Backup & recovery
- [ ] Performance tuning
- [ ] Load testing

## 🤝 Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for details on our development process.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🔗 Resources

- [Product Strategy](PRODUCT_STRATEGY.md) - Market analysis and monetization plan
- [Vector Search Explained](docs/VECTOR_SEARCH_EXPLAINED.md) - AI/ML integration guide
- [Rust Book](https://doc.rust-lang.org/book/) - Learn Rust
- [RocksDB](https://rocksdb.org/) - Storage backend

## 🌟 Why QuartzDB?

### vs. Traditional Databases

- **Edge-First**: Optimized for low latency at the edge
- **AI-Ready**: Native vector search for embeddings
- **Modern**: Built with Rust for safety and performance

### vs. Cloud Databases

- **Local-First**: No network latency for edge workloads
- **Cost-Effective**: Reduce cloud data transfer costs
- **Privacy**: Keep sensitive data at the edge

### vs. Other Vector DBs

- **Integrated**: Full database + vector search
- **Edge Support**: Run anywhere, not just cloud
- **Open Source**: MIT licensed, full control

---

**Built with ❤️ and Rust 🦀**
