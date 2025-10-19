#!/usr/bin/env python3
"""
QuartzDB Vector Search Example

This example demonstrates semantic search using QuartzDB's vector search API
with sentence-transformers for embedding generation.

Requirements:
    pip install sentence-transformers requests

Usage:
    python semantic_search_demo.py
"""

import requests
from sentence_transformers import SentenceTransformer
import json

# Configuration
QUARTZ_URL = "http://localhost:3000"
MODEL_NAME = "all-MiniLM-L6-v2"  # 384 dimensions
VECTOR_DIMENSION = 384

def initialize_index():
    """Initialize the vector index with appropriate configuration."""
    print("🔧 Initializing vector index...")
    
    response = requests.post(
        f"{QUARTZ_URL}/api/v1/indexes/semantic_384d",
        json={
            "dimension": VECTOR_DIMENSION,
            "metric": "cosine"
        }
    )
    
    if response.status_code == 200:
        print("✅ Vector index initialized successfully")
        return True
    else:
        print(f"❌ Failed to initialize index: {response.text}")
        return False

def insert_documents(model, documents):
    """Insert documents into the vector index."""
    print(f"\n📝 Inserting {len(documents)} documents...")
    
    for i, doc in enumerate(documents, 1):
        # Generate embedding
        embedding = model.encode(doc).tolist()
        
        # Insert into QuartzDB
        response = requests.post(
            f"{QUARTZ_URL}/api/v1/indexes/semantic_384d/vectors",
            json={
                "vector": embedding,
                "metadata": doc
            }
        )
        
        if response.status_code == 200:
            vector_id = response.json()['id']
            print(f"  ✓ Inserted document {i} (ID: {vector_id})")
        else:
            print(f"  ✗ Failed to insert document {i}: {response.text}")

def search_similar(model, query, k=3):
    """Search for documents similar to the query."""
    print(f"\n🔍 Searching for: \"{query}\"")
    print(f"   Retrieving top {k} results...\n")
    
    # Generate query embedding
    query_embedding = model.encode(query).tolist()
    
    # Search
    response = requests.post(
        f"{QUARTZ_URL}/api/v1/indexes/semantic_384d/vectors/search",
        json={
            "vector": query_embedding,
            "k": k
        }
    )
    
    if response.status_code != 200:
        print(f"❌ Search failed: {response.text}")
        return
    
    results = response.json()['results']
    
    if not results:
        print("No results found")
        return
    
    print("📊 Results:")
    print("-" * 80)
    for i, result in enumerate(results, 1):
        distance = result['distance']
        metadata = result['metadata']
        similarity = (1 + distance) / 2  # Convert cosine distance to similarity
        
        print(f"\n{i}. Similarity: {similarity:.4f} (distance: {distance:.4f})")
        print(f"   ID: {result['id']}")
        print(f"   Text: {metadata}")
    print("-" * 80)

def main():
    """Main demo function."""
    print("=" * 80)
    print("QuartzDB Vector Search Demo - Semantic Search")
    print("=" * 80)
    
    # Sample documents (QuartzDB features and use cases)
    documents = [
        "QuartzDB is a high-performance distributed edge database with vector search",
        "The database supports key-value storage with automatic persistence",
        "Vector similarity search enables semantic search and recommendations",
        "HNSW algorithm provides fast approximate nearest neighbor search",
        "QuartzDB offers a simple REST API for all database operations",
        "The storage engine uses LSM trees for efficient write performance",
        "Vector embeddings can be generated using OpenAI or Hugging Face models",
        "The database is built with Rust for memory safety and performance",
        "Cosine similarity is ideal for comparing text embeddings",
        "QuartzDB can handle millions of vectors with low latency search",
    ]
    
    # Initialize model
    print("\n🤖 Loading sentence-transformer model...")
    model = SentenceTransformer(MODEL_NAME)
    print(f"✅ Model loaded: {MODEL_NAME}")
    
    # Initialize index
    if not initialize_index():
        return
    
    # Insert documents
    insert_documents(model, documents)
    
    # Example searches
    queries = [
        "How do I search for similar vectors?",
        "What storage system does QuartzDB use?",
        "Can I use machine learning embeddings?",
    ]
    
    for query in queries:
        search_similar(model, query, k=3)
        print()
    
    print("=" * 80)
    print("✨ Demo completed successfully!")
    print("=" * 80)

if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\n\n⚠️  Demo interrupted by user")
    except Exception as e:
        print(f"\n❌ Error: {e}")
        import traceback
        traceback.print_exc()
