#!/usr/bin/env python3
"""
QuartzDB Vector Search - Simple Example (No Dependencies)

This example demonstrates the vector search API with manually created vectors.
No external dependencies required - just Python 3 and requests.

Requirements:
    pip install requests  # (usually pre-installed)

Usage:
    # Start the server in another terminal:
    cargo run -p quartz-server
    
    # Then run this script:
    python3 quartz-server/examples/simple_vector_demo.py
"""

import json
try:
    import requests
except ImportError:
    print("❌ Error: 'requests' library not found")
    print("Install it with: pip3 install requests")
    exit(1)

# Configuration
QUARTZ_URL = "http://localhost:3000"

def check_server():
    """Check if the server is running."""
    try:
        response = requests.get(f"{QUARTZ_URL}/api/v1/health", timeout=2)
        if response.status_code == 200:
            print("✅ Server is running")
            return True
    except requests.exceptions.ConnectionError:
        print("❌ Error: Cannot connect to QuartzDB server")
        print("   Please start the server first:")
        print("   cargo run -p quartz-server")
        return False
    except Exception as e:
        print(f"❌ Error checking server: {e}")
        return False

def initialize_index():
    """Initialize a 3D vector index for this demo."""
    print("\n🔧 Initializing 3D vector index...")
    
    response = requests.post(
        f"{QUARTZ_URL}/api/v1/indexes/simple_3d",
        json={
            "dimension": 3,
            "metric": "cosine"
        }
    )
    
    if response.status_code == 200:
        data = response.json()
        print(f"✅ Index initialized: {data['dimension']}D, metric={data['metric']}")
        return True
    else:
        print(f"❌ Failed to initialize index: {response.text}")
        return False

def insert_sample_vectors():
    """Insert sample 3D vectors representing different concepts."""
    print("\n📝 Inserting sample vectors...")
    
    # Sample vectors (3D for simplicity)
    # These represent different "concepts" in 3D space
    samples = [
        {
            "vector": [1.0, 0.0, 0.0],
            "metadata": "Concept A: Technology (pure x-axis)"
        },
        {
            "vector": [0.0, 1.0, 0.0],
            "metadata": "Concept B: Science (pure y-axis)"
        },
        {
            "vector": [0.0, 0.0, 1.0],
            "metadata": "Concept C: Art (pure z-axis)"
        },
        {
            "vector": [0.7, 0.7, 0.0],
            "metadata": "Concept D: Tech + Science hybrid"
        },
        {
            "vector": [0.5, 0.0, 0.5],
            "metadata": "Concept E: Tech + Art hybrid"
        },
        {
            "vector": [0.0, 0.6, 0.6],
            "metadata": "Concept F: Science + Art hybrid"
        },
        {
            "vector": [0.33, 0.33, 0.33],
            "metadata": "Concept G: Balanced mix of all"
        },
    ]
    
    vector_ids = []
    for sample in samples:
        response = requests.post(
            f"{QUARTZ_URL}/api/v1/indexes/simple_3d/vectors",
            json=sample
        )
        
        if response.status_code == 200:
            vector_id = response.json()['id']
            vector_ids.append(vector_id)
            print(f"  ✓ Inserted vector {vector_id}: {sample['metadata']}")
        else:
            print(f"  ✗ Failed to insert: {response.text}")
    
    return vector_ids

def search_similar(query_vector, query_name, k=3):
    """Search for vectors similar to the query."""
    print(f"\n🔍 Searching for vectors similar to: {query_name}")
    print(f"   Query vector: {query_vector}")
    print(f"   Retrieving top {k} results...\n")
    
    response = requests.post(
        f"{QUARTZ_URL}/api/v1/indexes/simple_3d/vectors/search",
        json={
            "vector": query_vector,
            "k": k
        }
    )
    
    if response.status_code != 200:
        print(f"❌ Search failed: {response.text}")
        return
    
    results = response.json()['results']
    
    if not results:
        print("   No results found")
        return
    
    print("   📊 Results:")
    print("   " + "-" * 70)
    for i, result in enumerate(results, 1):
        distance = result['distance']
        metadata = result.get('metadata', 'No metadata')
        vector = result['vector']
        
        # For cosine similarity, distance is actually the similarity score
        # Higher is better (1.0 = identical)
        similarity_pct = distance * 100
        
        print(f"\n   {i}. Similarity: {similarity_pct:.1f}%")
        print(f"      ID: {result['id']}")
        print(f"      Vector: {vector}")
        print(f"      Info: {metadata}")
    print("   " + "-" * 70)

def retrieve_vector(vector_id: int) -> dict:
    """Retrieve a vector by its ID."""
    response = requests.get(f"{QUARTZ_URL}/api/v1/indexes/simple_3d/vectors/{vector_id}")
    response.raise_for_status()
    return response.json()

def delete_vector(vector_id):
    """Delete a vector by ID."""
    print(f"\n🗑️  Deleting vector {vector_id}...")
    
    response = requests.delete(f"{QUARTZ_URL}/api/v1/indexes/simple_3d/vectors/{vector_id}")
    
    if response.status_code == 200:
        print(f"✅ Vector {vector_id} deleted successfully")
        return True
    else:
        print(f"❌ Failed to delete: {response.text}")
        return False

def main():
    """Run the complete vector search demo."""
    print("=" * 80)
    print("QuartzDB Vector Search Demo - Simple Example")
    print("=" * 80)
    
    # Check server
    if not check_server():
        return
    
    # Initialize index
    if not initialize_index():
        return
    
    # Insert sample vectors
    vector_ids = insert_sample_vectors()
    if not vector_ids:
        print("\n❌ No vectors were inserted")
        return
    
    print(f"\n✅ Inserted {len(vector_ids)} vectors")
    
    # Example 1: Search for technology-focused vectors
    search_similar(
        query_vector=[0.9, 0.1, 0.0],
        query_name="Technology-focused (90% tech, 10% science)",
        k=3
    )
    
    # Example 2: Search for science-focused vectors
    search_similar(
        query_vector=[0.1, 0.9, 0.0],
        query_name="Science-focused (10% tech, 90% science)",
        k=3
    )
    
    # Example 3: Search for balanced vectors
    search_similar(
        query_vector=[0.4, 0.3, 0.3],
        query_name="Balanced (40% tech, 30% science, 30% art)",
        k=3
    )
    
    # Demonstrate retrieve
    if vector_ids:
        retrieve_vector(vector_ids[0])
    
    # Demonstrate delete
    if len(vector_ids) > 1:
        delete_vector(vector_ids[-1])
        print("\n🔍 Verifying deletion...")
        try:
            retrieve_vector(vector_ids[-1])
            print("❌ Error: Vector should have been deleted!")
        except Exception as e:
            if "404" in str(e):
                print("✅ Confirmed: Vector successfully deleted")
            else:
                raise
    
    print("\n" + "=" * 80)
    print("✨ Demo completed successfully!")
    print("=" * 80)
    print("\n💡 Next steps:")
    print("   - Modify the vectors and queries to experiment")
    print("   - Try different distance metrics (euclidean, dotproduct)")
    print("   - Check out semantic_search_demo.py for real embeddings")
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
