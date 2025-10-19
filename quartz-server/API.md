# QuartzDB HTTP API Documentation

**Version:** 0.1.0  
**Base URL:** `http://localhost:3000`  
**Content-Type:** `application/json`

---

## 🚀 Quick Start

### Starting the Server

```bash
# With default settings
cargo run -p quartz-server

# With custom configuration
QUARTZ_HOST=0.0.0.0 \
QUARTZ_PORT=8080 \
QUARTZ_DATA_PATH=./data/my_db \
QUARTZ_CACHE_SIZE=50000 \
cargo run -p quartz-server
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `QUARTZ_HOST` | Server bind address | `0.0.0.0` |
| `QUARTZ_PORT` | Server port | `3000` |
| `QUARTZ_DATA_PATH` | Storage directory | `./data/quartz_server` |
| `QUARTZ_CACHE_SIZE` | Cache size (entries) | `10000` |

---

## 📚 API Endpoints

### Health Check

#### GET /api/v1/health

Check if the server is running and healthy.

#### Request

```bash
curl http://localhost:3000/api/v1/health
```

#### Health Response (200 OK)

```json
{
  "status": "healthy",
  "version": "0.1.0"
}
```

---

### Storage Statistics

#### GET /api/v1/stats

Get current storage engine statistics.

#### Stats Request

```bash
curl http://localhost:3000/api/v1/stats
```

#### Stats Response (200 OK)

```json
{
  "lsm_levels": 1,
  "cache_size": 10000,
  "wal_enabled": true
}
```

**Response Fields:**

- `lsm_levels`: Number of LSM tree levels
- `cache_size`: Maximum cache entries
- `wal_enabled`: Whether Write-Ahead Log is enabled

---

### Store Key-Value Pair

#### POST /api/v1/kv/{key}

Store a value for the given key.

#### Store Request

```bash
curl -X POST http://localhost:3000/api/v1/kv/user:123 \
  -H "Content-Type: application/json" \
  -d '{"value": "Alice Smith"}'
```

#### Request Body

```json
{
  "value": "string (required)"
}
```

#### Response (200 OK)

```json
{
  "key": "user:123",
  "message": "Value stored successfully"
}
```

#### Error Response (400 Bad Request)

```json
{
  "error": "bad_request",
  "message": "Key cannot be empty"
}
```

---

### Retrieve Value

#### GET /api/v1/kv/{key}

Get the value for a given key.

#### Retrieve Request

```bash
curl http://localhost:3000/api/v1/kv/user:123
```

#### Response (200 OK) response

```json
{
  "key": "user:123",
  "value": "Alice Smith"
}
```

#### Error Response (404 Not Found)

```json
{
  "error": "not_found",
  "message": "Key 'user:123' not found"
}
```

---

### Delete Key-Value Pair

#### DELETE /api/v1/kv/{key}

Delete a key and its value.

#### Delete Request

```bash
curl -X DELETE http://localhost:3000/api/v1/kv/user:123
```

#### Response (200 OK) delete

```json
{
  "key": "user:123",
  "message": "Key deleted successfully"
}
```

#### Error Response (404 Not Found) delete

```json
{
  "error": "not_found",
  "message": "Key 'user:123' not found"
}
```

---

## 🔍 Complete Examples

### Example 1: Basic CRUD Operations

```bash
# 1. Store a value
curl -X POST http://localhost:3000/api/v1/kv/product:1 \
  -H "Content-Type: application/json" \
  -d '{"value": "Laptop - $999"}'

# Response:
# {
#   "key": "product:1",
#   "message": "Value stored successfully"
# }

# 2. Retrieve the value
curl http://localhost:3000/api/v1/kv/product:1

# Response:
# {
#   "key": "product:1",
#   "value": "Laptop - $999"
# }

# 3. Update the value
curl -X POST http://localhost:3000/api/v1/kv/product:1 \
  -H "Content-Type: application/json" \
  -d '{"value": "Laptop - $899 (On Sale!)"}'

# 4. Verify the update
curl http://localhost:3000/api/v1/kv/product:1

