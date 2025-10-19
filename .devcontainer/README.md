# GitHub Codespaces Development Guide

This repository is configured for **GitHub Codespaces**, providing a fully-functional cloud development environment without requiring local Docker installation.

## 🚀 Quick Start with Codespaces

### Option 1: Create Codespace from GitHub

1. Go to your repository on GitHub
2. Click the green **Code** button
3. Select **Codespaces** tab
4. Click **Create codespace on main**
5. Wait 2-3 minutes for the environment to build

### Option 2: Use VS Code Desktop

1. Install the [GitHub Codespaces extension](https://marketplace.visualstudio.com/items?itemName=GitHub.codespaces) in VS Code
2. Press `Cmd+Shift+P` → "Codespaces: Create New Codespace"
3. Select this repository

## 💻 What's Included

The Codespace comes pre-configured with:

- ✅ **Rust 1.89+** (latest stable)
- ✅ **Docker-in-Docker** (build and run containers)
- ✅ **Python 3.11+** with sentence-transformers
- ✅ **GitHub CLI** (`gh` command)
- ✅ **VS Code extensions**: rust-analyzer, Docker, Python
- ✅ **Port forwarding** for QuartzDB (port 3000)

## 🛠️ Development Workflow

### 1. Build the Project

```bash
cargo build --release
```

### 2. Run QuartzDB Server

```bash
cargo run -p quartz-server
```

The server will start on `http://localhost:3000` (automatically forwarded)

### 3. Run Tests

```bash
# All tests
cargo test

# Integration tests only
cargo test --test integration_test

# Vector API tests
cargo test -p quartz-server --test vector_api_tests
```

### 4. Test Docker Build

```bash
# Build the Docker image
docker build -t quartzdb:local .

# Run the container
docker run -d -p 3000:3000 --name quartzdb quartzdb:local

# Check logs
docker logs quartzdb

# Test with Python demos
cd quartz-server/examples
python simple_vector_demo.py
python semantic_search_demo.py

# Stop container
docker stop quartzdb && docker rm quartzdb
```

### 5. Use Docker Compose

```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop all services
docker-compose down
```

## 🔧 Codespace Configuration

### `.devcontainer/devcontainer.json`

- Defines the development environment
- Auto-installs VS Code extensions
- Configures port forwarding
- Sets up Docker-in-Docker

### `.devcontainer/Dockerfile`

- Based on Microsoft's Rust devcontainer
- Includes Python, Docker, and build tools
- Pre-installs sentence-transformers

## 📊 GitHub Actions CI/CD

Every push triggers automated workflows:

1. **Docker Build & Test** (`.github/workflows/docker.yml`)
   - Builds Docker image
   - Checks image size (<150MB target)
   - Starts container and runs health checks
   - Tests both Python demos
   - Publishes to GitHub Container Registry

2. **Security Scan**
   - Runs Trivy vulnerability scanner
   - Reports to GitHub Security tab

## 🌐 Accessing Your Codespace

### Port Forwarding

When QuartzDB runs, Codespaces automatically forwards port 3000:

1. Look for the **PORTS** tab in VS Code terminal area
2. Port 3000 will show as "QuartzDB Server"
3. Click the globe icon to open in browser
4. Or use the forwarded URL in your Python scripts

### Public Access

To make your Codespace publicly accessible:

1. Go to **PORTS** tab
2. Right-click port 3000
3. Select **Port Visibility** → **Public**
4. Share the URL with others

## 💰 Codespaces Pricing

- **Free tier**: 120 core-hours/month + 15GB storage
- **2-core machine**: 60 hours/month free
- **4-core machine**: 30 hours/month free

For this project, a **2-core machine** is sufficient (60 hours free).

### Cost Optimization Tips

1. **Stop when not using**: Codespaces auto-stop after 30 minutes of inactivity
2. **Delete old Codespaces**: Keep only active ones
3. **Use prebuild**: Speeds up creation and reduces billable time
4. **Monitor usage**: Check GitHub Settings → Billing → Codespaces

## 🔄 Alternative: GitHub Actions Only

If you prefer not to use Codespaces, you can develop locally (without Docker) and rely on GitHub Actions for Docker builds:

### Local Development (M1 Mac, No Docker)

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build and run
cargo build --release
cargo run -p quartz-server

# Run tests
cargo test
```

### Docker Testing via GitHub Actions

1. Push code to GitHub
2. GitHub Actions automatically:
   - Builds Docker image
   - Runs integration tests
   - Publishes to GitHub Container Registry
3. Pull the image from GitHub Container Registry:

   ```bash
   docker pull ghcr.io/YOUR_USERNAME/quartzdb:latest
   ```

## 🐳 Using Pre-built Images

Instead of building locally, use images from GitHub Container Registry:

```bash
# Pull latest image
docker pull ghcr.io/YOUR_USERNAME/quartzdb:latest

# Run it
docker run -d -p 3000:3000 \
  -v $(pwd)/data:/data \
  ghcr.io/YOUR_USERNAME/quartzdb:latest

# Or use docker-compose with pre-built image
# Edit docker-compose.yml to use image instead of build
```

## 🚀 Deployment to DigitalOcean

Once Docker images are in GitHub Container Registry:

1. **DigitalOcean App Platform**:
   - Connect GitHub repository
   - Use pre-built image from GHCR
   - Auto-deploy on push

2. **DigitalOcean Droplet**:

   ```bash
   # SSH into droplet
   ssh root@your-droplet-ip
   
   # Install Docker
   curl -fsSL https://get.docker.com | sh
   
   # Pull and run
   docker pull ghcr.io/YOUR_USERNAME/quartzdb:latest
   docker run -d -p 3000:3000 \
     -v /data:/data \
     --restart always \
     ghcr.io/YOUR_USERNAME/quartzdb:latest
   ```

## 📝 Summary

**Best approach for M1 Mac (8GB/256GB)**:

1. ✅ **Use GitHub Codespaces** for full Docker development (free 60 hours/month)
2. ✅ **Or develop locally** without Docker (just `cargo build`)
3. ✅ **Let GitHub Actions** handle Docker builds/tests automatically
4. ✅ **Deploy pre-built images** from GitHub Container Registry

**Benefits**:

- No local Docker installation needed
- No M1 compatibility issues
- Saves local disk space (no 2GB+ Docker images)
- Free cloud compute for development
- Professional CI/CD pipeline

## 🆘 Troubleshooting

### Codespace won't start

- Check GitHub billing/limits
- Try creating a smaller machine (2-core)
- Delete old Codespaces to free quota

### Port 3000 not accessible

- Check PORTS tab shows "QuartzDB Server"
- Try stopping/starting the server
- Verify `QUARTZ_HOST=0.0.0.0` in environment

### Docker build fails in Codespace

- Check disk space: `df -h`
- Clear Docker cache: `docker system prune -a`
- Restart Codespace

### Python demos fail

- Verify server is running: `curl http://localhost:3000/api/v1/health`
- Check sentence-transformers installed: `pip list | grep sentence`
- Review server logs: `docker logs quartzdb`

## 📚 Additional Resources

- [GitHub Codespaces Docs](https://docs.github.com/en/codespaces)
- [Dev Container Specification](https://containers.dev/)
- [Docker in Codespaces](https://docs.github.com/en/codespaces/developing-in-codespaces/using-codespaces-for-pull-requests#using-docker-in-a-codespace)
- [QuartzDB Documentation](./README.md)
