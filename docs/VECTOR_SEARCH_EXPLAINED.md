# Vector Search in QuartzDB

**Last Updated:** June 2026  
**Audience:** Developers integrating QuartzDB, non-technical stakeholders

---

## What Is Vector Search?

Traditional databases search by exact field matches — title, ID, keyword.
Vector search instead understands *meaning*. It converts data into numeric representations (vectors) and finds items that are semantically similar.

**Traditional search:**

```
Query: "dog"
Results: Only rows where a field contains "dog"
```

**Vector search:**

```
Query: "dog"
Results: "dog", "puppy", "canine", "golden retriever"
Why? The vectors for these words are close in geometric space.
```

---

## How It Works

### Step 1: Turn Data Into Vectors (Embeddings)

AI embedding models (OpenAI, Cohere, Hugging Face, etc.) convert text or images into dense numeric arrays — typically 384 to 1536 floating-point numbers.

```
"cute dog"        → [0.2, 0.8, 0.1, 0.9, ...]   (384 floats)
"adorable puppy"  → [0.3, 0.7, 0.2, 0.8, ...]   (384 floats)
"angry cat"       → [0.9, 0.1, 0.8, 0.2, ...]   (384 floats)
```

Similar concepts produce vectors that are close together; unrelated concepts are far apart.

### Step 2: Store Vectors in QuartzDB

Each vector is stored with an ID and optional metadata:

```json
POST /api/vector/insert
{
  "id": "product_123",
  "vector": [0.2, 0.8, 0.1, 0.9, ...],
  "metadata": { "name": "Running shoes", "price": 79.99 }
}
```

### Step 3: Search by Similarity

Send a query vector and get back the closest matches:

```json
POST /api/vector/search
{
  "vector": [0.3, 0.7, 0.2, 0.8, ...],
  "k": 10
}
```

Response:

```json
{
  "results": [
    { "id": "product_123", "score": 0.95, "metadata": { "name": "Running shoes", "price": 79.99 } },
    { "id": "product_456", "score": 0.87, "metadata": { "name": "Trail sneakers", "price": 64.99 } }
  ]
}
```

---

## QuartzDB Architecture

QuartzDB runs on Cloudflare Workers (WASM) with Durable Objects for persistent state.

```text
┌──────────────────────────────────────────────┐
│              Your Application                │
│  (AI app, e-commerce, chatbot, etc.)         │
└──────────────────┬───────────────────────────┘
                   │  HTTPS (REST API)
                   ↓
┌──────────────────────────────────────────────┐
│        Cloudflare Edge (300+ locations)       │
│  ┌──────────────────────────────────────┐    │
│  │  Worker (WASM on V8 isolate)         │    │
│  │  - Router: /api/vector/*             │    │
│  │  - Auth: API key middleware          │    │
│  │  - Validation: request checks        │    │
│  │  - Shard routing: consistent hash    │    │
│  └──────────────┬───────────────────────┘    │
│                 │                             │
│   ┌─────────────┴────────────────┐           │
│   ↓                              ↓           │
│  ┌────────────────┐  ┌────────────────┐      │
│  │ VectorIndex DO │  │ VectorIndex DO │ ...  │
│  │ (HNSW graph)   │  │ (HNSW graph)   │      │
│  │  + SQLite      │  │  + SQLite      │      │
│  └────────────────┘  └────────────────┘      │
└──────────────────────────────────────────────┘
```

**Key components:**

- **Worker**: Stateless WASM process handling routing, authentication, validation, and shard selection.
- **Durable Objects**: Each shard stores an HNSW index in memory with SQLite-backed persistence. Strongly consistent and automatically replicated.
- **Shard Router**: Vectors are distributed across shards by consistent hashing on the vector ID. Search queries fan out to all shards and results are merged.

---

## API Reference

All vector endpoints live under `/api/vector/`. Authenticated requests require an `Authorization: Bearer qdb_...` header.

### Insert a Vector

```
POST /api/vector/insert
```

```json
{
  "id": "vec_001",
  "vector": [0.1, 0.2, ...],
  "metadata": { "label": "example" }
}
```

### Batch Insert