# Response:
# {
#   "key": "product:1",
#   "value": "Laptop - $899 (On Sale!)"
# }

# 5. Delete the value
curl -X DELETE http://localhost:3000/api/v1/kv/product:1

# Response:
# {
#   "key": "product:1",
#   "message": "Key deleted successfully"
# }

# 6. Verify deletion
curl http://localhost:3000/api/v1/kv/product:1

# Response (404):
# {
#   "error": "not_found",
#   "message": "Key 'product:1' not found"
# }
```

### Example 2: Storing JSON Data

```bash
# Store complex JSON as a string
curl -X POST http://localhost:3000/api/v1/kv/user:alice \
  -H "Content-Type: application/json" \
  -d '{
    "value": "{\"name\":\"Alice\",\"email\":\"alice@example.com\",\"age\":30}"
  }'

# Retrieve it
curl http://localhost:3000/api/v1/kv/user:alice

# You'll get the JSON string back, which you can parse
```

### Example 3: Batch Operations

```bash
#!/bin/bash

# Store multiple products
for i in {1..10}; do
  curl -X POST http://localhost:3000/api/v1/kv/product:$i \
    -H "Content-Type: application/json" \
    -d "{\"value\": \"Product $i\"}" \
    --silent
  echo ""
done

# Retrieve all products
for i in {1..10}; do
  curl http://localhost:3000/api/v1/kv/product:$i --silent | jq .
done
```

### Example 4: Using with jq for Pretty Output

```bash
# Store and immediately view with jq
curl -X POST http://localhost:3000/api/v1/kv/session:abc123 \
  -H "Content-Type: application/json" \
  -d '{"value": "user_data_here"}' | jq .

# Get stats with formatting
curl http://localhost:3000/api/v1/stats | jq '.'
```

### Example 5: Error Handling

```bash
# Try to get a non-existent key
curl -i http://localhost:3000/api/v1/kv/nonexistent

# Response:
# HTTP/1.1 404 Not Found
# {
#   "error": "not_found",
#   "message": "Key 'nonexistent' not found"
# }

# Try to delete a non-existent key
curl -i -X DELETE http://localhost:3000/api/v1/kv/nonexistent

# Response:
# HTTP/1.1 404 Not Found
# {
#   "error": "not_found",
#   "message": "Key 'nonexistent' not found"
# }
```

---

## 🔧 Using with Programming Languages

### Python (requests)

```python
import requests
import json

BASE_URL = "http://localhost:3000/api/v1"

# Store a value
response = requests.post(
    f"{BASE_URL}/kv/user:1",
    json={"value": "Alice"}
)
print(response.json())

# Get a value
response = requests.get(f"{BASE_URL}/kv/user:1")
data = response.json()
print(f"Key: {data['key']}, Value: {data['value']}")

# Delete a value
response = requests.delete(f"{BASE_URL}/kv/user:1")
print(response.json())

# Get stats
response = requests.get(f"{BASE_URL}/stats")
stats = response.json()
print(f"Cache size: {stats['cache_size']}")
print(f"WAL enabled: {stats['wal_enabled']}")
```

### JavaScript/Node.js (fetch)

```javascript
const BASE_URL = 'http://localhost:3000/api/v1';

