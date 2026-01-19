'use client';

import { useState, useEffect } from 'react';
import Link from 'next/link';
import { 
  Database,
  BarChart3,
  Key,
  Search,
  Settings,
  LogOut,
  Activity,
  HardDrive,
  Cpu,
  AlertCircle,
  RefreshCw,
  Loader2
} from 'lucide-react';
import { useAuth } from '@/lib/auth';
import api, { StatsResponse, HealthResponse } from '@/lib/api';

export default function DashboardPage() {
  const { apiKey, setApiKey, isAuthenticated, logout } = useAuth();
  const [tempApiKey, setTempApiKey] = useState('');
  const [stats, setStats] = useState<StatsResponse | null>(null);
  const [health, setHealth] = useState<HealthResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<'overview' | 'apikeys' | 'settings'>('overview');

  const loadData = async () => {
    setLoading(true);
    setError(null);
    try {
      const [statsRes, healthRes] = await Promise.all([
        api.stats(),
        api.health()
      ]);
      setStats(statsRes);
      setHealth(healthRes);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load data');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (isAuthenticated) {
      loadData();
    }
  }, [isAuthenticated]);

  const handleConnect = () => {
    if (tempApiKey.trim()) {
      setApiKey(tempApiKey.trim());
      setError(null);
    }
  };

  if (!isAuthenticated) {
    return (
      <div className="min-h-screen bg-gray-900 text-white flex items-center justify-center">
        <div className="max-w-md w-full mx-4">
          <div className="text-center mb-8">
            <Database className="h-16 w-16 text-cyan-400 mx-auto mb-4" />
            <h1 className="text-3xl font-bold">QuartzDB Dashboard</h1>
            <p className="text-gray-400 mt-2">Sign in with your API key</p>
          </div>
          
          <div className="bg-gray-800 border border-gray-700 rounded-xl p-6">
            {error && (
              <div className="mb-4 p-3 bg-red-500/10 border border-red-500/30 rounded-lg flex items-start gap-2">
                <AlertCircle className="h-5 w-5 text-red-400 flex-shrink-0" />
                <span className="text-red-400 text-sm">{error}</span>
              </div>
            )}
            
            <input
              type="password"
              value={tempApiKey}
              onChange={(e) => setTempApiKey(e.target.value)}
              placeholder="Enter your API key"
              className="w-full bg-gray-700 border border-gray-600 rounded-lg px-4 py-3 mb-4 focus:outline-none focus:border-cyan-500"
              onKeyDown={(e) => e.key === 'Enter' && handleConnect()}
            />
            <button
              onClick={handleConnect}
              className="w-full bg-cyan-500 hover:bg-cyan-600 px-4 py-3 rounded-lg font-medium transition"
            >
              Sign In
            </button>
            
            <p className="text-gray-500 text-sm text-center mt-4">
              Don&apos;t have an API key?{' '}
              <Link href="/docs" className="text-cyan-400 hover:underline">
                Learn how to get one
              </Link>
            </p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-900 text-white flex">
      {/* Sidebar */}
      <aside className="w-64 bg-gray-800 border-r border-gray-700 flex flex-col">
        <div className="p-4 border-b border-gray-700">
          <Link href="/" className="flex items-center gap-2">
            <Database className="h-8 w-8 text-cyan-400" />
            <span className="text-xl font-bold">QuartzDB</span>
          </Link>
        </div>
        
        <nav className="flex-1 p-4 space-y-2">
          <SidebarItem 
            icon={<BarChart3 />} 
            label="Overview" 
            active={activeTab === 'overview'}
            onClick={() => setActiveTab('overview')}
          />
          <SidebarItem 
            icon={<Key />} 
            label="API Keys" 
            active={activeTab === 'apikeys'}
            onClick={() => setActiveTab('apikeys')}
          />
          <SidebarItem 
            icon={<Settings />} 
            label="Settings" 
            active={activeTab === 'settings'}
            onClick={() => setActiveTab('settings')}
          />
          
          <div className="pt-4 border-t border-gray-700 mt-4">
            <Link href="/playground">
              <SidebarItem icon={<Search />} label="Playground" />
            </Link>
          </div>
        </nav>
        
        <div className="p-4 border-t border-gray-700">
          <button 
            onClick={logout}
            className="w-full flex items-center gap-2 text-gray-400 hover:text-white transition px-3 py-2 rounded-lg hover:bg-gray-700"
          >
            <LogOut className="h-5 w-5" />
            Sign Out
          </button>
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 overflow-auto">
        <header className="border-b border-gray-700 px-8 py-4 flex justify-between items-center">
          <h1 className="text-2xl font-bold">
            {activeTab === 'overview' && 'Overview'}
            {activeTab === 'apikeys' && 'API Keys'}
            {activeTab === 'settings' && 'Settings'}
          </h1>
          <button
            onClick={loadData}
            disabled={loading}
            className="flex items-center gap-2 text-gray-400 hover:text-white transition"
          >
            <RefreshCw className={`h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
            Refresh
          </button>
        </header>

        <div className="p-8">
          {activeTab === 'overview' && (
            <OverviewTab stats={stats} health={health} loading={loading} error={error} />
          )}
          {activeTab === 'apikeys' && (
            <ApiKeysTab apiKey={apiKey} />
          )}
          {activeTab === 'settings' && (
            <SettingsTab />
          )}
        </div>
      </main>
    </div>
  );
}

function SidebarItem({ 
  icon, 
  label, 
  active = false, 
  onClick 
}: { 
  icon: React.ReactNode; 
  label: string; 
  active?: boolean;
  onClick?: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg transition ${
        active 
          ? 'bg-cyan-500/20 text-cyan-400' 
          : 'text-gray-400 hover:text-white hover:bg-gray-700'
      }`}
    >
      <span className="h-5 w-5">{icon}</span>
      {label}
    </button>
  );
}

