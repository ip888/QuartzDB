# Production Deployment Guide

## Prerequisites

Before deploying QuartzDB FaaS to Cloudflare Workers, you need:

### 1. Cloudflare Account
- Sign up at https://dash.cloudflare.com/sign-up
- Free tier includes 100,000 requests/day
- No credit card required for development

### 2. Install Node.js & npm
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install nodejs npm

# macOS (via Homebrew)
brew install node

# Verify installation
node --version  # Should be v16+ 
npm --version
```

### 3. Install Wrangler CLI
```bash
# Install globally
npm install -g wrangler

# Verify installation
wrangler --version

# Login to Cloudflare
wrangler login
```

### 4. Install worker-build
```bash
cargo install worker-build
```

## Deployment Steps

### Step 1: Build the Worker
```bash
cd quartz-faas
worker-build --release
```

This compiles Rust code to WebAssembly and generates Worker files in `build/worker/`.

### Step 2: Deploy to Development
```bash
# Deploy to dev environment (workers.dev subdomain)
wrangler deploy

# Your worker will be available at:
# https://quartz-faas.<your-subdomain>.workers.dev
```

### Step 3: Test the Deployment
```bash
# Health check
curl https://quartz-faas.<your-subdomain>.workers.dev/health

# Put a key-value pair
curl -X POST https://quartz-faas.<your-subdomain>.workers.dev/api/put \
  -H "Content-Type: application/json" \
  -d '{"key": "test", "value": "hello world"}'

# Get the value
curl https://quartz-faas.<your-subdomain>.workers.dev/api/get/test

# Insert a vector
curl -X POST https://quartz-faas.<your-subdomain>.workers.dev/api/vector/insert \
  -H "Content-Type: application/json" \
  -d '{
    "id": 1,
    "vector": [0.1, 0.2, 0.3, 0.4, 0.5],
    "metadata": {"label": "test"}
  }'

# Search vectors
curl -X POST https://quartz-faas.<your-subdomain>.workers.dev/api/vector/search \
  -H "Content-Type: application/json" \
  -d '{
    "query": [0.1, 0.2, 0.3, 0.4, 0.5],
    "k": 5
  }'
```

### Step 4: Deploy to Production
```bash
# Deploy to production environment
wrangler deploy --env production

# Your production worker will be at:
# https://quartz-faas-prod.<your-subdomain>.workers.dev
```

## Custom Domain Setup

### Add Custom Domain
1. Go to Cloudflare Dashboard → Workers & Pages
2. Select your worker
3. Go to Settings → Triggers → Custom Domains
4. Add `api.quartzdb.com` (or your domain)
5. Cloudflare will automatically configure DNS and SSL

### Update wrangler.toml
```toml
[env.production]
name = "quartz-faas-prod"
workers_dev = false
routes = [
  { pattern = "api.quartzdb.com/*", zone_name = "quartzdb.com" }
]
```

## Environment Variables & Secrets

### Add API Keys (for future authentication)
```bash
# Add secret
wrangler secret put API_KEY

# List secrets
wrangler secret list
```

### Environment-specific configuration
Edit `wrangler.toml`:
```toml
[env.production.vars]
ENVIRONMENT = "production"
LOG_LEVEL = "info"
```

## Monitoring

### View Logs
```bash
# Tail logs in real-time
wrangler tail

# Tail production logs
wrangler tail --env production
```

### Metrics Dashboard
- Go to Cloudflare Dashboard → Workers & Pages
- Select your worker
- View metrics: requests, errors, CPU time, bandwidth

## Rollback

### Deploy specific version
```bash
# List deployments
wrangler deployments list

# Rollback to previous version
wrangler rollback --version <version-id>
```

## Cost Estimates

### Free Tier (Development)
- 100,000 requests/day
- 10ms CPU time per request
- No credit card required
- **Cost: $0/month**

### Paid Tier (Production - if needed)
- $5/month for 10M requests
- Additional $0.50 per 1M requests
- Durable Objects: $0.15 per 1M reads, $1.00 per 1M writes
- **Estimated Cost for 1M requests/month: $5-10/month**

Compare to alternatives:
- Pinecone: $70-280/month
- Weaviate: $25-99/month
- AWS Lambda: $50-150/month

## Troubleshooting

### Build Errors
```bash
# Clean build
rm -rf build/
cargo clean
worker-build --release
```

### Deployment Errors
```bash
# Check wrangler config
wrangler whoami

# Validate wrangler.toml
wrangler deploy --dry-run

# Check Durable Objects bindings
wrangler deployments list
```

### Runtime Errors
```bash
# View real-time logs
wrangler tail

# Check errors in dashboard
# Go to Workers & Pages → <worker> → Logs
```

## Next Steps

1. **Add Authentication** - Implement API key validation
2. **Add Rate Limiting** - Prevent abuse
3. **Add Monitoring** - Integrate with Sentry or similar
4. **Custom Domain** - Point api.quartzdb.com to worker
5. **CI/CD** - Automate deployment with GitHub Actions

## Security Checklist

- [ ] Add API key authentication
- [ ] Enable rate limiting
- [ ] Set up CORS properly
- [ ] Add request validation
- [ ] Configure WAF rules in Cloudflare
- [ ] Enable DDoS protection
- [ ] Set up monitoring & alerting
- [ ] Regular security audits

## Performance Optimization

- [ ] Enable caching for GET requests
- [ ] Optimize Durable Objects reads
- [ ] Batch vector operations
- [ ] Implement connection pooling
- [ ] Add request compression
- [ ] Monitor cold start times
- [ ] Profile CPU usage

## Resources

- [Cloudflare Workers Docs](https://developers.cloudflare.com/workers/)
- [Durable Objects Guide](https://developers.cloudflare.com/workers/runtime-apis/durable-objects/)
- [Wrangler CLI Reference](https://developers.cloudflare.com/workers/wrangler/commands/)
- [worker-rs GitHub](https://github.com/cloudflare/workers-rs)
