'use client';

import Link from 'next/link';
import { useState } from 'react';
import { 
  Database,
  ArrowLeft,
  Book,
  Terminal,
  Search,
  Copy,
  Check,
  ChevronDown,
  ChevronRight,
  AlertTriangle,
  Info
} from 'lucide-react';

const API_BASE = 'https://api.quartzdb.io';

// Sample 384-dimensional vector for documentation examples
// This is a deterministic vector where each element = index * 0.002617
// In real usage, you would use embeddings from a model like all-MiniLM-L6-v2
const SAMPLE_VECTOR_PREVIEW = '[0.0, 0.002617, 0.005234, 0.007851, 0.010468, 0.013085, 0.015702, 0.018319, 0.020936, 0.023553, ...]';

// Generate full sample vector for copy-paste examples
function generateSampleVector(): string {
  const vec = Array.from({ length: 384 }, (_, i) => Number((i * 0.002617).toFixed(6)));
  return JSON.stringify(vec);
}

export default function DocsPage() {
  const sampleVector = generateSampleVector();
  
  return (
    <div className="min-h-screen bg-gray-900 text-white">
      <Header />
      
      <div className="max-w-6xl mx-auto px-4 py-8">
        <div className="grid lg:grid-cols-4 gap-8">
          {/* Sidebar */}
          <aside className="lg:col-span-1">
            <nav className="sticky top-8 space-y-2">
              <NavSection title="Getting Started">
                <NavLink href="#overview">Overview</NavLink>
                <NavLink href="#getting-api-key">Getting an API Key</NavLink>
                <NavLink href="#authentication">Authentication</NavLink>
                <NavLink href="#quickstart">Quick Start</NavLink>
              </NavSection>
              <NavSection title="API Reference">
                <NavLink href="#health">Health Check</NavLink>
                <NavLink href="#insert">Insert Vector</NavLink>
                <NavLink href="#batch-insert">Batch Insert</NavLink>
                <NavLink href="#search">Search</NavLink>
                <NavLink href="#get">Get Vector</NavLink>
                <NavLink href="#delete">Delete Vector</NavLink>
                <NavLink href="#stats">Statistics</NavLink>
              </NavSection>
              <NavSection title="Guides">
                <NavLink href="#embeddings">Creating Embeddings</NavLink>
                <NavLink href="#best-practices">Best Practices</NavLink>
              </NavSection>
            </nav>
          </aside>

          {/* Main Content */}
          <main className="lg:col-span-3 space-y-12">
            {/* Overview */}
            <section id="overview">
              <h1 className="text-4xl font-bold mb-4">QuartzDB Documentation</h1>
              <p className="text-gray-400 text-lg mb-6">
                QuartzDB is a serverless vector database built on Cloudflare Workers. 
                It provides fast semantic search using the HNSW algorithm with zero cold starts.
              </p>
              
              <InfoBox type="info">
                <strong>What is a Vector Database?</strong><br/>
                A vector database stores numerical representations (embeddings) of data like text, images, or audio.
                It enables semantic search - finding similar items based on meaning, not just keywords.
              </InfoBox>
              
              <div className="grid md:grid-cols-3 gap-4 mt-6">
                <FeatureCard icon={<Terminal />} title="REST API" desc="Simple HTTP endpoints" />
                <FeatureCard icon={<Search />} title="HNSW Search" desc="Sub-millisecond queries" />
                <FeatureCard icon={<Database />} title="Durable Storage" desc="Persistent vectors" />
              </div>
            </section>

            {/* Getting API Key */}
            <section id="getting-api-key">
              <h2 className="text-2xl font-bold mb-4">Getting an API Key</h2>
              
              <InfoBox type="warning">
                <strong>Current Status:</strong> QuartzDB is in private beta. API keys are issued manually by administrators.
              </InfoBox>
              
              <div className="mt-6 space-y-4">
                <h3 className="text-lg font-semibold">How to Request Access</h3>
                
                <div className="bg-gray-800 border border-gray-700 rounded-lg p-6">
                  <ol className="list-decimal list-inside space-y-3 text-gray-300">
                    <li>
                      <strong>Contact the administrator</strong> - Email or message the QuartzDB team with your use case
                    </li>
                    <li>
                      <strong>Receive your API key</strong> - You&apos;ll get a 64-character hexadecimal key
                    </li>
                    <li>
                      <strong>Store it securely</strong> - Save the key in environment variables, never commit to git
                    </li>
                    <li>
                      <strong>Start using the API</strong> - Include the key in the <code className="text-cyan-400">X-API-Key</code> header
                    </li>
                  </ol>
                </div>
                
                <h3 className="text-lg font-semibold mt-6">API Key Format</h3>
                <p className="text-gray-400 mb-2">
                  API keys are 64-character hexadecimal strings:
                </p>
                <CodeBlock 
                  language="text"
                  code="8d9f52035876a6ade379a6f5208d34e0849bdf44e1ba58417e4bef821537232e"
                />
                
                <h3 className="text-lg font-semibold mt-6">Security Best Practices</h3>
                <ul className="list-disc list-inside text-gray-400 space-y-2">
                  <li>Never share your API key publicly</li>
                  <li>Use environment variables to store keys</li>
                  <li>Rotate keys periodically</li>
                  <li>Use different keys for development and production</li>
                </ul>
              </div>
            </section>

            {/* Authentication */}
            <section id="authentication">
              <h2 className="text-2xl font-bold mb-4">Authentication</h2>
              <p className="text-gray-400 mb-4">
                All API endpoints (except <code className="text-cyan-400">/health</code>) require an API key.
                Include your key in the <code className="text-cyan-400">X-API-Key</code> header.
              </p>
              
              <h4 className="font-semibold mb-2">Example Request</h4>
              <CodeBlock 
                language="bash"
                code={`# Replace YOUR_API_KEY with your actual key
curl -s "${API_BASE}/api/vector/stats" \\
  -H "X-API-Key: YOUR_API_KEY"

# Example with actual key format:
curl -s "${API_BASE}/api/vector/stats" \\
  -H "X-API-Key: 8d9f52035876a6ade379a6f5208d34e0849bdf44e1ba58417e4bef821537232e"`}
              />
              
              <h4 className="font-semibold mt-6 mb-2">Expected Response (Success)</h4>
              <CodeBlock 
                language="json"
                code={`{
  "success": true,
  "algorithm": "HNSW",
  "dimension": 384,
  "num_vectors": 42,
  "num_active": 40,
  "num_deleted": 2,
  "deletion_ratio_percent": "4.8",
  "recommendation": "Healthy: <10% vectors deleted"
}`}
              />
              
              <h4 className="font-semibold mt-6 mb-2">Error Response (Invalid Key)</h4>
              <CodeBlock 
                language="text"
                code="Unauthorized: Invalid or missing API key"
              />
            </section>

            {/* Quick Start */}
            <section id="quickstart">
              <h2 className="text-2xl font-bold mb-4">Quick Start</h2>
              
              <InfoBox type="info">
                All examples below use a sample 384-dimensional vector. In production, you would generate 
                vectors using an embedding model (see <a href="#embeddings" className="text-cyan-400 hover:underline">Creating Embeddings</a>).
              </InfoBox>
              
              <div className="space-y-8 mt-6">
                <Step number={1} title="Check the service is running">
                  <p className="text-gray-400 mb-3">No authentication required for health check:</p>
                  <CodeBlock 
                    language="bash"
                    code={`curl -s "${API_BASE}/health" | jq .`}
                  />
                  <h4 className="font-semibold mt-4 mb-2">Expected Response</h4>
                  <CodeBlock 
                    language="json"
                    code={`{
  "status": "healthy",
  "service": "quartz-faas",
  "version": "0.1.0",
  "uptime_seconds": 3600,
  "checks": {
    "storage": "ok",
    "vector_index": "ok"
  }
}`}
                  />
                </Step>

                <Step number={2} title="Insert a vector">
                  <p className="text-gray-400 mb-3">
                    Insert a vector with a unique ID and optional metadata. The vector must be exactly 384 dimensions.
                  </p>
                  <CodeBlock 
                    language="bash"
                    code={`# Set your API key
export API_KEY="YOUR_API_KEY"

# Insert a vector (384 dimensions required)
curl -s -X POST "${API_BASE}/api/vector/insert" \\
  -H "Content-Type: application/json" \\
  -H "X-API-Key: $API_KEY" \\
  -d '{
    "id": "doc_quickstart_1",
    "vector": ${sampleVector},
    "metadata": {
      "title": "Quick Start Example",
      "category": "documentation",
      "created": "2026-01-04"
    }
  }' | jq .`}
                  />
                  <h4 className="font-semibold mt-4 mb-2">Expected Response</h4>
                  <CodeBlock 
                    language="json"
                    code={`{
  "success": true,
  "id": "doc_quickstart_1",
  "message": "Vector inserted into HNSW index"
}`}
                  />
                </Step>

                <Step number={3} title="Search for similar vectors">
                  <p className="text-gray-400 mb-3">
                    Find the k nearest neighbors to a query vector:
                  </p>
                  <CodeBlock 
                    language="bash"
                    code={`curl -s -X POST "${API_BASE}/api/vector/search" \\
  -H "Content-Type: application/json" \\
  -H "X-API-Key: $API_KEY" \\
  -d '{
    "vector": ${sampleVector},
    "k": 5
  }' | jq .`}
                  />
                  <h4 className="font-semibold mt-4 mb-2">Expected Response</h4>
                  <CodeBlock 
                    language="json"
                    code={`{
  "success": true,
  "count": 5,
  "algorithm": "HNSW",
  "results": [
    {
      "id": "doc_quickstart_1",
      "score": 1.0,
      "distance": 0.0,
      "metadata": {
        "title": "Quick Start Example",
        "category": "documentation",
        "created": "2026-01-04"
      }
    },
    {
      "id": "doc_similar_1",
      "score": 0.89,
      "distance": 0.11,
      "metadata": {"title": "Another Document"}
    }
  ]
}`}
                  />
                  <InfoBox type="info">
                    <strong>Understanding scores:</strong> Score of 1.0 = exact match, 0.0 = completely different. 
                    Distance is the inverse (0.0 = identical, higher = more different).
                  </InfoBox>
                </Step>
              </div>
            </section>

            {/* API Reference - Health */}
            <section id="health">
              <h2 className="text-2xl font-bold mb-4">Health Check</h2>
              <EndpointBadge method="GET" path="/health" auth={false} />
              <p className="text-gray-400 my-4">Check if the service is running. No authentication required.</p>
              
              <h4 className="font-semibold mb-2">Request</h4>
              <CodeBlock 
                language="bash"
                code={`curl -s "${API_BASE}/health"`}
              />
              
              <h4 className="font-semibold mt-6 mb-2">Response</h4>
              <CodeBlock 
                language="json"
                code={`{
  "status": "healthy",
  "service": "quartz-faas",
  "version": "0.1.0",
  "uptime_seconds": 3600,
  "checks": {
    "storage": "ok",
    "vector_index": "ok"
  }
}`}
              />
            </section>

            {/* API Reference - Insert */}
            <section id="insert">
              <h2 className="text-2xl font-bold mb-4">Insert Vector</h2>
              <EndpointBadge method="POST" path="/api/vector/insert" auth={true} />
              <p className="text-gray-400 my-4">Insert a single vector with optional metadata.</p>
              
              <h4 className="font-semibold mb-2">Request Body Parameters</h4>
              <ParamTable params={[
                { name: 'id', type: 'string', required: true, desc: 'Unique identifier (1-256 chars, alphanumeric, dash, underscore)' },
                { name: 'vector', type: 'number[]', required: true, desc: 'Array of exactly 384 floating-point numbers' },
                { name: 'metadata', type: 'object', required: false, desc: 'Optional JSON object with custom data' },
              ]} />
              
              <h4 className="font-semibold mt-6 mb-2">Request Example</h4>
              <CodeBlock 
                language="bash"
                code={`curl -s -X POST "${API_BASE}/api/vector/insert" \\
  -H "Content-Type: application/json" \\
  -H "X-API-Key: $API_KEY" \\
  -d '{
    "id": "product_12345",
    "vector": ${sampleVector},
    "metadata": {
      "name": "Wireless Headphones",
      "price": 79.99,
      "category": "electronics",
      "tags": ["audio", "bluetooth", "wireless"]
    }
  }' | jq .`}
              />
              
              <h4 className="font-semibold mt-6 mb-2">Success Response</h4>
              <CodeBlock 
                language="json"
                code={`{
  "success": true,
  "id": "product_12345",
  "message": "Vector inserted into HNSW index"
}`}
              />
              
              <h4 className="font-semibold mt-6 mb-2">Error Response (duplicate ID)</h4>
              <CodeBlock 
                language="json"
                code={`{
  "success": false,
  "error": "Vector with ID 'product_12345' already exists"
}`}
              />
            </section>

            {/* API Reference - Batch Insert */}
            <section id="batch-insert">
              <h2 className="text-2xl font-bold mb-4">Batch Insert</h2>
              <EndpointBadge method="POST" path="/api/vector/batch-insert" auth={true} />
              <p className="text-gray-400 my-4">Insert multiple vectors in a single request. Maximum 100 vectors per batch.</p>
              
              <h4 className="font-semibold mb-2">Request Body Parameters</h4>
              <ParamTable params={[
                { name: 'vectors', type: 'array', required: true, desc: 'Array of vector objects (max 100). Each must have id, vector, and optional metadata.' },
              ]} />
              
              <h4 className="font-semibold mt-6 mb-2">Request Example</h4>
              <CodeBlock 
                language="bash"
                code={`curl -s -X POST "${API_BASE}/api/vector/batch-insert" \\
  -H "Content-Type: application/json" \\
  -H "X-API-Key: $API_KEY" \\
  -d '{
    "vectors": [
      {
        "id": "batch_item_1",
        "vector": ${sampleVector},
        "metadata": {"index": 1}
      },
      {
        "id": "batch_item_2",
        "vector": ${sampleVector},
        "metadata": {"index": 2}
      }
    ]
  }' | jq .`}
              />
              
              <h4 className="font-semibold mt-6 mb-2">Success Response</h4>
              <CodeBlock 
                language="json"
                code={`{
  "success": true,
  "total": 2,
  "inserted": 2,
  "failed": 0,
  "results": [
    {"id": "batch_item_1", "success": true, "message": "inserted"},
    {"id": "batch_item_2", "success": true, "message": "inserted"}
  ]
}`}
              />
            </section>

            {/* API Reference - Search */}
            <section id="search">
              <h2 className="text-2xl font-bold mb-4">Search Vectors</h2>
              <EndpointBadge method="POST" path="/api/vector/search" auth={true} />
              <p className="text-gray-400 my-4">Find the k nearest vectors to a query vector using HNSW algorithm.</p>
              
              <h4 className="font-semibold mb-2">Request Body Parameters</h4>
              <ParamTable params={[
                { name: 'vector', type: 'number[]', required: true, desc: 'Query vector (exactly 384 dimensions)' },
                { name: 'k', type: 'number', required: false, desc: 'Number of results to return (default: 10, max: 100)' },
              ]} />
              
              <h4 className="font-semibold mt-6 mb-2">Request Example</h4>
              <CodeBlock 
                language="bash"
                code={`curl -s -X POST "${API_BASE}/api/vector/search" \\
  -H "Content-Type: application/json" \\
  -H "X-API-Key: $API_KEY" \\
  -d '{
    "vector": ${sampleVector},
    "k": 10
  }' | jq .`}
              />
              
              <h4 className="font-semibold mt-6 mb-2">Response</h4>
              <CodeBlock 
                language="json"
                code={`{
  "success": true,
  "count": 3,
  "algorithm": "HNSW",
  "results": [
    {
      "id": "product_12345",
      "score": 0.95,
      "distance": 0.05,
      "metadata": {
        "name": "Wireless Headphones",
        "price": 79.99
      }
    },
    {
      "id": "product_67890",
      "score": 0.82,
      "distance": 0.18,
      "metadata": {
        "name": "Bluetooth Speaker",
        "price": 49.99
      }
    },
    {
      "id": "product_11111",
      "score": 0.76,
      "distance": 0.24,
      "metadata": {
        "name": "Earbuds",
        "price": 29.99
      }
    }
  ]
}`}
              />
              
              <InfoBox type="info">
                <strong>Score interpretation:</strong>
                <ul className="list-disc list-inside mt-2">
                  <li>1.0 = Identical vector (exact match)</li>
                  <li>0.8+ = Very similar (strong semantic match)</li>
                  <li>0.6-0.8 = Somewhat similar</li>
                  <li>&lt;0.6 = Loosely related or unrelated</li>
                </ul>
              </InfoBox>
            </section>

            {/* API Reference - Get */}
            <section id="get">
              <h2 className="text-2xl font-bold mb-4">Get Vector by ID</h2>
              <EndpointBadge method="GET" path="/api/vector/get/:id" auth={true} />
              <p className="text-gray-400 my-4">Retrieve a specific vector and its metadata by ID.</p>
              
              <h4 className="font-semibold mb-2">URL Parameters</h4>
              <ParamTable params={[
                { name: 'id', type: 'string', required: true, desc: 'The ID of the vector to retrieve' },
              ]} />
              
              <h4 className="font-semibold mt-6 mb-2">Request Example</h4>
              <CodeBlock 
                language="bash"
                code={`curl -s "${API_BASE}/api/vector/get/product_12345" \\
  -H "X-API-Key: $API_KEY" | jq .`}
              />
              
              <h4 className="font-semibold mt-6 mb-2">Success Response</h4>
              <CodeBlock 
                language="json"
                code={`{
  "id": "product_12345",
  "vector": [0.0, 0.002617, 0.005234, ...],
  "metadata": {
    "name": "Wireless Headphones",
    "price": 79.99
  }
}`}
              />
              
              <h4 className="font-semibold mt-6 mb-2">Error Response (not found)</h4>
              <CodeBlock 
                language="text"
                code="Vector 'nonexistent_id' not found"
              />
            </section>

            {/* API Reference - Delete */}
            <section id="delete">
              <h2 className="text-2xl font-bold mb-4">Delete Vector</h2>
              <EndpointBadge method="DELETE" path="/api/vector/delete" auth={true} />
              <p className="text-gray-400 my-4">Soft-delete a vector by ID. The vector is marked as deleted but storage is reclaimed during compaction.</p>
              
              <h4 className="font-semibold mb-2">Request Body Parameters</h4>
              <ParamTable params={[
                { name: 'id', type: 'string', required: true, desc: 'ID of the vector to delete' },
              ]} />
              
              <h4 className="font-semibold mt-6 mb-2">Request Example</h4>
              <CodeBlock 
                language="bash"
                code={`curl -s -X DELETE "${API_BASE}/api/vector/delete" \\
  -H "Content-Type: application/json" \\
  -H "X-API-Key: $API_KEY" \\
  -d '{"id": "product_12345"}' | jq .`}
              />
              
              <h4 className="font-semibold mt-6 mb-2">Success Response</h4>
              <CodeBlock 
                language="json"
                code={`{
  "success": true,
  "message": "Vector deleted"
}`}
              />
            </section>

            {/* API Reference - Stats */}
            <section id="stats">
              <h2 className="text-2xl font-bold mb-4">Statistics</h2>
              <EndpointBadge method="GET" path="/api/vector/stats" auth={true} />
              <p className="text-gray-400 my-4">Get index statistics and health information.</p>
              
              <h4 className="font-semibold mb-2">Request Example</h4>
              <CodeBlock 
                language="bash"
                code={`curl -s "${API_BASE}/api/vector/stats" \\
  -H "X-API-Key: $API_KEY" | jq .`}
              />
              
              <h4 className="font-semibold mt-6 mb-2">Response</h4>
              <CodeBlock 
                language="json"
                code={`{
  "success": true,
  "algorithm": "HNSW",
  "dimension": 384,
  "num_vectors": 1000,
  "num_active": 950,
  "num_deleted": 50,
  "num_nodes": 950,
  "entry_point_level": 3,
  "connections_per_layer": [0, 12, 156, 782],
  "deletion_ratio_percent": "5.0",
  "recommendation": "Healthy: <10% vectors deleted"
}`}
              />
              
              <h4 className="font-semibold mt-6 mb-2">Response Fields</h4>
              <ParamTable params={[
                { name: 'dimension', type: 'number', required: true, desc: 'Vector dimensions (always 384)' },
                { name: 'num_vectors', type: 'number', required: true, desc: 'Total vectors (active + deleted)' },
                { name: 'num_active', type: 'number', required: true, desc: 'Active (non-deleted) vectors' },
                { name: 'num_deleted', type: 'number', required: true, desc: 'Soft-deleted vectors' },
                { name: 'deletion_ratio_percent', type: 'string', required: true, desc: 'Percentage of deleted vectors' },
                { name: 'recommendation', type: 'string', required: true, desc: 'Health recommendation' },
              ]} />
            </section>

            {/* Embeddings Guide */}
            <section id="embeddings">
              <h2 className="text-2xl font-bold mb-4">Creating Embeddings</h2>
              
              <InfoBox type="warning">
                <strong>Important:</strong> QuartzDB stores vectors - it does NOT generate embeddings.
                You must use an external embedding model to convert text/images into 384-dimensional vectors.
              </InfoBox>
              
              <div className="mt-6 space-y-6">
                <h3 className="text-lg font-semibold">What are Embeddings?</h3>
                <p className="text-gray-400">
                  Embeddings are numerical representations of data (text, images, audio) that capture semantic meaning.
                  Similar items have similar embeddings, enabling semantic search.
                </p>
                
                <h3 className="text-lg font-semibold">Recommended Models (384 dimensions)</h3>
                <div className="bg-gray-800 border border-gray-700 rounded-lg overflow-hidden">
                  <table className="w-full text-sm">
                    <thead className="bg-gray-700/50">
                      <tr>
                        <th className="text-left px-4 py-2">Model</th>
                        <th className="text-left px-4 py-2">Best For</th>
                        <th className="text-left px-4 py-2">Speed</th>
                      </tr>
                    </thead>
                    <tbody className="divide-y divide-gray-700">
                      <tr>
                        <td className="px-4 py-2 font-mono text-cyan-400">all-MiniLM-L6-v2</td>
                        <td className="px-4 py-2 text-gray-400">General semantic search</td>
                        <td className="px-4 py-2 text-gray-400">Fast</td>
                      </tr>
                      <tr>
                        <td className="px-4 py-2 font-mono text-cyan-400">paraphrase-MiniLM-L6-v2</td>
                        <td className="px-4 py-2 text-gray-400">Paraphrase detection</td>
                        <td className="px-4 py-2 text-gray-400">Fast</td>
                      </tr>
                      <tr>
                        <td className="px-4 py-2 font-mono text-cyan-400">multi-qa-MiniLM-L6-cos-v1</td>
                        <td className="px-4 py-2 text-gray-400">Question answering</td>
                        <td className="px-4 py-2 text-gray-400">Fast</td>
                      </tr>
                    </tbody>
                  </table>
                </div>

                <h3 className="text-lg font-semibold mt-6">Python Example (Complete)</h3>
                <p className="text-gray-400 mb-3">
                  Install dependencies first: <code className="text-cyan-400">pip install sentence-transformers requests</code>
                </p>
                <CodeBlock 
                  language="python"
                  code={`#!/usr/bin/env python3
"""
QuartzDB Python Example - Complete working code
Requires: pip install sentence-transformers requests
"""

from sentence_transformers import SentenceTransformer
import requests
import json

# Configuration
API_URL = "${API_BASE}"
API_KEY = "YOUR_API_KEY"  # Replace with your actual API key

# Load the embedding model (downloads on first run, ~90MB)
print("Loading embedding model...")
model = SentenceTransformer('all-MiniLM-L6-v2')

# Sample documents to index
documents = [
    {"id": "doc_1", "text": "Python is a programming language", "category": "tech"},
    {"id": "doc_2", "text": "Machine learning enables computers to learn", "category": "ai"},
    {"id": "doc_3", "text": "Vector databases store embeddings", "category": "database"},
]

# Insert documents
print("\\nInserting documents...")
for doc in documents:
    # Generate embedding (returns 384-dimensional vector)
    vector = model.encode(doc["text"]).tolist()
    
    # Insert into QuartzDB
    response = requests.post(
        f"{API_URL}/api/vector/insert",
        headers={
            "Content-Type": "application/json",
            "X-API-Key": API_KEY
        },
        json={
            "id": doc["id"],
            "vector": vector,
            "metadata": {
                "text": doc["text"],
                "category": doc["category"]
            }
        }
    )
    print(f"  {doc['id']}: {response.json()}")

# Search for similar documents
query = "What is artificial intelligence?"
print(f"\\nSearching for: '{query}'")

query_vector = model.encode(query).tolist()

response = requests.post(
    f"{API_URL}/api/vector/search",
    headers={
        "Content-Type": "application/json",
        "X-API-Key": API_KEY
    },
    json={
        "vector": query_vector,
        "k": 3
    }
)

results = response.json()
print(f"\\nTop {len(results['results'])} results:")
for i, result in enumerate(results['results'], 1):
    print(f"  {i}. {result['id']} (score: {result['score']:.3f})")
    if result.get('metadata'):
        print(f"     Text: {result['metadata'].get('text', 'N/A')}")`}
                />

                <h3 className="text-lg font-semibold mt-6">JavaScript/Node.js Example</h3>
                <p className="text-gray-400 mb-3">
                  For Node.js, use the <code className="text-cyan-400">@xenova/transformers</code> package:
                </p>
                <CodeBlock 
                  language="javascript"
                  code={`// npm install @xenova/transformers

import { pipeline } from '@xenova/transformers';

const API_URL = '${API_BASE}';
const API_KEY = 'YOUR_API_KEY';

// Load the embedding model
const embedder = await pipeline('feature-extraction', 'Xenova/all-MiniLM-L6-v2');

// Generate embedding for text
async function getEmbedding(text) {
  const output = await embedder(text, { pooling: 'mean', normalize: true });
  return Array.from(output.data);
}

// Insert a document
async function insertDocument(id, text, metadata = {}) {
  const vector = await getEmbedding(text);
  
  const response = await fetch(\`\${API_URL}/api/vector/insert\`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'X-API-Key': API_KEY
    },
    body: JSON.stringify({ id, vector, metadata: { ...metadata, text } })
  });
  
  return response.json();
}

// Search for similar documents
async function search(query, k = 10) {
  const vector = await getEmbedding(query);
  
  const response = await fetch(\`\${API_URL}/api/vector/search\`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'X-API-Key': API_KEY
    },
    body: JSON.stringify({ vector, k })
  });
  
  return response.json();
}

// Example usage
await insertDocument('doc_1', 'QuartzDB is a vector database');
const results = await search('What is QuartzDB?');
console.log(results);`}
                />
              </div>
            </section>

            {/* Best Practices */}
            <section id="best-practices">
              <h2 className="text-2xl font-bold mb-4">Best Practices</h2>
              
              <div className="space-y-4">
                <BestPractice 
                  title="Use batch insert for bulk data"
                  desc="When inserting many vectors, use /api/vector/batch-insert with up to 100 vectors per request. This is much faster than individual inserts."
                />
                <BestPractice 
                  title="Normalize your vectors"
                  desc="For best cosine similarity results, normalize vectors to unit length. Most embedding models do this automatically."
                />
                <BestPractice 
                  title="Store useful metadata"
                  desc="Include metadata like text snippets, IDs, URLs, or categories to make search results actionable without additional lookups."
                />
                <BestPractice 
                  title="Use meaningful IDs"
                  desc="Use descriptive IDs like 'product_12345' or 'doc_article_slug' instead of random UUIDs for easier debugging."
                />
                <BestPractice 
                  title="Monitor deletion ratio"
                  desc="Check /api/vector/stats regularly. If deletion_ratio_percent exceeds 20%, index performance may degrade."
                />
                <BestPractice 
                  title="Handle errors gracefully"
                  desc="Always check response success field and handle HTTP error codes (401 unauthorized, 400 bad request, 429 rate limited)."
                />
              </div>
            </section>
          </main>
        </div>
      </div>
    </div>
  );
}

