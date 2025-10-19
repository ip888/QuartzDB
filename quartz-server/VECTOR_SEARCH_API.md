# QuartzDB Vector Search API

Complete API reference for QuartzDB's vector similarity search capabilities with **Named Indexes** support.

## Overview

QuartzDB provides a high-performance vector search API powered by the HNSW (Hierarchical Navigable Small World) algorithm. This enables efficient similarity search for machine learning embeddings, semantic search, recommendation systems, and more.

### Key Features

- **Named Indexes**: Support multiple independent vector indexes with different dimensions and metrics
- **Multiple Distance Metrics**: Cosine similarity, Euclidean distance, Dot product
- **HNSW Index**: O(log n) search complexity with high recall
- **Persistent Storage**: Vectors automatically saved to disk per index
- **Metadata Support**: Attach custom metadata to vectors
- **RESTful API**: Simple HTTP interface for all operations

## Named Indexes

QuartzDB supports multiple named vector indexes, allowing you to:
- Use different dimensions for different use cases (e.g., `products_384d`, `documents_768d`)
- Apply different distance metrics per index (e.g., cosine for semantic, euclidean for images)
- Isolate vector collections by purpose or tenant

Each index has its own:
- Storage directory: `./data/server/indexes/{name}/`
- Vector ID sequence (starting from 1)
- HNSW graph and metadata

## Quick Start

### 1. List Available Indexes

Check what indexes exist:

```bash
curl http://localhost:3000/api/v1/indexes
```

**Response:**
```json
{
  "indexes": [
    {
      "name": "products_384d",
      "dimension": 384,
      "metric": "cosine",
      "num_vectors": 1250
    },
    {
      "name": "images_512d",
      "dimension": 512,
      "metric": "euclidean",
      "num_vectors": 8430
    }
  ]
}
```

### 2. Create/Initialize an Index

Before inserting vectors, create a named index with dimension and distance metric:

```bash
curl -X POST http://localhost:3000/api/v1/indexes/products_384d \
  -H "Content-Type: application/json" \
  -d '{
    "dimension": 384,
    "metric": "cosine"
  }'
```

**Response:**
```json
{
  "message": "Vector index 'products_384d' initialized successfully",
  "dimension": 384,
  "metric": "cosine"
}
```

### 3. Insert Vectors

Insert a vector with optional metadata into a specific index:

```bash
curl -X POST http://localhost:3000/api/v1/indexes/products_384d/vectors \
  -H "Content-Type: application/json" \
  -d '{
    "vector": [0.1, 0.2, 0.3, ...],
    "metadata": "Wireless Bluetooth Headphones - Premium Sound"
  }'
```

**Response:**
```json
{
  "id": 1,
  "message": "Vector inserted successfully"
}
```

### 3. Search for Similar Vectors

Find the k nearest neighbors:

```bash
curl -X POST http://localhost:3000/api/v1/indexes/{name}/vectors/search \
  -H "Content-Type: application/json" \
  -d '{
    "vector": [0.15, 0.18, 0.32, ...],
    "k": 5
  }'
```

**Response:**
```json
{
  "results": [
    {
      "id": 1,
      "distance": 0.02,
      "vector": [0.1, 0.2, 0.3, ...],
      "metadata": "user query: how to use QuartzDB"
    },
    ...
  ]
}
```

## API Reference

### List Indexes

**GET** `/api/v1/indexes`

List all available vector indexes with their configurations and statistics.

#### Example Request

```bash
curl http://localhost:3000/api/v1/indexes
```

#### Example Response (200 OK)

```json
{
  "indexes": [
    {
      "name": "products_384d",
      "dimension": 384,
      "metric": "cosine",
      "num_vectors": 1250
    },
    {
      "name": "documents_768d",
      "dimension": 768,
      "metric": "cosine",
      "num_vectors": 5430
    }
  ]
}
```

---

### Create/Initialize Index

**POST** `/api/v1/indexes/{name}`

Create and initialize a new named vector index with specified configuration.

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Unique name for the index (e.g., "products_384d", "images_512d") |

#### Request Body

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `dimension` | integer | Yes | Vector dimension (e.g., 384, 768, 1536) |
| `metric` | string | No | Distance metric: `"cosine"`, `"euclidean"`, `"dotproduct"` (default: `"cosine"`) |
| `m` | integer | No | HNSW M parameter (reserved for future use) |
| `ef_construction` | integer | No | HNSW ef_construction parameter (reserved for future use) |

