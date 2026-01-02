# QuartzDB FaaS - Cloudflare Workers Integration

Serverless edge API for QuartzDB running on Cloudflare Workers.

## Features

- ✅ Key-Value storage API
- ✅ Vector search API  
- ✅ Durable Objects integration
- ✅ Persistent state management
- ✅ Edge caching via Durable Objects

## API Endpoints

### Health Check
```
GET /health
```

### Key-Value Operations
```
POST /api/put
{
  "key": "mykey",
  "value": "myvalue"
}

GET /api/get/:key

DELETE /api/delete/:key
```

### Vector Operations
```
POST /api/vector/insert
{
  "id": 123,
  "vector": [0.1, 0.2, 0.3, ...],
  "metadata": { "label": "example" }
}

POST /api/vector/search
{
  "query": [0.1, 0.2, 0.3, ...],
  "k": 10,
  "metric": "cosine"
}
```

## Development

### Prerequisites
- Rust 1.89+
- wrangler CLI: `npm install -g wrangler`
- worker-build: `cargo install worker-build`

### Build
```bash
cargo build --target wasm32-unknown-unknown --release
```

### Deploy
```bash
wrangler deploy
```

### Test Locally
```bash
wrangler dev
```

## Architecture

The FaaS layer provides:
1. **HTTP API** - REST endpoints for storage and vector operations
2. **Durable Objects** - Strongly consistent, low-latency state management
   - `StorageObject` - Key-value data with memory + persistent storage
   - `VectorIndexObject` - Vector embeddings with cosine similarity search
3. **Edge Computing** - Global deployment for <50ms latency
4. **Auto-scaling** - Serverless infrastructure scales automatically

## Performance

- Cold start: <10ms
- Hot path latency: <5ms
- Global edge network: 300+ locations
- Automatic DDoS protection

## Pricing

Cloudflare Workers Free Tier:
- 100,000 requests/day
- 10ms CPU time per request
- Perfect for MVP and testing

## Status

🚧 **Week 1 - Day 2** Complete!

Completed:
1. ✅ Basic API scaffold
2. ✅ Durable Objects integration

Next steps:
3. ⏳ Production deployment
4. ⏳ Monitoring and analytics
5. ⏳ Authentication & API keys
