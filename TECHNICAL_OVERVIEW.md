# QuartzDB Technical Overview

## Architecture

QuartzDB is a high-performance distributed database built in Rust, designed for AI/ML workloads with integrated vector search capabilities.

### Core Components

```
QuartzDB
├── quartz-storage/     - LSM-tree storage engine with WAL
├── quartz-vector/      - HNSW vector search implementation
├── quartz-server/      - HTTP API server
├── quartz-client/      - Client SDK with connection pooling
├── quartz-core/        - Core types and transaction logic
├── quartz-network/     - Network layer (planned)
└── quartz-edge/        - Edge computing features (planned)
```

## Storage Engine

**Technology:** LSM-Tree (Log-Structured Merge-Tree)

**Features:**

- Write-Ahead Logging (WAL) for durability
- Multi-level compaction
- In-memory cache with LRU eviction
- ~2µs read latency (cache hits)
- ~3-6µs write latency

**Design Decisions:**

- Chose LSM-tree over B-tree for better write performance
- Separate WAL from data files for crash recovery
- Background compaction to maintain read performance
- Configurable cache size and compaction thresholds

## Vector Search

**Algorithm:** Hierarchical Navigable Small World (HNSW)

**Features:**

- Multiple distance metrics (Cosine, Euclidean, Dot Product)
- Named indexes for multi-tenancy
- Configurable M (max connections) and ef_construction parameters
- ~10ms search latency for 100k vectors

**Design Decisions:**

- HNSW chosen over IVF/FLAT for better search speed
- Named indexes stored in HashMap for O(1) lookup
- Distance metrics implemented as trait for extensibility
- Persistence via JSON serialization (TODO: optimize with binary format)

**Distance Calculation:**

- Cosine Similarity: `1.0 - distance` (HNSW returns distance, convert to similarity)
- Euclidean: Standard L2 distance
- Dot Product: Direct vector dot product

## HTTP API

**Framework:** Axum (async Rust web framework)

**Endpoints:**

- `GET /api/v1/health` - Health check
- `GET /api/v1/stats` - Server statistics
- `PUT /api/v1/kv/:key` - Set key-value
- `GET /api/v1/kv/:key` - Get value
- `DELETE /api/v1/kv/:key` - Delete key
- `POST /api/v1/vector/index` - Create vector index
- `POST /api/v1/vector/insert` - Insert vectors
- `POST /api/v1/vector/search` - Search vectors
- `GET /api/v1/vector/:index/get/:id` - Get vector by ID
- `DELETE /api/v1/vector/:index/delete/:id` - Delete vector

**Design Decisions:**

- REST-first API for simplicity
- JSON request/response for compatibility
- Health checks for production deployments
- Middleware: Logging, CORS, compression

## Deployment

### Docker

**Multi-stage build:**

1. Builder stage: Rust 1.89 Alpine with full build tools
2. Runtime stage: Minimal Alpine with only required libraries
3. Target size: <100MB

**Optimizations:**

- Dependency caching (build Cargo.toml first)
- Binary stripping for smaller size
- Non-root user for security
- Health check endpoint configured

### Cloud Platforms

**Supported:**

- DigitalOcean App Platform ($5-12/month)
- GitHub Codespaces (cloud development)
- Any Docker-compatible platform

**CI/CD:**

- GitHub Actions for automated builds
- Docker image published to GitHub Container Registry
- Automated testing on every push
- Security scanning with Trivy

## Development Environment

### Local Development

```bash
cargo build --release
cargo run -p quartz-server
cargo test
```

### GitHub Codespaces

- Pre-configured development container
- Docker-in-Docker support
- All tools pre-installed
- Free tier: 60 hours/month

### Requirements

- Rust 1.75+
- (Optional) Docker for containerized deployment

## Testing

**Test Coverage:**

- Unit tests for all core components
- Integration tests for storage engine
- API integration tests
- Vector search tests with real embeddings
- Benchmark suite for performance tracking

**Current Status:** 6/6 integration tests passing

## Performance Characteristics

**Storage:**

- Read latency: ~2µs (cache hit), ~500µs (disk)
- Write latency: ~3-6µs (with WAL)
- Throughput: 100k+ writes/sec (batch operations)

**Vector Search:**

- Search latency: ~10ms (100k vectors)
- Accuracy: >95% recall@10
- Memory: ~50MB per 100k vectors (768-dim embeddings)

## Future Roadmap

### Phase 1: Core Stability (Current)

- ✅ Storage engine with durability
- ✅ Vector search with HNSW
- ✅ HTTP API server
- ✅ Basic testing and documentation

### Phase 2: Production Ready (Next)

- Python client library
- Batch operations API
- Index statistics and monitoring
- Advanced vector filtering
- Binary persistence format

### Phase 3: Distribution

- Replication and consensus
- Automatic sharding
- Edge computing features
- Multi-region deployment

## Technical Decisions Log

### Storage Layer

- **Decision:** LSM-tree over B-tree
- **Rationale:** Better write performance for append-heavy workloads, suitable for vector embeddings
- **Trade-off:** More complex compaction, slightly slower reads

### Vector Search

- **Decision:** HNSW over IVF/FLAT
- **Rationale:** Better query latency and recall balance
- **Trade-off:** Higher memory usage, more complex implementation

### API Design

- **Decision:** REST over gRPC
- **Rationale:** Wider compatibility, easier client implementation
- **Trade-off:** Slightly higher overhead, no bidirectional streaming

### Deployment

- **Decision:** Docker-first deployment
- **Rationale:** Platform independence, easy scaling
- **Trade-off:** Small overhead vs native binaries

### Language Choice

- **Decision:** Rust
- **Rationale:** Memory safety, performance, excellent async support
- **Trade-off:** Steeper learning curve, longer compile times

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines.

## License

MIT License - See LICENSE file for details.