function Header() {
  return (
    <header className="border-b border-gray-700 sticky top-0 bg-gray-900/95 backdrop-blur z-10">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex justify-between items-center py-4">
          <div className="flex items-center gap-4">
            <Link href="/" className="flex items-center gap-2 text-gray-400 hover:text-white transition">
              <ArrowLeft className="h-4 w-4" />
              Back
            </Link>
            <div className="flex items-center gap-2">
              <Book className="h-6 w-6 text-cyan-400" />
              <span className="text-xl font-bold">Documentation</span>
            </div>
          </div>
          <div className="flex items-center gap-4">
            <Link href="/playground" className="text-gray-300 hover:text-white transition">
              Playground
            </Link>
            <Link href="/dashboard" className="bg-cyan-500 hover:bg-cyan-600 px-4 py-2 rounded-lg font-medium transition">
              Dashboard
            </Link>
          </div>
        </div>
      </div>
    </header>
  );
}

function NavSection({ title, children }: { title: string; children: React.ReactNode }) {
  const [open, setOpen] = useState(true);
  return (
    <div>
      <button 
        onClick={() => setOpen(!open)}
        className="flex items-center gap-2 text-sm font-semibold text-gray-400 mb-2 hover:text-white transition"
      >
        {open ? <ChevronDown className="h-4 w-4" /> : <ChevronRight className="h-4 w-4" />}
        {title}
      </button>
      {open && <div className="space-y-1 ml-4">{children}</div>}
    </div>
  );
}

