# QuartzDB Testing Guide v2.0
## Production-Grade Test Suite with Metrics & Progress

This guide covers the **complete test suite** for QuartzDB - from quick validation to performance stress testing.

---

## 🚀 Quick Start

```bash
cd /home/igor/projects/db/QuartzDB/tests

# Set API key (if authentication enabled)
export API_KEY="your_api_key"

# Quick validation (8 tests, ~2 seconds)
./quick_test.sh

# Real-world scenarios (5 tests, ~20 seconds)
./smoke_test.sh

# Business scenarios (6 industries, ~8 seconds)
./scenario_test.sh

# Performance stress test (~60 seconds)
./load_test.sh
```

---

## 📋 Test Suites Overview

| Suite | Purpose | Tests | Duration | Use Case |
|-------|---------|-------|----------|----------|
| **quick_test.sh** | API validation | 8 | ~2s | CI/CD, development |
| **smoke_test.sh** | Real-world scenarios | 5 | ~20s | Pre-deployment |
| **scenario_test.sh** | Business use cases | 6 | ~8s | Feature validation |
| **load_test.sh** | Performance & stress | 5 | ~60s | Capacity planning |

---

## 🧪 Test Suite Details

### 1. Quick Test Suite (`quick_test.sh`)

**Purpose:** Rapid API validation for development and CI/CD

```bash
./quick_test.sh [--verbose] [--timeout SECONDS]
```

**Tests:**
1. ✅ **Health Check** - Verify server is running
2. ✅ **Insert Vector** - Insert 384-dim vector with metadata
3. ✅ **Search Vectors** - k-NN search with k=5
4. ✅ **Delete Vector** - Soft-delete by ID
5. ✅ **Statistics** - Retrieve index stats
6. ✅ **Error Handling** - Reject invalid dimensions
7. ✅ **Bulk Insert** - 10 vectors with progress bar
8. ✅ **Latency Benchmark** - Average search latency

---

### 2. Smoke Test Suite (`smoke_test.sh`)

**Purpose:** Validate real-world usage scenarios

```bash
./smoke_test.sh [--verbose] [--timeout SECONDS]
```

**Scenarios:**
1. 🛒 **E-Commerce** - Product search with 5 products
2. 📄 **Documents** - Semantic search with 3 documents
3. 🔄 **Data Lifecycle** - Insert → Search → Delete flow
4. 📊 **Statistics** - System monitoring
5. ⚡ **Performance** - Search latency (10 iterations)

---

### 3. Business Scenario Tests (`scenario_test.sh`)

**Purpose:** Validate industry-specific use cases

```bash
./scenario_test.sh [scenario] [--timeout SECONDS]

# Run specific scenario
./scenario_test.sh ecommerce
./scenario_test.sh healthcare
./scenario_test.sh finance
./scenario_test.sh education
./scenario_test.sh media
./scenario_test.sh realestate

# Run all scenarios
./scenario_test.sh all
```

**Industries:**
| Scenario | Use Case | Data Size |
|----------|----------|-----------|
| **E-Commerce** | Product recommendations by image | 5 products |
| **Healthcare** | Medical document search (HIPAA) | 4 cases |
| **Finance** | Fraud detection patterns | 8 transactions |
| **Education** | Course recommendations | 4 courses |
| **Media** | Netflix-style content discovery | 6 items |
| **Real Estate** | Similar property matching | 6 properties |

---

### 4. Load Test Suite (`load_test.sh`)

**Purpose:** Performance and stress testing

```bash
./load_test.sh [--requests N] [--concurrency N] [--timeout SECONDS]

# Examples
./load_test.sh --requests 100 --concurrency 5
./load_test.sh -r 500 -c 10
```

**Tests:**
1. 📊 **Sequential Insert** - Insert N vectors, measure throughput
2. 📈 **Search Latency** - Latency distribution (min/avg/P50/P95/P99/max)
3. 👥 **Concurrent Users** - Simulate N concurrent users
4. 💾 **Memory Pressure** - Rapid 50-vector insertion
5. ⏱️ **Sustained Load** - 10-second continuous requests

**Metrics Collected:**
- Throughput (requests/second)
- Latency percentiles (P50, P95, P99)
- Error rates
- Success/failure counts

---

## 🔧 Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `BASE_URL` | `http://localhost:8787` | Server URL |
| `API_KEY` | (none) | Authentication key |
| `TIMEOUT` | `10` | Request timeout (seconds) |
| `REQUESTS` | `100` | Total requests (load test) |
| `CONCURRENCY` | `5` | Concurrent users (load test) |

### Command Line Options

```bash
# Quick test with verbose output
./quick_test.sh --verbose

# Custom timeout
./quick_test.sh --timeout 30

# Load test configuration
./load_test.sh --requests 200 --concurrency 10 --timeout 15
```

---

## 📊 Test Output Features

All test suites include:

1. **Progress Bars** - Visual progress for long operations
   ```
   [████████████████████] 100% Vector 10/10
   ```

2. **Timestamps** - Every log entry includes time
   ```
   [14:23:14] INFO  Bulk inserting 10 vectors...
   ```

3. **Metrics** - Detailed performance data
   ```
   Latency:
     Min:        20ms
     Avg:        22ms
     P50:        22ms
     P95:        26ms
     P99:        26ms
     Max:        26ms
   ```

4. **Summary Tables** - Clear pass/fail status with timing
   ```
   TEST                      STATUS   TIME
   ───────────────────────── ────── ────────
   Health Check              PASS     15ms
   Insert Vector             PASS     71ms
   ```

---

## ✅ Test Coverage Summary

| Category | Tests | Coverage |
|----------|-------|----------|
| **API Endpoints** | Health, Insert, Search, Delete, Stats | 100% |
| **Error Handling** | Invalid dimensions, missing fields | Covered |
| **Performance** | Latency, throughput, concurrent users | Comprehensive |
| **Business Scenarios** | 6 industries | E-commerce to Real Estate |
| **Stress Testing** | Memory pressure, sustained load | Production-ready |

---

## 🎯 Recommended Test Workflow

1. **Development** → `./quick_test.sh` (2 seconds)
2. **Pre-commit** → `./quick_test.sh && ./smoke_test.sh` (22 seconds)
3. **Pre-deployment** → All 4 suites (90 seconds)
4. **Capacity Planning** → `./load_test.sh -r 1000 -c 50`