```
POST /api/vector/batch-insert
```

```json
{
  "vectors": [
    { "id": "vec_001", "vector": [0.1, 0.2, ...], "metadata": {} },
    { "id": "vec_002", "vector": [0.3, 0.4, ...], "metadata": {} }
  ]
}
```

### Search

```
POST /api/vector/search
```

```json
{
  "vector": [0.1, 0.2, ...],
  "k": 10
}
```

The `k` parameter controls how many results to return. Valid range: 1-100.

### Get by ID

```
GET /api/vector/get/:id
```

### Delete

```
DELETE /api/vector/delete/:id
```

---

## The HNSW Algorithm

QuartzDB uses **HNSW (Hierarchical Navigable Small World)** for approximate nearest-neighbor search.

Think of it like a highway system:

| Layer | Analogy | Purpose |
|-------|---------|---------|
| Top layers | Express highways | Skip most nodes, get close fast |
| Middle layers | Regional roads | Narrow the search area |
| Bottom layer | Local streets | Find the exact nearest neighbors |

Each layer is a proximity graph. Searches start at the top (sparse, long-range links) and descend through denser layers until reaching the bottom, where the final k nearest neighbors are identified.

**Why HNSW?**

- Sub-millisecond search over large datasets
- High recall (>95% accuracy at recommended settings)
- Memory-efficient layered structure
- No training phase — vectors are inserted incrementally

For a deep dive, see [HNSW_EXPLAINED.md](HNSW_EXPLAINED.md).

---

## Similarity Metric

QuartzDB uses **cosine similarity** to compare vectors.

$$\text{cosine\_similarity}(\mathbf{a}, \mathbf{b}) = \frac{\mathbf{a} \cdot \mathbf{b}}{|\mathbf{a}| \times |\mathbf{b}|}$$

- Returns a score from -1 to 1
- 1 = identical direction (most similar)
- 0 = orthogonal (unrelated)
- -1 = opposite

Cosine similarity is well-suited for text embeddings, where direction in vector space matters more than magnitude.

---

## Bringing Your Own Embeddings

QuartzDB stores and searches vectors — it does **not** generate embeddings. Use any embedding provider:

| Provider | Model | Dimensions | Notes |
|----------|-------|------------|-------|
| OpenAI | `text-embedding-3-small` | 1536 | Most popular |
| Cohere | `embed-english-v3.0` | 1024 | Multilingual support |
| Hugging Face | `all-MiniLM-L6-v2` | 384 | Free, runs locally |
| Google Vertex AI | Gecko | 768 | GCP integration |

QuartzDB currently supports **384-dimensional** vectors. Ensure your embedding model outputs 384 dimensions, or project/truncate to that size before inserting.

---

## Use Cases

### Semantic Product Search

Store product descriptions as vectors. Users search "comfortable shoes for running" and get results for sneakers, trainers, and running shoes — even without an exact keyword match.

### AI Chatbot Memory (RAG)

Store past conversations as vectors. When a user asks a new question, retrieve the most relevant prior messages for context, then pass them to an LLM.

### Content Recommendations

Store article/video embeddings. Given the current item's vector, find the most similar items to recommend.

### Duplicate Detection

Insert document embeddings and search with `k=1`. A high similarity score indicates a near-duplicate.

---

## Performance Characteristics

| Property | Value |
|----------|-------|
| Supported dimensions | 384 |
| Similarity metric | Cosine |
| Index algorithm | HNSW |
| Deployment | Cloudflare Workers (WASM) |
| Storage backend | Durable Objects (SQLite) |
| Latency | Sub-10ms from nearest edge location |
| Scaling | Automatic, shard-based |

---

## Further Reading

- [HNSW_EXPLAINED.md](HNSW_EXPLAINED.md) — Detailed walkthrough of the HNSW algorithm
- [USER_GUIDE.md](../USER_GUIDE.md) — Full API guide with authentication and billing
- [HNSW Paper (Malkov & Yashunin, 2016)](https://arxiv.org/abs/1603.09320)
- [OpenAI Embeddings Guide](https://platform.openai.com/docs/guides/embeddings)
