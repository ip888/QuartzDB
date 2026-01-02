# Monitoring & Analytics Setup

## Overview

This guide covers setting up comprehensive monitoring, logging, and analytics for QuartzDB FaaS.

## Metrics to Track

### Product Metrics
- **Latency** - P50, P95, P99 response times
- **Uptime** - Service availability (target: 99.9%)
- **Error Rate** - 4xx and 5xx errors (target: <1%)
- **Request Volume** - Requests per second/minute/day
- **Vector Operations** - Insert, search, delete operations
- **Storage Operations** - Put, get, delete operations

### Business Metrics
- **Active Users** - Daily/Monthly Active Users (DAU/MAU)
- **API Key Usage** - Requests per API key
- **Feature Usage** - Which endpoints are most popular
- **Geographic Distribution** - Where requests come from

## Built-in Cloudflare Analytics

### Workers Analytics Dashboard
1. Go to Cloudflare Dashboard → Workers & Pages
2. Select your worker
3. View metrics:
   - Total requests
   - Success rate
   - CPU time
   - Errors
   - Bandwidth

### Real-time Logs
```bash
# Tail all logs
wrangler tail

# Filter for errors only
wrangler tail --status error

# Filter by specific path
wrangler tail --search "/api/vector"

# Production environment
wrangler tail --env production
```

## Custom Metrics with Workers Analytics Engine

### 1. Enable Analytics Engine in wrangler.toml
```toml
[[analytics_engine_datasets]]
binding = "ANALYTICS"
```

### 2. Track Custom Events
Add to `quartz-faas/src/lib.rs`:

```rust
use worker::*;

pub async fn track_request(
    env: &Env,
    endpoint: &str,
    status: u16,
    duration_ms: u64,
) -> Result<()> {
    if let Ok(analytics) = env.analytics_engine("ANALYTICS") {
        analytics
            .write_data_point(json!({
                "doubles": [duration_ms as f64],
                "blobs": [endpoint, &status.to_string()],
            }))
            .await?;
    }
    Ok(())
}
```

### 3. Query Analytics
```bash
# Query from CLI
wrangler analytics

# Or use GraphQL API
# https://developers.cloudflare.com/analytics/graphql-api/
```

## External Monitoring Services

### Option 1: Sentry for Error Tracking

#### Install
```bash
cd quartz-faas
cargo add sentry
```

#### Configure
```rust
// In main()
let _guard = sentry::init((
    "https://your-dsn@sentry.io/project",
    sentry::ClientOptions {
        release: Some(env!("CARGO_PKG_VERSION").into()),
        ..Default::default()
    },
));
```

#### Track Errors
```rust
use sentry;

// Capture error
sentry::capture_error(&error);

// Add context
sentry::configure_scope(|scope| {
    scope.set_tag("endpoint", "/api/vector/search");
    scope.set_user(Some(sentry::User {
        id: Some(api_key),
        ..Default::default()
    }));
});
```

### Option 2: Datadog for APM

#### Setup
```bash
# Add Datadog integration in Cloudflare Dashboard
# Workers & Pages → <worker> → Integrations → Datadog
```

#### Configure
```toml
# wrangler.toml
[observability]
enabled = true

[[observability.integrations]]
type = "datadog"
```

### Option 3: Grafana Cloud for Dashboards

#### Push Metrics via HTTP
```rust
// Send metrics to Grafana
pub async fn send_metric(name: &str, value: f64) {
    let client = reqwest::Client::new();
    client
        .post("https://prometheus-us-central1.grafana.net/api/prom/push")
        .basic_auth("user", Some("password"))
        .json(&json!({
            "streams": [{
                "stream": {"job": "quartz-faas"},
                "values": [[timestamp(), value.to_string()]]
            }]
        }))
        .send()
        .await
        .ok();
}
```

## Structured Logging

### Add tracing to FaaS layer
```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(req))]
async fn handle_vector_search(req: Request) -> Result<Response> {
    info!("Vector search request received");
    
    let start = std::time::Instant::now();
    let result = perform_search().await;
    let duration = start.elapsed();
    
    info!(
        duration_ms = duration.as_millis(),
        results_count = result.len(),
        "Vector search completed"
    );
    
    Ok(Response::from_json(&result)?)
}
```

### Log Levels
```rust
// Set log level based on environment
let log_level = env.var("LOG_LEVEL").unwrap_or("info".to_string());

tracing_subscriber::fmt()
    .with_max_level(match log_level.as_str() {
        "debug" => tracing::Level::DEBUG,
        "info" => tracing::Level::INFO,
        "warn" => tracing::Level::WARN,
        "error" => tracing::Level::ERROR,
        _ => tracing::Level::INFO,
    })
    .init();
```

## Health Checks

### Enhanced Health Endpoint
Update `quartz-faas/src/lib.rs`:

