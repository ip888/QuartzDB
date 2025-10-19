# Development container with Rust, Docker, Python, and Node.js
FROM mcr.microsoft.com/devcontainers/rust:1-bookworm

# Install additional dependencies
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends \
    wget \
    curl \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    libclang-dev \
    clang \
    llvm-dev \
    && apt-get clean -y && rm -rf /var/lib/apt/lists/*

# Note: Python and Node.js are installed via devcontainer features
# This ensures we get the latest versions (Python 3.13, Node.js 22 LTS)

# Python packages will be installed after Python feature is set up
# These will be installed via postCreateCommand or manually as needed

# Install Rust components
RUN rustup component add rustfmt clippy

# Set up workspace directory
WORKDIR /workspaces/QuartzDB

# Keep container running
CMD ["sleep", "infinity"]
