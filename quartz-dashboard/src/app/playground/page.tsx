'use client';

import { useState } from 'react';
import Link from 'next/link';
import { 
  Database, 
  Search, 
  Play, 
  Copy, 
  Check,
  AlertCircle,
  Loader2,
  ArrowLeft
} from 'lucide-react';
import { useAuth } from '@/lib/auth';
import api, { VectorSearchResult } from '@/lib/api';

export default function PlaygroundPage() {
  const { apiKey, setApiKey, isAuthenticated } = useAuth();
  const [tempApiKey, setTempApiKey] = useState('');
  const [queryType, setQueryType] = useState<'vector' | 'random'>('random');
  const [vectorInput, setVectorInput] = useState('');
  const [kValue, setKValue] = useState(10);
  const [results, setResults] = useState<VectorSearchResult[] | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [latency, setLatency] = useState<number | null>(null);
  const [copied, setCopied] = useState(false);

  const handleConnect = () => {
    if (tempApiKey.trim()) {
      setApiKey(tempApiKey.trim());
      setError(null);
    }
  };

  const generateRandomVector = () => {
    return Array.from({ length: 384 }, () => Math.random());
  };

  const handleSearch = async () => {
    setLoading(true);
    setError(null);
    setResults(null);
    setLatency(null);

    try {
      let vector: number[];
      
      if (queryType === 'random') {
        vector = generateRandomVector();
      } else {
        try {
          vector = JSON.parse(vectorInput);
          if (!Array.isArray(vector) || !vector.every(v => typeof v === 'number')) {
            throw new Error('Invalid vector format');
          }
        } catch {
          setError('Invalid vector JSON. Please enter a valid array of numbers.');
          setLoading(false);
          return;
        }
      }

      const start = performance.now();
      const response = await api.search(vector, kValue);
      const end = performance.now();
      
      setLatency(Math.round(end - start));
      setResults(response.results);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Search failed');
    } finally {
      setLoading(false);
    }
  };

  const copyResults = () => {
    if (results) {
      navigator.clipboard.writeText(JSON.stringify(results, null, 2));
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  if (!isAuthenticated) {
    return (
      <div className="min-h-screen bg-gray-900 text-white">
        <Header />
        <div className="max-w-md mx-auto mt-20 p-6">
          <div className="bg-gray-800 border border-gray-700 rounded-xl p-6">
            <h2 className="text-xl font-semibold mb-4">Connect to QuartzDB</h2>
            <p className="text-gray-400 text-sm mb-4">
              Enter your API key to start using the playground.
            </p>
            <input
              type="password"
              value={tempApiKey}
              onChange={(e) => setTempApiKey(e.target.value)}
              placeholder="Enter your API key"
              className="w-full bg-gray-700 border border-gray-600 rounded-lg px-4 py-2 mb-4 focus:outline-none focus:border-cyan-500"
              onKeyDown={(e) => e.key === 'Enter' && handleConnect()}
            />
            <button
              onClick={handleConnect}
              className="w-full bg-cyan-500 hover:bg-cyan-600 px-4 py-2 rounded-lg font-medium transition"
            >
              Connect
            </button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-900 text-white">
      <Header />
      
      <main className="max-w-6xl mx-auto px-4 py-8">
        <div className="grid lg:grid-cols-2 gap-8">
          {/* Query Panel */}
          <div className="space-y-6">
            <div className="bg-gray-800 border border-gray-700 rounded-xl p-6">
              <h2 className="text-xl font-semibold mb-4 flex items-center gap-2">
                <Search className="h-5 w-5 text-cyan-400" />
                Vector Search
              </h2>

              {/* Query Type */}
              <div className="mb-4">
                <label className="block text-sm text-gray-400 mb-2">Query Type</label>
                <div className="flex gap-2">
                  <button
                    onClick={() => setQueryType('random')}
                    className={`flex-1 py-2 px-4 rounded-lg border transition ${
                      queryType === 'random' 
                        ? 'bg-cyan-500/20 border-cyan-500 text-cyan-400' 
                        : 'border-gray-600 hover:border-gray-500'
                    }`}
                  >
                    Random Vector
                  </button>
                  <button
                    onClick={() => setQueryType('vector')}
                    className={`flex-1 py-2 px-4 rounded-lg border transition ${
                      queryType === 'vector' 
                        ? 'bg-cyan-500/20 border-cyan-500 text-cyan-400' 
                        : 'border-gray-600 hover:border-gray-500'
                    }`}
                  >
                    Custom Vector
                  </button>
                </div>
              </div>

              {/* Vector Input */}
              {queryType === 'vector' && (
                <div className="mb-4">
                  <label className="block text-sm text-gray-400 mb-2">
                    Vector (384 dimensions)
                  </label>
                  <textarea
                    value={vectorInput}
                    onChange={(e) => setVectorInput(e.target.value)}
                    placeholder="[0.1, 0.2, 0.3, ...]"
                    rows={4}
                    className="w-full bg-gray-700 border border-gray-600 rounded-lg px-4 py-2 font-mono text-sm focus:outline-none focus:border-cyan-500"
                  />
                </div>
              )}

              {/* K Value */}
              <div className="mb-6">
                <label className="block text-sm text-gray-400 mb-2">
                  Number of Results (k): {kValue}
                </label>
                <input
                  type="range"
                  min="1"
                  max="100"
                  value={kValue}
                  onChange={(e) => setKValue(parseInt(e.target.value))}
                  className="w-full accent-cyan-500"
                />
              </div>

              {/* Search Button */}
              <button
                onClick={handleSearch}
                disabled={loading}
                className="w-full bg-cyan-500 hover:bg-cyan-600 disabled:bg-gray-600 px-4 py-3 rounded-lg font-medium transition flex items-center justify-center gap-2"
              >
                {loading ? (
                  <>
                    <Loader2 className="h-5 w-5 animate-spin" />
                    Searching...
                  </>
                ) : (
                  <>
                    <Play className="h-5 w-5" />
                    Execute Search
                  </>
                )}
              </button>

              {/* Error */}
              {error && (
                <div className="mt-4 p-3 bg-red-500/10 border border-red-500/30 rounded-lg flex items-start gap-2">
                  <AlertCircle className="h-5 w-5 text-red-400 flex-shrink-0 mt-0.5" />
                  <span className="text-red-400 text-sm">{error}</span>
                </div>
              )}
            </div>

            {/* Stats */}
            {latency !== null && (
              <div className="bg-gray-800 border border-gray-700 rounded-xl p-4">
                <div className="flex justify-between items-center">
                  <span className="text-gray-400">Latency</span>
                  <span className="text-cyan-400 font-mono">{latency}ms</span>
                </div>
                {results && (
                  <div className="flex justify-between items-center mt-2">
                    <span className="text-gray-400">Results</span>
                    <span className="text-white font-mono">{results.length}</span>
                  </div>
                )}
              </div>
            )}
          </div>

          {/* Results Panel */}
          <div className="bg-gray-800 border border-gray-700 rounded-xl p-6">
            <div className="flex justify-between items-center mb-4">
              <h2 className="text-xl font-semibold">Results</h2>
              {results && results.length > 0 && (
                <button
                  onClick={copyResults}
                  className="text-gray-400 hover:text-white transition flex items-center gap-1 text-sm"
                >
                  {copied ? <Check className="h-4 w-4" /> : <Copy className="h-4 w-4" />}
                  {copied ? 'Copied!' : 'Copy JSON'}
                </button>
              )}
            </div>

            {results === null ? (
              <div className="text-center py-12 text-gray-500">
                <Search className="h-12 w-12 mx-auto mb-4 opacity-50" />
                <p>Run a search to see results</p>
              </div>
            ) : results.length === 0 ? (
              <div className="text-center py-12 text-gray-500">
                <p>No results found</p>
              </div>
            ) : (
              <div className="space-y-3 max-h-[600px] overflow-y-auto">
                {results.map((result, index) => (
                  <ResultCard key={result.id} result={result} rank={index + 1} />
                ))}
              </div>
            )}
          </div>
        </div>
      </main>
    </div>
  );
}

function Header() {
  return (
    <header className="border-b border-gray-700">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex justify-between items-center py-4">
          <div className="flex items-center gap-4">
            <Link href="/" className="flex items-center gap-2 text-gray-400 hover:text-white transition">
              <ArrowLeft className="h-4 w-4" />
              Back
            </Link>
            <div className="flex items-center gap-2">
              <Database className="h-6 w-6 text-cyan-400" />
              <span className="text-xl font-bold">Playground</span>
            </div>
          </div>
          <Link 
            href="/dashboard" 
            className="text-gray-300 hover:text-white transition"
          >
            Dashboard
          </Link>
        </div>
      </div>
    </header>
  );
}

function ResultCard({ result, rank }: { result: VectorSearchResult; rank: number }) {
  const [expanded, setExpanded] = useState(false);
  
  return (
    <div 
      className="bg-gray-700/50 border border-gray-600 rounded-lg p-4 cursor-pointer hover:border-gray-500 transition"
      onClick={() => setExpanded(!expanded)}
    >
      <div className="flex justify-between items-start">
        <div className="flex items-center gap-3">
          <span className="text-xs bg-gray-600 px-2 py-1 rounded">#{rank}</span>
          <span className="font-mono text-sm">{result.id}</span>
        </div>
        <div className="text-right">
          <div className="text-cyan-400 font-mono text-sm">
            {(result.score * 100).toFixed(1)}%
          </div>
          <div className="text-gray-500 text-xs">
            dist: {result.distance.toFixed(4)}
          </div>
        </div>
      </div>
      
      {expanded && result.metadata && (
        <div className="mt-3 pt-3 border-t border-gray-600">
          <pre className="text-xs text-gray-400 overflow-x-auto">
            {JSON.stringify(result.metadata, null, 2)}
          </pre>
        </div>
      )}
    </div>
  );
}