function NavLink({ href, children }: { href: string; children: React.ReactNode }) {
  return (
    <a 
      href={href} 
      className="block text-sm text-gray-500 hover:text-cyan-400 transition py-1"
    >
      {children}
    </a>
  );
}

function FeatureCard({ icon, title, desc }: { icon: React.ReactNode; title: string; desc: string }) {
  return (
    <div className="bg-gray-800 border border-gray-700 rounded-lg p-4">
      <div className="text-cyan-400 mb-2">{icon}</div>
      <div className="font-semibold">{title}</div>
      <div className="text-sm text-gray-400">{desc}</div>
    </div>
  );
}

function InfoBox({ type, children }: { type: 'info' | 'warning'; children: React.ReactNode }) {
  const styles = {
    info: 'bg-blue-500/10 border-blue-500/30 text-blue-300',
    warning: 'bg-yellow-500/10 border-yellow-500/30 text-yellow-300'
  };
  const icons = {
    info: <Info className="h-5 w-5 flex-shrink-0" />,
    warning: <AlertTriangle className="h-5 w-5 flex-shrink-0" />
  };
  
  return (
    <div className={`p-4 rounded-lg border flex gap-3 ${styles[type]}`}>
      {icons[type]}
      <div className="text-sm">{children}</div>
    </div>
  );
}