function OverviewTab({ 
  stats, 
  health, 
  loading, 
  error 
}: { 
  stats: StatsResponse | null;
  health: HealthResponse | null;
  loading: boolean;
  error: string | null;
}) {
  if (loading && !stats) {
    return (
      <div className="flex items-center justify-center py-20">
        <Loader2 className="h-8 w-8 animate-spin text-cyan-400" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-4 bg-red-500/10 border border-red-500/30 rounded-lg flex items-start gap-2">
        <AlertCircle className="h-5 w-5 text-red-400 flex-shrink-0" />
        <span className="text-red-400">{error}</span>
      </div>
    );
  }

  return (
    <div className="space-y-8">
      {/* Status Cards */}
      <div className="grid md:grid-cols-4 gap-4">
        <StatCard 
          icon={<Activity />}
          label="Status"
          value={health?.status === 'healthy' ? 'Healthy' : 'Unknown'}
          color={health?.status === 'healthy' ? 'green' : 'yellow'}
        />
        <StatCard 
          icon={<Database />}
          label="Total Vectors"
          value={stats?.num_vectors?.toLocaleString() ?? '-'}
        />
        <StatCard 
          icon={<HardDrive />}
          label="Active Vectors"
          value={stats?.num_active?.toLocaleString() ?? '-'}
        />
        <StatCard 
          icon={<Cpu />}
          label="Dimensions"
          value={stats?.dimension?.toString() ?? '-'}
        />
      </div>

      {/* Index Details */}
      <div className="bg-gray-800 border border-gray-700 rounded-xl p-6">
        <h2 className="text-lg font-semibold mb-4">Index Details</h2>
        <div className="grid md:grid-cols-2 gap-6">
          <div>
            <table className="w-full">
              <tbody className="divide-y divide-gray-700">
                <TableRow label="Algorithm" value={stats?.algorithm ?? '-'} />
                <TableRow label="Dimension" value={stats?.dimension?.toString() ?? '-'} />
                <TableRow label="Total Vectors" value={stats?.num_vectors?.toLocaleString() ?? '-'} />
                <TableRow label="Active Vectors" value={stats?.num_active?.toLocaleString() ?? '-'} />
                <TableRow label="Deleted Vectors" value={stats?.num_deleted?.toLocaleString() ?? '-'} />
                <TableRow label="Deletion Ratio" value={`${stats?.deletion_ratio_percent ?? '0'}%`} />
              </tbody>
            </table>
          </div>
          <div>
            <h3 className="text-sm text-gray-400 mb-2">Health Recommendation</h3>
            <div className={`p-4 rounded-lg ${
              stats?.recommendation?.includes('Healthy') 
                ? 'bg-green-500/10 border border-green-500/30' 
                : 'bg-yellow-500/10 border border-yellow-500/30'
            }`}>
              <p className={stats?.recommendation?.includes('Healthy') ? 'text-green-400' : 'text-yellow-400'}>
                {stats?.recommendation ?? 'Loading...'}
              </p>
            </div>
            
            <h3 className="text-sm text-gray-400 mt-4 mb-2">Service Info</h3>
            <table className="w-full">
              <tbody className="divide-y divide-gray-700">
                <TableRow label="Service" value={health?.service ?? '-'} />
                <TableRow label="Version" value={health?.version ?? '-'} />
                <TableRow label="Uptime" value={health ? `${Math.floor(health.uptime_seconds / 60)}m` : '-'} />
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </div>
  );
}