```rust
.get_async("/health", |_, ctx| async move {
    // Check Durable Objects connectivity
    let storage_health = check_storage(&ctx.env).await;
    let vector_health = check_vector_index(&ctx.env).await;
    
    let healthy = storage_health && vector_health;
    
    let response = serde_json::json!({
        "status": if healthy { "healthy" } else { "unhealthy" },
        "service": "quartz-faas",
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_seconds": get_uptime_seconds(),
        "checks": {
            "storage": storage_health,
            "vector_index": vector_health,
        }
    });
    
    if healthy {
        Response::from_json(&response)
    } else {
        Response::error("Service unhealthy", 503)
    }
})
```

### External Health Check Services
- **UptimeRobot** - Free tier: 50 monitors, 5-min intervals
- **Pingdom** - Synthetic monitoring
- **Better Uptime** - Status pages + monitoring

## Alerts & Notifications

### Cloudflare Alerts
1. Dashboard → Notifications
2. Create alerts for:
   - Error rate > 5%
   - Response time > 1000ms
   - Worker downtime
   - CPU time limit exceeded

### PagerDuty Integration
```bash
# Add webhook in Cloudflare notifications
# Point to PagerDuty Events API v2
# https://events.pagerduty.com/v2/enqueue
```

### Slack Notifications
```rust
// Send alert to Slack
async fn send_slack_alert(message: &str) {
    let webhook_url = env::var("SLACK_WEBHOOK_URL").unwrap();
    
    reqwest::Client::new()
        .post(&webhook_url)
        .json(&json!({
            "text": message,
            "username": "QuartzDB Alert",
            "icon_emoji": ":warning:"
        }))
        .send()
        .await
        .ok();
}
```

## Performance Monitoring

### Trace Request Lifecycle
```rust
#[instrument]
async fn handle_request(req: Request) -> Result<Response> {
    let method = req.method();
    let path = req.path();
    
    let start = std::time::Instant::now();
    
    // Process request
    let response = process(req).await;
    
    let duration = start.elapsed();
    
    // Log performance
    info!(
        method = ?method,
        path = %path,
        status = response.status_code(),
        duration_ms = duration.as_millis(),
        "Request completed"
    );
    
    // Track slow requests
    if duration.as_millis() > 500 {
        warn!("Slow request detected: {}ms", duration.as_millis());
    }
    
    response
}
```

### Track Vector Search Performance
```rust
#[instrument]
async fn vector_search(query: Vec<f32>, k: usize) -> Result<Vec<SearchResult>> {
    let vector_dim = query.len();
    
    let start = std::time::Instant::now();
    let results = do_search(query, k).await?;
    let search_time = start.elapsed();
    
    info!(
        vector_dim,
        k,
        results_count = results.len(),
        search_time_ms = search_time.as_millis(),
        "Vector search completed"
    );
    
    Ok(results)
}
```

## Dashboard Examples

### Grafana Dashboard JSON
```json
{
  "dashboard": {
    "title": "QuartzDB FaaS Monitoring",
    "panels": [
      {
        "title": "Request Rate",
        "targets": [{
          "expr": "rate(requests_total[5m])"
        }]
      },
      {
        "title": "Error Rate",
        "targets": [{
          "expr": "rate(requests_total{status=~\"5..\"}[5m])"
        }]
      },
      {
        "title": "Latency (P95)",
        "targets": [{
          "expr": "histogram_quantile(0.95, request_duration_ms)"
        }]
      }
    ]
  }
}
```

## Cost Analysis

### Monitoring Costs
- **Cloudflare Analytics**: Free
- **Sentry**: Free tier 5K events/month, then $26/month
- **Datadog**: $15/host/month
- **Grafana Cloud**: Free tier, then $49/month
- **UptimeRobot**: Free tier 50 monitors

### Recommended Stack (Budget-Friendly)
1. **Cloudflare Workers Analytics** - Built-in, free
2. **wrangler tail** - Real-time logs, free
3. **Sentry Free Tier** - Error tracking
4. **UptimeRobot Free** - Uptime monitoring
5. **Custom Health Checks** - Build your own dashboard

**Total Cost: $0/month** for MVP phase

## Implementation Checklist

- [ ] Enable Cloudflare Workers Analytics
- [ ] Set up wrangler tail for logs
- [ ] Add structured logging with tracing
- [ ] Implement enhanced health check endpoint
- [ ] Configure Cloudflare alerts (error rate, latency)
- [ ] Set up external uptime monitoring (UptimeRobot)
- [ ] Add error tracking (Sentry or similar)
- [ ] Create custom dashboard (Grafana/Datadog)
- [ ] Set up Slack notifications
- [ ] Document runbook for incidents

## Next Steps

After monitoring is set up:
1. Establish SLOs (Service Level Objectives)
   - Latency: P95 < 100ms
   - Uptime: 99.9%
   - Error rate: < 1%
2. Create runbooks for common issues
3. Set up on-call rotation (for production)
4. Regular metric reviews (weekly)
5. Performance optimization based on metrics
