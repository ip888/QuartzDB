# Development container with Rust, Docker, and Python
FROM mcr.microsoft.com/devcontainers/rust:1-bookworm

# Install additional dependencies
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends \
    python3 \
    python3-pip \
    python3-venv \
    wget \
    curl \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    && apt-get clean -y && rm -rf /var/lib/apt/lists/*

# Install Python packages for demos
RUN pip3 install --no-cache-dir \
    requests \
    numpy \
    sentence-transformers

# Install Rust components
RUN rustup component add rustfmt clippy

# Set up workspace directory
WORKDIR /workspaces/QuartzDB

# Keep container running
CMD ["sleep", "infinity"]