function CodeBlock({ language, code }: { language: string; code: string }) {
  const [copied, setCopied] = useState(false);

  const copy = () => {
    navigator.clipboard.writeText(code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="bg-gray-800 border border-gray-700 rounded-lg overflow-hidden">
      <div className="flex justify-between items-center px-4 py-2 border-b border-gray-700 bg-gray-800/50">
        <span className="text-xs text-gray-500 uppercase">{language}</span>
        <button onClick={copy} className="text-gray-400 hover:text-white transition flex items-center gap-1 text-sm">
          {copied ? <Check className="h-4 w-4" /> : <Copy className="h-4 w-4" />}
          {copied ? 'Copied!' : 'Copy'}
        </button>
      </div>
      <pre className="p-4 overflow-x-auto text-sm">
        <code>{code}</code>
      </pre>
    </div>
  );
}

function EndpointBadge({ method, path, auth }: { method: string; path: string; auth: boolean }) {
  const colors: Record<string, string> = {
    GET: 'bg-green-500/20 text-green-400 border-green-500/30',
    POST: 'bg-blue-500/20 text-blue-400 border-blue-500/30',
    DELETE: 'bg-red-500/20 text-red-400 border-red-500/30',
  };

  return (
    <div className="flex items-center gap-3">
      <span className={`px-2 py-1 rounded text-xs font-bold border ${colors[method]}`}>
        {method}
      </span>
      <code className="text-cyan-400">{path}</code>
      {auth && (
        <span className="text-xs text-gray-500 bg-gray-700 px-2 py-1 rounded">
          🔒 Auth Required
        </span>
      )}
    </div>
  );
}

function ParamTable({ params }: { params: { name: string; type: string; required: boolean; desc: string }[] }) {
  return (
    <div className="bg-gray-800 border border-gray-700 rounded-lg overflow-hidden">
      <table className="w-full text-sm">
        <thead className="bg-gray-700/50">
          <tr>
            <th className="text-left px-4 py-2">Parameter</th>
            <th className="text-left px-4 py-2">Type</th>
            <th className="text-left px-4 py-2">Required</th>
            <th className="text-left px-4 py-2">Description</th>
          </tr>
        </thead>
        <tbody className="divide-y divide-gray-700">
          {params.map((p) => (
            <tr key={p.name}>
              <td className="px-4 py-2 font-mono text-cyan-400">{p.name}</td>
              <td className="px-4 py-2 text-gray-400">{p.type}</td>
              <td className="px-4 py-2">{p.required ? '✓' : '-'}</td>
              <td className="px-4 py-2 text-gray-400">{p.desc}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function Step({ number, title, children }: { number: number; title: string; children: React.ReactNode }) {
  return (
    <div>
      <div className="flex items-center gap-3 mb-3">
        <span className="flex items-center justify-center w-8 h-8 bg-cyan-500 rounded-full font-bold">
          {number}
        </span>
        <h3 className="text-lg font-semibold">{title}</h3>
      </div>
      {children}
    </div>
  );
}

function BestPractice({ title, desc }: { title: string; desc: string }) {
  return (
    <div className="bg-gray-800 border border-gray-700 rounded-lg p-4">
      <div className="flex items-start gap-3">
        <Check className="h-5 w-5 text-green-400 flex-shrink-0 mt-0.5" />
        <div>
          <div className="font-semibold">{title}</div>
          <div className="text-sm text-gray-400">{desc}</div>
        </div>
      </div>
    </div>
  );
}