#### Example Request

```json
{
  "dimension": 768,
  "metric": "euclidean"
}
```

#### Example Response (200 OK)

```json
{
  "message": "Vector index 'documents_768d' initialized successfully",
  "dimension": 768,
  "metric": "euclidean"
}
```

#### Error Responses

- **400 Bad Request**: Invalid distance metric
  ```json
  {
    "error": "bad_request",
    "message": "Invalid distance metric: unknown. Must be one of: cosine, euclidean, dotproduct"
  }
  ```

- **400 Bad Request**: Index already exists with different configuration
  ```json
  {
    "error": "bad_request",
    "message": "Index 'documents_768d' already exists with dimension=384, metric=cosine"
  }
  ```

---

### Delete Index

**DELETE** `/api/v1/indexes/{name}`

Delete a vector index and all its vectors.

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Name of the index to delete |

#### Example Request

```bash
curl -X DELETE http://localhost:3000/api/v1/indexes/old_index
```

#### Example Response (200 OK)

```json
{
  "message": "Index 'old_index' deleted successfully"
}
```

#### Error Responses

- **404 Not Found**: Index does not exist
  ```json
  {
    "error": "index_not_found",
    "message": "Index 'old_index' not found"
  }
  ```

---

### Insert Vector

**POST** `/api/v1/indexes/{name}/vectors`

Insert a new vector into the specified index.

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Name of the index to insert into |

#### Request Body

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `vector` | array[float] | Yes | Vector data (must match index dimension) |
| `metadata` | string | No | Optional metadata (e.g., original text, document ID) |

#### Example Request

```json
{
  "vector": [0.123, 0.456, 0.789, ...],
  "metadata": "Product: Wireless Headphones - Best noise cancellation"
}
```

#### Example Response (200 OK)

```json
{
  "id": 42,
  "message": "Vector inserted successfully"
}
```

#### Error Responses

- **503 Service Unavailable**: Index not initialized
  ```json
  {
    "error": "vector_index_not_initialized",
    "message": "Vector index has not been initialized. Please initialize it first."
  }
  ```

- **400 Bad Request**: Dimension mismatch
  ```json
  {
    "error": "bad_request",
    "message": "Vector dimension mismatch: expected 384, got 768"
  }
  ```

---

### Search Vectors

**POST** `/api/v1/indexes/{name}/vectors/search`

Search for k nearest neighbors to a query vector in the specified index.

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Name of the index to search |

#### Request Body

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `vector` | array[float] | Yes | Query vector (must match index dimension) |
| `k` | integer | No | Number of results to return (default: 10) |

#### Example Request

```json
{
  "vector": [0.111, 0.222, 0.333, ...],
  "k": 5
}
```

#### Example Response (200 OK)

```json
{
  "results": [
    {
      "id": 42,
      "distance": 0.05,
      "vector": [0.123, 0.456, 0.789, ...],
      "metadata": "Product: Wireless Headphones"
    },
    {
      "id": 17,
      "distance": 0.12,
      "vector": [0.111, 0.444, 0.777, ...],
      "metadata": "Product: Bluetooth Speaker"
    },
    ...
  ]
}
```

#### Distance Interpretation

| Metric | Range | Interpretation |
|--------|-------|----------------|
| **Cosine** | -1 to 1 | Higher is more similar (1 = identical, 0 = orthogonal, -1 = opposite) |
| **Euclidean** | 0 to ∞ | Lower is more similar (0 = identical) |
| **Dot Product** | -∞ to ∞ | Higher is more similar (magnitude-dependent) |

#### Error Responses

- **503 Service Unavailable**: Index not initialized
- **400 Bad Request**: Dimension mismatch

---

### Retrieve Vector

**GET** `/api/v1/indexes/{name}/vectors/{id}`

Retrieve a specific vector by ID from the specified index.

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Name of the index |
| `id` | integer | Yes | Vector ID |

#### Example Request

```bash
curl http://localhost:3000/api/v1/indexes/products_384d/vectors/42
```

#### Example Response (200 OK)

```json
{
  "id": 42,
  "vector": [0.123, 0.456, 0.789, ...],
  "metadata": null
}
```

#### Error Responses

