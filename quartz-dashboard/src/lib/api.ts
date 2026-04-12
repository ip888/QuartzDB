// QuartzDB API Client

const API_BASE = process.env.NEXT_PUBLIC_API_URL || 'https://api.quartzdb.io';

export interface VectorSearchResult {
  id: string;
  score: number;
  distance: number;
  metadata?: Record<string, unknown>;
}

export interface SearchResponse {
  success: boolean;
  results: VectorSearchResult[];
  count: number;
  algorithm: string;
}

export interface StatsResponse {
  success: boolean;
  num_vectors: number;
  num_active: number;
  num_deleted: number;
  dimension: number;
  algorithm: string;
  deletion_ratio_percent: string;
  recommendation: string;
}

export interface InsertResponse {
  success: boolean;
  id: string;
  message: string;
}

export interface HealthResponse {
  status: string;
  service: string;
  version: string;
  uptime_seconds: number;
  checks: {
    storage: string;
    vector_index: string;
  };
}

export interface SignupResponse {
  success: boolean;
  tenant_id: string;
  api_key: string;
  plan: string;
  vector_limit: number;
  query_limit_per_month: number;
  message: string;
}

export interface UsageResponse {
  success: boolean;
  tenant_id: string;
  plan: string;
  usage: {
    queries: number;
    inserts: number;
    vectors_stored: number;
    month: string;
  };
  limits: {
    vector_limit: number;
    query_limit_per_month: number;
  };
}

class QuartzAPI {
  private apiKey: string = '';

  setApiKey(key: string) {
    this.apiKey = key;
  }

  getApiKey(): string {
    return this.apiKey;
  }

  private async fetch<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...(options.headers as Record<string, string>),
    };

    if (this.apiKey) {
      headers['X-API-Key'] = this.apiKey;
    }

    const response = await fetch(`${API_BASE}${endpoint}`, {
      ...options,
      headers,
    });

    if (!response.ok) {
      const text = await response.text();
      throw new Error(text || `HTTP ${response.status}`);
    }

    return response.json();
  }

  async health(): Promise<HealthResponse> {
    return this.fetch<HealthResponse>('/health');
  }

  async stats(): Promise<StatsResponse> {
    return this.fetch<StatsResponse>('/api/vector/stats');
  }

  async search(vector: number[], k: number = 10): Promise<SearchResponse> {
    return this.fetch<SearchResponse>('/api/vector/search', {
      method: 'POST',
      body: JSON.stringify({ vector, k }),
    });
  }

  async insert(id: string, vector: number[], metadata?: Record<string, unknown>): Promise<InsertResponse> {
    return this.fetch<InsertResponse>('/api/vector/insert', {
      method: 'POST',
      body: JSON.stringify({ id, vector, metadata }),
    });
  }

  async getVector(id: string): Promise<{ id: string; vector: number[]; metadata?: Record<string, unknown> }> {
    return this.fetch(`/api/vector/get/${encodeURIComponent(id)}`);
  }

  async deleteVector(id: string): Promise<{ success: boolean; message: string }> {
    return this.fetch(`/api/vector/delete/${encodeURIComponent(id)}`, {
      method: 'DELETE',
    });
  }

  async signup(tenantId: string): Promise<SignupResponse> {
    return this.fetch<SignupResponse>('/api/tenants/signup', {
      method: 'POST',
      body: JSON.stringify({ tenant_id: tenantId }),
    });
  }

  async usage(): Promise<UsageResponse> {
    return this.fetch<UsageResponse>('/api/usage');
  }
}

export const api = new QuartzAPI();
export default api;