// Store a value
async function putValue(key, value) {
  const response = await fetch(`${BASE_URL}/kv/${key}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ value })
  });
  return response.json();
}

// Get a value
async function getValue(key) {
  const response = await fetch(`${BASE_URL}/kv/${key}`);
  if (response.ok) {
    return response.json();
  } else {
    throw new Error(`Key not found: ${key}`);
  }
}

// Delete a value
async function deleteValue(key) {
  const response = await fetch(`${BASE_URL}/kv/${key}`, {
    method: 'DELETE'
  });
  return response.json();
}

// Usage
(async () => {
  await putValue('user:1', 'Alice');
  const data = await getValue('user:1');
  console.log(data); // { key: 'user:1', value: 'Alice' }
  
  await deleteValue('user:1');
})();
```

### Rust (reqwest)

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct PutRequest {
    value: String,
}

#[derive(Deserialize)]
struct GetResponse {
    key: String,
    value: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    
    // Store a value
    let response = client
        .post("http://localhost:3000/api/v1/kv/user:1")
        .json(&PutRequest { value: "Alice".to_string() })
        .send()
        .await?;
    println!("PUT response: {:?}", response.status());
    
    // Get a value
    let response: GetResponse = client
        .get("http://localhost:3000/api/v1/kv/user:1")
        .send()
        .await?
        .json()
        .await?;
    println!("Value: {}", response.value);
    
    // Delete a value
    client
        .delete("http://localhost:3000/api/v1/kv/user:1")
        .send()
        .await?;
    
    Ok(())
}
```

---

## 🚨 Error Responses

All errors follow a consistent format:

```json
{
  "error": "error_type",
  "message": "Human-readable error message"
}
```

### Error Types

| Status Code | Error Type | Description |
|-------------|------------|-------------|
| 400 | `bad_request` | Invalid request (e.g., empty key) |
| 404 | `not_found` | Key not found |
| 500 | `storage_error` | Internal storage engine error |

---

## ⚡ Performance Tips

### 1. Use Connection Pooling

Reuse HTTP connections for better performance:

```bash
# Using curl with keep-alive
curl --keepalive-time 60 http://localhost:3000/api/v1/kv/key1
```

### 2. Batch Operations

Group multiple operations in your application code:

```python
# Good: Batch operations
keys = ['key1', 'key2', 'key3']
with requests.Session() as session:
    for key in keys:
        session.get(f"{BASE_URL}/kv/{key}")

# Bad: Creating new connection each time
for key in keys:
    requests.get(f"{BASE_URL}/kv/{key}")  # New connection!
```

### 3. Monitor Cache Hit Rate

Check stats to see if you need to increase cache size:

```bash
curl http://localhost:3000/api/v1/stats | jq '.cache_size'
```

---

## 🔐 Security Considerations

**⚠️ Current Version (0.1.0):**

- No authentication implemented yet
- No rate limiting
- No TLS/HTTPS support
- **DO NOT expose to public internet**

**Coming Soon:**

- API key authentication
- JWT tokens
- Rate limiting
- TLS support

---

## 📊 Monitoring

### Health Check Endpoint

Use `/api/v1/health` for:

- Kubernetes liveness probes
- Load balancer health checks
- Uptime monitoring

```yaml
# Kubernetes example
livenessProbe:
  httpGet:
    path: /api/v1/health
    port: 3000
  initialDelaySeconds: 3
  periodSeconds: 3
```

### Stats Endpoint

Monitor storage metrics:

```bash
# Watch stats every 2 seconds
watch -n 2 'curl -s http://localhost:3000/api/v1/stats | jq .'
```

---

## 🐛 Troubleshooting

### Server won't start

```bash
# Check if port is already in use
lsof -i :3000

# Use a different port
QUARTZ_PORT=8080 cargo run -p quartz-server
```

### Cannot connect to server

```bash
# Verify server is running
curl http://localhost:3000/api/v1/health

# Check firewall settings
# Make sure port 3000 (or custom port) is open
```

### Slow performance

```bash
# Increase cache size
QUARTZ_CACHE_SIZE=100000 cargo run -p quartz-server

# Check storage stats
curl http://localhost:3000/api/v1/stats
```

---

## 📝 API Changelog

### Version 0.1.0 (Current)

- Initial release
- Basic CRUD operations
- Health check endpoint
- Stats endpoint
- JSON responses
- CORS support
- Request/response logging

---

## 🔮 Future API Features

Planned for upcoming versions:

- **Batch operations**: PUT/GET/DELETE multiple keys in one request
- **Key listing**: GET /api/v1/keys?prefix=user:
- **Range queries**: GET /api/v1/kv?start=key1&end=key9
- **TTL support**: Automatic key expiration
- **Webhooks**: Notifications on key changes
- **GraphQL API**: Alternative to REST
- **WebSocket support**: Real-time updates

---

## Built with ❤️ and Rust 🦀
