'use client';

import Link from 'next/link';
import { useState } from 'react';
import { Database, Copy, Check, AlertTriangle } from 'lucide-react';
import api from '@/lib/api';

export default function SignupPage() {
  const [tenantId, setTenantId] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [result, setResult] = useState<{
    api_key: string;
    tenant_id: string;
    plan: string;
    vector_limit: number;
    query_limit_per_month: number;
  } | null>(null);
  const [copied, setCopied] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    setLoading(true);

    try {
      const res = await api.signup(tenantId.trim());
      setResult({
        api_key: res.api_key,
        tenant_id: res.tenant_id,
        plan: res.plan,
        vector_limit: res.vector_limit,
        query_limit_per_month: res.query_limit_per_month,
      });
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Signup failed. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  const copyApiKey = async () => {
    if (!result) return;
    await navigator.clipboard.writeText(result.api_key);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="min-h-screen bg-gradient-to-b from-gray-900 via-gray-800 to-gray-900 text-white">
      {/* Header */}
      <header className="border-b border-gray-700">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center py-4">
            <Link href="/" className="flex items-center gap-2">
              <Database className="h-8 w-8 text-cyan-400" />
              <span className="text-2xl font-bold">QuartzDB</span>
            </Link>
            <nav className="flex items-center gap-6">
              <Link href="/pricing" className="text-gray-300 hover:text-white transition">
                Pricing
              </Link>
              <Link href="/docs" className="text-gray-300 hover:text-white transition">
                Docs
              </Link>
            </nav>
          </div>
        </div>
      </header>

      <div className="max-w-lg mx-auto px-4 py-20">
        {!result ? (
          <>
            <div className="text-center mb-8">
              <h1 className="text-4xl font-bold mb-3">Get Started Free</h1>
              <p className="text-gray-400">
                Create a project and get your API key in seconds.
              </p>
            </div>

            <form onSubmit={handleSubmit} className="bg-gray-800 border border-gray-700 rounded-xl p-6 space-y-5">
              <div>
                <label htmlFor="tenant_id" className="block text-sm font-medium text-gray-300 mb-2">
                  Project Name
                </label>
                <input
                  id="tenant_id"
                  type="text"
                  value={tenantId}
                  onChange={(e) => setTenantId(e.target.value)}
                  placeholder="my-awesome-project"
                  required
                  className="w-full bg-gray-900 border border-gray-600 rounded-lg px-4 py-3 text-white placeholder-gray-500 focus:outline-none focus:border-cyan-500 focus:ring-1 focus:ring-cyan-500 transition"
                />
                <p className="mt-1 text-xs text-gray-500">
                  Letters, numbers, and hyphens only. This will be your tenant ID.
                </p>
              </div>

              {error && (
                <div className="bg-red-500/10 border border-red-500/30 rounded-lg p-3 text-red-400 text-sm">
                  {error}
                </div>
              )}

              <button
                type="submit"
                disabled={loading || !tenantId.trim()}
                className="w-full bg-cyan-500 hover:bg-cyan-600 disabled:bg-gray-600 disabled:cursor-not-allowed px-6 py-3 rounded-lg font-medium transition"
              >
                {loading ? 'Creating Account...' : 'Create Free Account'}
              </button>
            </form>

            <div className="mt-6 bg-gray-800/50 border border-gray-700 rounded-xl p-5">
              <h3 className="text-sm font-semibold text-gray-300 mb-3">Free Tier Includes</h3>
              <ul className="space-y-2 text-sm text-gray-400">
                <li className="flex items-center gap-2">
                  <span className="text-cyan-400">✓</span> 1,000 vectors
                </li>
                <li className="flex items-center gap-2">
                  <span className="text-cyan-400">✓</span> 10,000 queries/month
                </li>
                <li className="flex items-center gap-2">
                  <span className="text-cyan-400">✓</span> HNSW vector search
                </li>
                <li className="flex items-center gap-2">
                  <span className="text-cyan-400">✓</span> Community support
                </li>
              </ul>
            </div>
          </>
        ) : (
          <div className="space-y-6">
            <div className="text-center mb-2">
              <h1 className="text-4xl font-bold mb-3">You&apos;re All Set!</h1>
              <p className="text-gray-400">
                Project <span className="text-cyan-400 font-mono">{result.tenant_id}</span> created on the{' '}
                <span className="text-cyan-400 capitalize">{result.plan}</span> plan.
              </p>
            </div>

            {/* API Key Display */}
            <div className="bg-gray-800 border border-cyan-500/50 rounded-xl p-6 space-y-4">
              <div className="flex items-center gap-2 text-yellow-400 text-sm font-medium">
                <AlertTriangle className="h-4 w-4" />
                Save your API key now — it won&apos;t be shown again!
              </div>

              <div className="relative">
                <label className="block text-sm font-medium text-gray-300 mb-2">Your API Key</label>
                <div className="flex items-center gap-2">
                  <code className="flex-1 bg-gray-900 border border-gray-600 rounded-lg px-4 py-3 text-cyan-400 font-mono text-sm break-all select-all">
                    {result.api_key}
                  </code>
                  <button
                    onClick={copyApiKey}
                    className="shrink-0 bg-gray-700 hover:bg-gray-600 p-3 rounded-lg transition"
                    title="Copy API key"
                  >
                    {copied ? <Check className="h-5 w-5 text-green-400" /> : <Copy className="h-5 w-5" />}
                  </button>
                </div>
              </div>
            </div>

            {/* Plan Limits */}
            <div className="bg-gray-800 border border-gray-700 rounded-xl p-5">
              <h3 className="text-sm font-semibold text-gray-300 mb-3">Your Plan Limits</h3>
              <div className="grid grid-cols-2 gap-4 text-sm">
                <div>
                  <p className="text-gray-500">Vectors</p>
                  <p className="text-lg font-semibold">{result.vector_limit.toLocaleString()}</p>
                </div>
                <div>
                  <p className="text-gray-500">Queries/month</p>
                  <p className="text-lg font-semibold">{result.query_limit_per_month.toLocaleString()}</p>
                </div>
              </div>
            </div>

            {/* Next Steps */}
            <div className="flex flex-col sm:flex-row gap-3">
              <Link
                href="/docs"
                className="flex-1 text-center border border-gray-600 hover:border-gray-500 px-5 py-3 rounded-lg font-medium transition"
              >
                Read the Docs
              </Link>
              <Link
                href="/dashboard"
                className="flex-1 text-center bg-cyan-500 hover:bg-cyan-600 px-5 py-3 rounded-lg font-medium transition"
              >
                Go to Dashboard
              </Link>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
