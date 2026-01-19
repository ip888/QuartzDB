'use client';

import Link from 'next/link';
import { useState, useEffect } from 'react';
import { 
  Database, 
  Zap, 
  Globe, 
  Search, 
  Shield, 
  BarChart3,
  ArrowRight,
  Code,
  Play
} from 'lucide-react';
import api from '@/lib/api';

export default function HomePage() {
  const [healthStatus, setHealthStatus] = useState<'loading' | 'healthy' | 'error'>('loading');

  useEffect(() => {
    api.health()
      .then(() => setHealthStatus('healthy'))
      .catch(() => setHealthStatus('error'));
  }, []);

  return (
    <div className="min-h-screen bg-gradient-to-b from-gray-900 via-gray-800 to-gray-900 text-white">
      {/* Header */}
      <header className="border-b border-gray-700">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center py-4">
            <div className="flex items-center gap-2">
              <Database className="h-8 w-8 text-cyan-400" />
              <span className="text-2xl font-bold">QuartzDB</span>
            </div>
            <nav className="flex items-center gap-6">
              <Link href="/docs" className="text-gray-300 hover:text-white transition">
                Docs
              </Link>
              <Link href="/playground" className="text-gray-300 hover:text-white transition">
                Playground
              </Link>
              <Link 
                href="/dashboard" 
                className="bg-cyan-500 hover:bg-cyan-600 px-4 py-2 rounded-lg font-medium transition"
              >
                Dashboard
              </Link>
            </nav>
          </div>
        </div>
      </header>

      {/* Hero Section */}
      <section className="py-20 px-4">
        <div className="max-w-4xl mx-auto text-center">
          <div className="inline-flex items-center gap-2 bg-cyan-500/10 border border-cyan-500/30 rounded-full px-4 py-1 mb-6">
            <span className={`w-2 h-2 rounded-full ${healthStatus === 'healthy' ? 'bg-green-400' : healthStatus === 'error' ? 'bg-red-400' : 'bg-yellow-400'} animate-pulse`}></span>
            <span className="text-cyan-400 text-sm">
              {healthStatus === 'healthy' ? 'API Online' : healthStatus === 'error' ? 'API Offline' : 'Checking...'}
            </span>
          </div>
          
          <h1 className="text-5xl sm:text-6xl font-bold mb-6 bg-gradient-to-r from-white via-cyan-200 to-cyan-400 bg-clip-text text-transparent">
            Serverless Vector Database
          </h1>
          
          <p className="text-xl text-gray-400 mb-8 max-w-2xl mx-auto">
            High-performance semantic search built on Cloudflare Workers. 
            Zero cold starts, global edge deployment, sub-millisecond latency.
          </p>
          
          <div className="flex flex-col sm:flex-row gap-4 justify-center">
            <Link 
              href="/playground" 
              className="inline-flex items-center justify-center gap-2 bg-cyan-500 hover:bg-cyan-600 px-6 py-3 rounded-lg font-medium transition"
            >
              <Play className="h-5 w-5" />
              Try Playground
            </Link>
            <Link 
              href="/docs" 
              className="inline-flex items-center justify-center gap-2 border border-gray-600 hover:border-gray-500 px-6 py-3 rounded-lg font-medium transition"
            >
              <Code className="h-5 w-5" />
              View API Docs
            </Link>
          </div>
        </div>
      </section>

      {/* Features */}
      <section className="py-20 px-4 bg-gray-800/50">
        <div className="max-w-6xl mx-auto">
          <h2 className="text-3xl font-bold text-center mb-12">Why QuartzDB?</h2>
          
          <div className="grid md:grid-cols-3 gap-8">
            <FeatureCard 
              icon={<Zap className="h-8 w-8" />}
              title="Zero Cold Starts"
              description="Built on Cloudflare Workers with Durable Objects. Your database is always warm and ready."
            />
            <FeatureCard 
              icon={<Globe className="h-8 w-8" />}
              title="Global Edge"
              description="Deployed to 300+ data centers worldwide. Sub-millisecond latency from anywhere."
            />
            <FeatureCard 
              icon={<Search className="h-8 w-8" />}
              title="HNSW Algorithm"
              description="State-of-the-art approximate nearest neighbor search. Fast and accurate."
            />
            <FeatureCard 
              icon={<Shield className="h-8 w-8" />}
              title="Secure by Default"
              description="API key authentication, rate limiting, and input validation built-in."
            />
            <FeatureCard 
              icon={<BarChart3 className="h-8 w-8" />}
              title="Real-time Stats"
              description="Monitor your index health, query performance, and usage metrics."
            />
            <FeatureCard 
              icon={<Database className="h-8 w-8" />}
              title="Serverless Scale"
              description="From 0 to millions of vectors. Pay only for what you use."
            />
          </div>
        </div>
      </section>

      {/* Quick Start */}
      <section className="py-20 px-4">
        <div className="max-w-4xl mx-auto">
          <h2 className="text-3xl font-bold text-center mb-12">Quick Start</h2>
          
          <div className="space-y-6">
            <StepCard 
              step={1}
              title="Get your API Key"
              code={`# Go to Dashboard > API Keys
# Create a new key`}
            />
            <StepCard 
              step={2}
              title="Insert a Vector"
              code={`curl -X POST https://api.quartzdb.io/api/vector/insert \\
  -H "Content-Type: application/json" \\
  -H "X-API-Key: YOUR_API_KEY" \\
  -d '{"id": "doc1", "vector": [0.1, 0.2, ...], "metadata": {"title": "Hello"}}'`}
            />
            <StepCard 
              step={3}
              title="Search Similar Vectors"
              code={`curl -X POST https://api.quartzdb.io/api/vector/search \\
  -H "Content-Type: application/json" \\
  -H "X-API-Key: YOUR_API_KEY" \\
  -d '{"vector": [0.1, 0.2, ...], "k": 10}'`}
            />
          </div>
          
          <div className="text-center mt-12">
            <Link 
              href="/docs" 
              className="inline-flex items-center gap-2 text-cyan-400 hover:text-cyan-300 transition"
            >
              View full documentation
              <ArrowRight className="h-4 w-4" />
            </Link>
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer className="border-t border-gray-700 py-8 px-4">
        <div className="max-w-7xl mx-auto flex flex-col sm:flex-row justify-between items-center gap-4">
          <div className="flex items-center gap-2">
            <Database className="h-5 w-5 text-cyan-400" />
            <span className="font-semibold">QuartzDB</span>
          </div>
          <p className="text-gray-500 text-sm">
            Built with Cloudflare Workers & Rust
          </p>
        </div>
      </footer>
    </div>
  );
}

function FeatureCard({ icon, title, description }: { icon: React.ReactNode; title: string; description: string }) {
  return (
    <div className="bg-gray-800 border border-gray-700 rounded-xl p-6 hover:border-cyan-500/50 transition">
      <div className="text-cyan-400 mb-4">{icon}</div>
      <h3 className="text-xl font-semibold mb-2">{title}</h3>
      <p className="text-gray-400">{description}</p>
    </div>
  );
}

function StepCard({ step, title, code }: { step: number; title: string; code: string }) {
  return (
    <div className="bg-gray-800 border border-gray-700 rounded-xl overflow-hidden">
      <div className="flex items-center gap-3 px-4 py-3 border-b border-gray-700">
        <span className="flex items-center justify-center w-6 h-6 bg-cyan-500 rounded-full text-sm font-bold">
          {step}
        </span>
        <span className="font-medium">{title}</span>
      </div>
      <pre className="p-4 overflow-x-auto text-sm text-gray-300">
        <code>{code}</code>
      </pre>
    </div>
  );
}