- **503 Service Unavailable**: Index not initialized
- **404 Not Found**: Vector does not exist
  ```json
  {
    "error": "vector_not_found",
    "message": "Vector with id 42 not found"
  }
  ```

---

### Delete Vector

**DELETE** `/api/v1/indexes/{name}/vectors/{id}`

Delete a vector from the specified index.

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Name of the index |
| `id` | integer | Yes | Vector ID |

#### Example Request

```bash
curl -X DELETE http://localhost:3000/api/v1/indexes/products_384d/vectors/42
```

#### Example Response (200 OK)

```json
{
  "id": 42,
  "message": "Vector deleted successfully"
}
```

#### Error Responses

- **503 Service Unavailable**: Index not initialized
- **404 Not Found**: Vector does not exist

---

## Distance Metrics

### Cosine Similarity

**Best for:** Normalized vectors, semantic similarity, text embeddings

**Formula:** `similarity = (A · B) / (||A|| × ||B||)`

**Use cases:**
- Text embeddings (BERT, OpenAI, Cohere)
- Sentence similarity
- Document clustering
- Recommendation systems (user/item embeddings)

**Example:**
```json
{
  "dimension": 384,
  "metric": "cosine"
}
```

### Euclidean Distance

**Best for:** Spatial data, image features, unnormalized vectors

**Formula:** `distance = √(Σ(Ai - Bi)²)`

**Use cases:**
- Image similarity (CNN features)
- Geographic coordinates
- Time series data
- Physical measurements

**Example:**
```json
{
  "dimension": 2048,
  "metric": "euclidean"
}
```

### Dot Product

**Best for:** Pre-normalized vectors, faster approximate cosine

**Formula:** `score = Σ(Ai × Bi)`

**Use cases:**
- Already-normalized embeddings
- Maximum inner product search (MIPS)
- Fast approximate cosine similarity

**Example:**
```json
{
  "dimension": 768,
  "metric": "dotproduct"
}
```

---

## Integration Examples

### OpenAI Embeddings

```python
import openai
import requests

# Generate embedding
response = openai.Embedding.create(
    input="How do I use QuartzDB?",
    model="text-embedding-3-small"  # 1536 dimensions
)
embedding = response['data'][0]['embedding']

# Insert into QuartzDB
requests.post('http://localhost:3000/api/v1/vectors', json={
    'vector': embedding,
    'metadata': 'How do I use QuartzDB?'
})
```

### Cohere Embeddings

```python
import cohere
import requests

co = cohere.Client('your-api-key')

# Generate embedding
response = co.embed(
    texts=["QuartzDB vector search guide"],
    model="embed-english-v3.0"  # 1024 dimensions
)
embedding = response.embeddings[0]

# Insert into QuartzDB
requests.post('http://localhost:3000/api/v1/vectors', json={
    'vector': embedding,
    'metadata': 'QuartzDB vector search guide'
})
```

### Hugging Face Sentence Transformers

```python
from sentence_transformers import SentenceTransformer
import requests

model = SentenceTransformer('all-MiniLM-L6-v2')  # 384 dimensions

# Generate embedding
embedding = model.encode("Semantic search with QuartzDB").tolist()

# Insert into QuartzDB
requests.post('http://localhost:3000/api/v1/vectors', json={
    'vector': embedding,
    'metadata': 'Semantic search with QuartzDB'
})
```

---

## Use Cases

### 1. Semantic Search

Build a documentation search engine:

```bash
# Initialize with sentence-transformer dimensions
POST /api/v1/indexes/{name}
{
  "dimension": 384,
  "metric": "cosine"
}

# Insert documentation embeddings
POST /api/v1/vectors
{
  "vector": [...],
  "metadata": "Installation Guide - QuartzDB Setup"
}

# Search for relevant docs
POST /api/v1/indexes/{name}/vectors/search
{
  "vector": [query_embedding],
  "k": 5
}
```

### 2. Product Recommendations

Find similar products:

```bash
# Initialize with product feature dimensions
POST /api/v1/indexes/{name}
{
  "dimension": 128,
  "metric": "euclidean"
}

# Insert product vectors
POST /api/v1/vectors
{
  "vector": [...],
  "metadata": "product_id:12345,category:electronics"
}

# Find similar products
POST /api/v1/indexes/{name}/vectors/search
{
  "vector": [current_product_embedding],
  "k": 10
}
```

### 3. Image Similarity

Find visually similar images:

```bash
# Initialize with CNN feature dimensions
POST /api/v1/indexes/{name}
{
  "dimension": 2048,
  "metric": "cosine"
}

# Insert image features
POST /api/v1/vectors
{
  "vector": [...],
  "metadata": "image_url:https://example.com/img1.jpg"
}

# Search for similar images
POST /api/v1/indexes/{name}/vectors/search
{
  "vector": [query_image_features],
  "k": 20
}
```

---

## Performance Characteristics

### HNSW Index

- **Search Time Complexity:** O(log n)
- **Insert Time Complexity:** O(log n)
- **Memory Usage:** ~40-60 bytes per vector per connection
- **Recall:** 95-99% with default settings

### Benchmarks

| Operation | 1K vectors | 10K vectors | 100K vectors |
|-----------|------------|-------------|--------------|
| **Search (k=10)** | <1ms | ~2ms | ~5ms |
| **Insert** | ~1μs | ~2μs | ~3μs |
| **Memory** | ~500KB | ~5MB | ~50MB |

*Measured on MacBook Pro M1, 384-dimensional vectors, cosine metric*

### Scalability Recommendations

| Dataset Size | Configuration | Expected Performance |
|--------------|---------------|----------------------|
| < 10K | Default (balanced) | <2ms search latency |
| 10K - 100K | Default (balanced) | 2-10ms search latency |
| 100K - 1M | High quality preset | 10-50ms search latency |
| > 1M | Consider sharding | Varies by shard size |

---

## Best Practices

### 1. Vector Normalization

For cosine similarity, normalize vectors before insertion for consistent results:

```python
import numpy as np

def normalize(vector):
    norm = np.linalg.norm(vector)
    return (vector / norm).tolist() if norm > 0 else vector

# Use normalized vector
normalized = normalize(embedding)
requests.post('/api/v1/vectors', json={'vector': normalized})
```

### 2. Batch Operations

For bulk inserts, insert vectors sequentially (parallel support coming soon):

```python
for doc in documents:
    embedding = embed(doc['text'])
    requests.post('/api/v1/vectors', json={
        'vector': embedding,
        'metadata': doc['id']
    })
```

### 3. Error Handling

Always check for initialization before operations:

```python
try:
    response = requests.post('/api/v1/indexes/{name}/vectors/search', json={
        'vector': query_vector,
        'k': 10
    })
    response.raise_for_status()
    results = response.json()['results']
except requests.HTTPError as e:
    if e.response.status_code == 503:
        print("Error: Vector index not initialized")
        # Initialize index first
```

### 4. Metadata Design

Use structured metadata for filtering (coming soon):

```json
{
  "vector": [...],
  "metadata": "{\"doc_id\": \"123\", \"category\": \"tech\", \"date\": \"2024-01-01\"}"
}
```

---

## Limitations

- **Single Index:** Currently supports one vector index per server instance
- **No Filtering:** Metadata filtering not yet supported (search returns all matches)
- **No Batch Insert:** Must insert vectors one at a time
- **Fixed Dimension:** Cannot change dimension after initialization
- **In-Memory Search:** HNSW graph kept in memory for fast search

---

## Error Reference

| Status Code | Error Type | Description |
|-------------|------------|-------------|
| 200 | Success | Operation completed successfully |
| 400 | `bad_request` | Invalid input (wrong dimension, invalid metric) |
| 404 | `not_found` / `vector_not_found` | Resource does not exist |
| 500 | `vector_error` / `storage_error` | Internal error |
| 503 | `vector_index_not_initialized` | Index must be initialized first |

---

## Roadmap

### Coming Soon

- [ ] Filtered search (metadata-based filtering)
- [ ] Batch insert API
- [ ] Multiple indices support
- [ ] Index statistics endpoint
- [ ] HNSW parameter tuning API
- [ ] Incremental index rebuilding

### Future

- [ ] GPU-accelerated search
- [ ] Distributed vector index
- [ ] Approximate filtering
- [ ] Vector compression
- [ ] Multi-vector queries

---

## Support

For questions, issues, or feature requests:
- GitHub Issues: [QuartzDB Repository](https://github.com/your-org/quartzdb)
- Documentation: See `docs/HNSW_EXPLAINED.md` for algorithm details
- Examples: See `quartz-server/examples/` for code samples

---

**Version:** 0.1.0  
**Last Updated:** October 2025