function ApiKeysTab({ apiKey }: { apiKey: string }) {
  const [copied, setCopied] = useState(false);

  const copyKey = () => {
    navigator.clipboard.writeText(apiKey);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const maskedKey = apiKey.slice(0, 8) + '...' + apiKey.slice(-8);

  return (
    <div className="space-y-6">
      <div className="bg-gray-800 border border-gray-700 rounded-xl p-6">
        <h2 className="text-lg font-semibold mb-4">Current API Key</h2>
        <div className="flex items-center gap-4">
          <code className="flex-1 bg-gray-700 px-4 py-3 rounded-lg font-mono text-sm">
            {maskedKey}
          </code>
          <button
            onClick={copyKey}
            className="bg-cyan-500 hover:bg-cyan-600 px-4 py-3 rounded-lg font-medium transition"
          >
            {copied ? 'Copied!' : 'Copy Full Key'}
          </button>
        </div>
        <p className="text-gray-500 text-sm mt-4">
          Keep this key secure. Do not share it publicly or commit it to version control.
        </p>
      </div>

      <div className="bg-gray-800 border border-gray-700 rounded-xl p-6">
        <h2 className="text-lg font-semibold mb-4">Usage Examples</h2>
        
        <div className="space-y-4">
          <CodeExample 
            title="cURL"
            code={`curl -X POST https://api.quartzdb.io/api/vector/search \\
  -H "Content-Type: application/json" \\
  -H "X-API-Key: ${maskedKey}" \\
  -d '{"vector": [...], "k": 10}'`}
          />
          
          <CodeExample 
            title="Python"
            code={`import requests

response = requests.post(
    "https://api.quartzdb.io/api/vector/search",
    headers={"X-API-Key": "${maskedKey}"},
    json={"vector": [...], "k": 10}
)
results = response.json()`}
          />
          
          <CodeExample 
            title="JavaScript"
            code={`const response = await fetch(
  "https://api.quartzdb.io/api/vector/search",
  {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "X-API-Key": "${maskedKey}"
    },
    body: JSON.stringify({ vector: [...], k: 10 })
  }
);
const results = await response.json();`}
          />
        </div>
      </div>
    </div>
  );
}

function SettingsTab() {
  return (
    <div className="space-y-6">
      <div className="bg-gray-800 border border-gray-700 rounded-xl p-6">
        <h2 className="text-lg font-semibold mb-4">Index Configuration</h2>
        <p className="text-gray-400">
          Index configuration is currently fixed at deployment time.
          Contact support to modify these settings.
        </p>
        
        <div className="mt-4 grid md:grid-cols-2 gap-4">
          <div className="bg-gray-700/50 rounded-lg p-4">
            <div className="text-sm text-gray-400">Dimensions</div>
            <div className="text-xl font-semibold">384</div>
          </div>
          <div className="bg-gray-700/50 rounded-lg p-4">
            <div className="text-sm text-gray-400">Distance Metric</div>
            <div className="text-xl font-semibold">Cosine</div>
          </div>
          <div className="bg-gray-700/50 rounded-lg p-4">
            <div className="text-sm text-gray-400">M (connections)</div>
            <div className="text-xl font-semibold">16</div>
          </div>
          <div className="bg-gray-700/50 rounded-lg p-4">
            <div className="text-sm text-gray-400">ef_construction</div>
            <div className="text-xl font-semibold">200</div>
          </div>
        </div>
      </div>

      <div className="bg-gray-800 border border-gray-700 rounded-xl p-6">
        <h2 className="text-lg font-semibold mb-4">API Endpoint</h2>
        <code className="block bg-gray-700 px-4 py-3 rounded-lg font-mono text-sm">
          https://api.quartzdb.io
        </code>
      </div>
    </div>
  );
}

function StatCard({ 
  icon, 
  label, 
  value, 
  color = 'cyan' 
}: { 
  icon: React.ReactNode; 
  label: string; 
  value: string;
  color?: 'cyan' | 'green' | 'yellow' | 'red';
}) {
  const colorClasses = {
    cyan: 'text-cyan-400',
    green: 'text-green-400',
    yellow: 'text-yellow-400',
    red: 'text-red-400'
  };

  return (
    <div className="bg-gray-800 border border-gray-700 rounded-xl p-4">
      <div className={`${colorClasses[color]} mb-2`}>{icon}</div>
      <div className="text-sm text-gray-400">{label}</div>
      <div className="text-2xl font-bold">{value}</div>
    </div>
  );
}

function TableRow({ label, value }: { label: string; value: string }) {
  return (
    <tr>
      <td className="py-2 text-gray-400">{label}</td>
      <td className="py-2 text-right font-mono">{value}</td>
    </tr>
  );
}

function CodeExample({ title, code }: { title: string; code: string }) {
  const [copied, setCopied] = useState(false);

  const copy = () => {
    navigator.clipboard.writeText(code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="bg-gray-700/50 rounded-lg overflow-hidden">
      <div className="flex justify-between items-center px-4 py-2 border-b border-gray-600">
        <span className="text-sm font-medium">{title}</span>
        <button
          onClick={copy}
          className="text-xs text-gray-400 hover:text-white transition"
        >
          {copied ? 'Copied!' : 'Copy'}
        </button>
      </div>
      <pre className="p-4 overflow-x-auto text-sm">
        <code>{code}</code>
      </pre>
    </div>
  );
}
