# QuartzDB Deployment Guide

This guide covers all deployment options for QuartzDB, from local development to production on DigitalOcean.

## 📋 Table of Contents

- [Development Environments](#development-environments)
- [Docker Deployment](#docker-deployment)
- [DigitalOcean Deployment](#digitalocean-deployment)
- [GitHub Container Registry](#github-container-registry)
- [Cost Comparison](#cost-comparison)

---

## 🖥️ Development Environments

### Option 1: GitHub Codespaces (Recommended for M1 Mac)

**Perfect for M1 MacBook with limited resources**

**Pros:**

- ✅ No local Docker installation needed
- ✅ Full Docker-in-Docker support in cloud
- ✅ Pre-configured development environment
- ✅ 60 hours/month free (2-core machine)
- ✅ Works on any device with a browser
- ✅ No M1 compatibility issues

**Setup:**

1. Go to your GitHub repository
2. Click **Code** → **Codespaces** → **Create codespace on main**
3. Wait 2-3 minutes for environment setup
4. Start developing!

**Usage:**

```bash
# Build the project
cargo build --release

# Run tests
cargo test

# Build Docker image
docker build -t quartzdb:test .

# Run container
docker run -d -p 3000:3000 quartzdb:test

# Test with demos
cd quartz-server/examples
python simple_vector_demo.py
```

[📖 Detailed Codespaces Guide](.devcontainer/README.md)

---

### Option 2: Local Development (No Docker)

**For M1 Mac without Docker Desktop**

**Pros:**

- ✅ Minimal resource usage
- ✅ Fast builds with native compilation
- ✅ No containers overhead

**Cons:**

- ❌ Can't test Docker builds locally
- ❌ Must rely on GitHub Actions for Docker testing

**Setup:**

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone repository
git clone https://github.com/YOUR_USERNAME/QuartzDB.git
cd QuartzDB

# Build
cargo build --release

# Run server
cargo run -p quartz-server
```

**Testing Docker builds:**

Instead of building locally, push to GitHub and let GitHub Actions build/test:

```bash
git add .
git commit -m "Update code"
git push origin main

# Wait for GitHub Actions to complete
# View at: https://github.com/YOUR_USERNAME/QuartzDB/actions
```

GitHub Actions will:

- Build Docker image
- Run all tests
- Publish to GitHub Container Registry (GHCR)

---

## 🐳 Docker Deployment

### Local Docker Testing (Codespaces or Docker Desktop)

**Build image:**

```bash
docker build -t quartzdb:latest .
```

**Run container:**

```bash
docker run -d \
  -p 3000:3000 \
  -v $(pwd)/data:/data \
  --name quartzdb \
  quartzdb:latest
```

**Check logs:**

```bash
docker logs -f quartzdb
```

**Test health:**

```bash
curl http://localhost:3000/api/v1/health
```

**Stop and remove:**

```bash
docker stop quartzdb
docker rm quartzdb
```

---

### Docker Compose

**Start all services:**

```bash
docker-compose up -d
```

**View logs:**

```bash
docker-compose logs -f quartzdb
```

**Stop all services:**

```bash
docker-compose down
```

**Rebuild and restart:**

```bash
docker-compose up -d --build
```

---

## 🚀 DigitalOcean Deployment

### Prerequisites

1. **DigitalOcean account** (get $200 credit: <https://try.digitalocean.com/freetrialoffer/>)
2. **GitHub repository** with code pushed
3. **doctl CLI** (optional, for command-line deployment)

### Option A: Web Console (Easiest)

1. Go to <https://cloud.digitalocean.com/apps>
2. Click **Create App**
3. Select **GitHub** as source
4. Authorize DigitalOcean to access your repository
5. Select **QuartzDB** repository
6. Configure:
   - Branch: `main`
   - Autodeploy: ✅ Enabled
   - Dockerfile: `Dockerfile`
7. Choose instance size: **Basic (512MB RAM, 1 vCPU) - $5/month**
8. Add environment variables:
   - `RUST_LOG=info`
   - `QUARTZ_HOST=0.0.0.0`
   - `QUARTZ_PORT=3000`
9. Configure health check: `/api/v1/health`
10. Click **Create Resources**

**Result:** Your app will be live at `https://<app-name>.ondigitalocean.app`

---

### Option B: Command Line (doctl)

**Install doctl:**

```bash
# macOS
brew install doctl

# Linux
cd ~
wget https://github.com/digitalocean/doctl/releases/download/v1.94.0/doctl-1.94.0-linux-amd64.tar.gz
tar xf doctl-1.94.0-linux-amd64.tar.gz
sudo mv doctl /usr/local/bin
```

**Authenticate:**

```bash
doctl auth init
```

**Deploy using app.yaml:**

```bash
# Run deployment script
./scripts/deploy-do.sh

# Or manually
doctl apps create --spec .do/app.yaml
```

**Monitor deployment:**

```bash
# List apps
doctl apps list

# Get app details
doctl apps get <APP_ID>

# View logs
doctl apps logs <APP_ID> --type run

# View build logs
doctl apps logs <APP_ID> --type build
```

**Update app:**

```bash
# Push code to GitHub
git push origin main

# App auto-deploys on push
# Or trigger manually
doctl apps create-deployment <APP_ID>
```

---

### Persistent Storage (Optional)

By default, data is ephemeral (lost on restart). For production:

1. Edit `.do/app.yaml`:

   ```yaml
   services:
     - name: quartzdb-server
       volumes:
         - name: quartzdb-data
           mount_path: /data
           size_gb: 5
   
   volumes:
     - name: quartzdb-data
       region: nyc
       size_gb: 5
   ```

2. Redeploy:

   ```bash
   doctl apps update <APP_ID> --spec .do/app.yaml
   ```

**Cost:** $1/month per 1GB

---

## 📦 GitHub Container Registry

All successful builds publish Docker images to GitHub Container Registry.

### Pull Pre-built Image

```bash
# Public repository
docker pull ghcr.io/YOUR_USERNAME/quartzdb:latest

# With authentication (private repo)
echo $GITHUB_TOKEN | docker login ghcr.io -u YOUR_USERNAME --password-stdin
docker pull ghcr.io/YOUR_USERNAME/quartzdb:latest
```

### Available Tags

- `latest` - Latest build from main branch
- `main` - Same as latest
- `pr-123` - Builds from pull requests
- `main-abc1234` - Builds with git SHA
- `v1.0.0` - Semantic version tags

### Use in Production

**docker-compose.yml:**

```yaml
services:
  quartzdb:
    image: ghcr.io/YOUR_USERNAME/quartzdb:latest
    ports:
      - "3000:3000"
    volumes:
      - ./data:/data
    restart: always
```

**DigitalOcean Droplet:**

```bash
# SSH into droplet
ssh root@your-droplet-ip

# Install Docker
curl -fsSL https://get.docker.com | sh

# Pull and run
docker pull ghcr.io/YOUR_USERNAME/quartzdb:latest

docker run -d \
  -p 3000:3000 \
  -v /data:/data \
  --restart always \
  --name quartzdb \
  ghcr.io/YOUR_USERNAME/quartzdb:latest
```

---

## 💰 Cost Comparison

### Development

| Option | Cost | Pros | Cons |
|--------|------|------|------|
| **Local (No Docker)** | Free | Fast, native | Can't test Docker |
| **Codespaces** | Free* | Full Docker, Cloud | 60 hours/month limit |
| **Docker Desktop** | Free | Local Docker | 2GB+ disk, M1 issues |

*Free tier: 120 core-hours/month (60 hours on 2-core)

---

### Production Hosting

| Platform | Spec | Cost/Month | Best For |
|----------|------|-----------|----------|
| **DigitalOcean App** | 512MB/1CPU | $5 | Quick deploy, auto-scaling |
| **DO Droplet** | 1GB/1CPU | $6 | More control, persistent |
| **DO Droplet** | 2GB/2CPU | $12 | Production workloads |
| **Fly.io** | 256MB | Free | Hobby projects |
| **Railway** | 512MB | $5 | Easy setup |

**Pinecone comparison:**

- Pinecone Starter: $70/month
- QuartzDB on DO: $5-12/month
- **Savings: $58-65/month (85-93% cheaper)**

---

### Storage Costs

| Provider | Cost | Notes |
|----------|------|-------|
| **DigitalOcean Volumes** | $1/GB/month | Persistent SSD |
| **DigitalOcean Spaces** | $5/250GB | Object storage |
| **GitHub Container Registry** | Free | Public repos unlimited |

---

## 🔧 Troubleshooting

### Build Fails in GitHub Actions

**Check:**

1. View Actions tab: `https://github.com/YOUR_USERNAME/QuartzDB/actions`
2. Look for error in build logs
3. Common issues:
   - Rust version mismatch
   - Missing dependencies
   - Test failures

**Fix:**

```bash
# Test locally first (in Codespaces)
cargo test
docker build -t quartzdb:test .
```

---

### DigitalOcean App Won't Start

**Debug:**

1. View logs: `doctl apps logs <APP_ID> --type run`
2. Check build logs: `doctl apps logs <APP_ID> --type build`
3. Verify health check: `curl https://your-app.ondigitalocean.app/api/v1/health`

**Common issues:**

- Port binding: Ensure `QUARTZ_HOST=0.0.0.0`
- Health check path: Must be `/api/v1/health`
- Build timeout: Increase timeout in app.yaml

---

### Can't Pull from GitHub Container Registry

**Authentication:**

```bash
# Create GitHub Personal Access Token
# Settings → Developer settings → Personal access tokens → Tokens (classic)
# Scopes needed: read:packages

# Login
echo $GITHUB_TOKEN | docker login ghcr.io -u YOUR_USERNAME --password-stdin
```

**Make image public:**

1. Go to package: `https://github.com/YOUR_USERNAME/QuartzDB/pkgs/container/quartzdb`
2. Package settings → Change visibility → Public

---

## 📊 Monitoring

### Health Check

```bash
curl https://your-app.ondigitalocean.app/api/v1/health
```

**Expected response:**

```json
{
  "status": "healthy",
  "version": "0.1.0"
}
```

---

### Performance Metrics

```bash
# From inside container
docker exec quartzdb cat /proc/meminfo
docker stats quartzdb
```

---

## 🚀 Next Steps

1. **Deploy to DigitalOcean** using web console or doctl
2. **Test with 100k+ vectors** using Python demos
3. **Run benchmarks** vs Pinecone/Weaviate
4. **Share demo URL** for feedback
5. **Monitor costs** in DO dashboard

---

## 📚 Additional Resources

- [DigitalOcean App Platform Docs](https://docs.digitalocean.com/products/app-platform/)
- [GitHub Actions for Docker](https://docs.github.com/en/actions/publishing-packages/publishing-docker-images)
- [GitHub Container Registry](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry)
- [Codespaces Docs](.devcontainer/README.md)
- [QuartzDB API Documentation](quartz-server/API.md)
- [Vector Search Documentation](quartz-server/VECTOR_SEARCH_API.md)

---

**Built with ❤️ for edge computing**
