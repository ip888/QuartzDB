# QuartzDB Vector Search Examples

This directory contains examples demonstrating QuartzDB's vector search capabilities.

## Prerequisites

All examples require the QuartzDB server to be running:

```bash
# From the project root directory
cargo run -p quartz-server
```

The server will start on `http://localhost:3000` by default.

## Examples

### 1. Simple Vector Demo (Recommended to start)

**File:** `simple_vector_demo.py`

A minimal example with no external dependencies (except `requests` which is usually pre-installed).

**What it demonstrates:**
- Initialize a 3D vector index
- Insert sample vectors with metadata
- Search for similar vectors
- Retrieve specific vectors
- Delete vectors

**Requirements:**
```bash
pip3 install requests
```

**Run:**
```bash
# Make sure the server is running first!
python3 quartz-server/examples/simple_vector_demo.py
```

**Expected output:**
```
================================================================================
QuartzDB Vector Search Demo - Simple Example
================================================================================
✅ Server is running

🔧 Initializing 3D vector index...
✅ Index initialized: 3D, metric=cosine

📝 Inserting sample vectors...
  ✓ Inserted vector 1: Concept A: Technology (pure x-axis)
  ✓ Inserted vector 2: Concept B: Science (pure y-axis)
  ...

🔍 Searching for vectors similar to: Technology-focused
   Query vector: [0.9, 0.1, 0.0]
   Retrieving top 3 results...

   📊 Results:
   ----------------------------------------------------------------------
   1. Similarity: 95.2%
      ID: 1
      Vector: [1.0, 0.0, 0.0]
      Info: Concept A: Technology (pure x-axis)
   ...
```

### 2. Semantic Search Demo (Advanced)

**File:** `semantic_search_demo.py`

A production-ready example using real text embeddings from Hugging Face sentence-transformers.

**What it demonstrates:**
- Generate embeddings from text using sentence-transformers
- Semantic similarity search
- Search for documents by meaning (not keywords)

**Requirements:**
```bash
pip3 install sentence-transformers requests
```

**⚠️ Note:** First run will download the model (~100MB), which may take a few minutes.

**Run:**
```bash
# Make sure the server is running first!
python3 quartz-server/examples/semantic_search_demo.py
```

**Expected output:**
```
================================================================================
QuartzDB Vector Search Demo - Semantic Search
================================================================================

🤖 Loading sentence-transformer model...
✅ Model loaded: all-MiniLM-L6-v2

🔧 Initializing vector index...
✅ Vector index initialized successfully

📝 Inserting 10 documents...
  ✓ Inserted document 1 (ID: 1)
  ✓ Inserted document 2 (ID: 2)
  ...

🔍 Searching for: "How do I search for similar vectors?"
   Retrieving top 3 results...

📊 Results:
--------------------------------------------------------------------------------

1. Similarity: 0.8234 (distance: 0.6468)
   ID: 3
   Text: Vector similarity search enables semantic search and recommendations
...
```

## Troubleshooting

### Server not running

```
❌ Error: Cannot connect to QuartzDB server
   Please start the server first:
   cargo run -p quartz-server
```

**Solution:** Start the server in a separate terminal.

### Missing dependencies

```
ModuleNotFoundError: No module named 'sentence_transformers'
```

**Solution:** Install the required packages:
```bash
pip3 install sentence_transformers requests
```

Or use the simpler example that doesn't require sentence-transformers:
```bash
python3 quartz-server/examples/simple_vector_demo.py
```

### Port already in use

```
Error: Address already in use (os error 48)
```

**Solution:** Either:
1. Stop the other process using port 3000
2. Or set a different port:
   ```bash
   QUARTZ_PORT=3001 cargo run -p quartz-server
   ```
   Then update `QUARTZ_URL` in the Python script to `http://localhost:3001`

### Index already initialized

```
Error: Vector index already exists
```

**Solution:** Restart the server to clear the index, or delete the data directory:
```bash
rm -rf ./data/quartz_server
cargo run -p quartz-server
```

## Using with Other Languages

The vector search API is language-agnostic. Here are examples in other languages:

### cURL

```bash
# Initialize index
curl -X POST http://localhost:3000/api/v1/vectors/init \
  -H "Content-Type: application/json" \
  -d '{"dimension": 3, "metric": "cosine"}'

# Insert vector
curl -X POST http://localhost:3000/api/v1/vectors \
  -H "Content-Type: application/json" \
  -d '{"vector": [1.0, 0.0, 0.0], "metadata": "test"}'

# Search
curl -X POST http://localhost:3000/api/v1/vectors/search \
  -H "Content-Type: application/json" \
  -d '{"vector": [0.9, 0.1, 0.0], "k": 5}'
```

### JavaScript/Node.js

```javascript
const axios = require('axios');

const QUARTZ_URL = 'http://localhost:3000';

// Initialize
await axios.post(`${QUARTZ_URL}/api/v1/vectors/init`, {
  dimension: 384,
  metric: 'cosine'
});

// Insert
const response = await axios.post(`${QUARTZ_URL}/api/v1/vectors`, {
  vector: embedding,
  metadata: 'document 1'
});
const vectorId = response.data.id;

// Search
const results = await axios.post(`${QUARTZ_URL}/api/v1/vectors/search`, {
  vector: queryEmbedding,
  k: 10
});
```

### Rust

```rust
use reqwest;
use serde_json::json;

let client = reqwest::Client::new();

// Initialize
client.post("http://localhost:3000/api/v1/vectors/init")
    .json(&json!({"dimension": 384, "metric": "cosine"}))
    .send()
    .await?;

// Insert
let response = client.post("http://localhost:3000/api/v1/vectors")
    .json(&json!({"vector": embedding, "metadata": "doc1"}))
    .send()
    .await?;

// Search
let results = client.post("http://localhost:3000/api/v1/vectors/search")
    .json(&json!({"vector": query_embedding, "k": 10}))
    .send()
    .await?;
```

## Next Steps

1. **Read the API documentation:** See `../VECTOR_SEARCH_API.md` for complete API reference
2. **Experiment:** Modify the examples to test different scenarios
3. **Integrate:** Use the API in your own application
4. **Scale up:** Try with real embeddings (OpenAI, Cohere, etc.)

## Resources

- **API Documentation:** `quartz-server/VECTOR_SEARCH_API.md`
- **HNSW Algorithm:** `docs/HNSW_EXPLAINED.md`
- **Integration Guide:** `docs/DAY3_VECTOR_API_SUMMARY.md`

## Common Use Cases

1. **Semantic Search:** Search documents by meaning
2. **Recommendation Systems:** Find similar products/users
3. **Image Similarity:** Find visually similar images
4. **Duplicate Detection:** Find near-duplicate content
5. **Clustering:** Group similar items together

---

**Need help?** Check the main documentation or open an issue on GitHub.
