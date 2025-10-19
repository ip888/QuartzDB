# Multi-stage Docker build for QuartzDB
# Stage 1: Builder - Compile Rust application
# Stage 2: Runtime - Minimal Alpine Linux with binary only

# ============================================================================
# BUILDER STAGE
# ============================================================================
FROM rust:1.89-alpine AS builder

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    openssl-libs-static

WORKDIR /app

# Copy dependency manifests first (for better layer caching)
COPY Cargo.toml Cargo.lock ./
COPY quartz-core/Cargo.toml quartz-core/Cargo.toml
COPY quartz-storage/Cargo.toml quartz-storage/Cargo.toml
COPY quartz-vector/Cargo.toml quartz-vector/Cargo.toml
COPY quartz-server/Cargo.toml quartz-server/Cargo.toml
COPY quartz-client/Cargo.toml quartz-client/Cargo.toml
COPY quartz-network/Cargo.toml quartz-network/Cargo.toml
COPY quartz-edge/Cargo.toml quartz-edge/Cargo.toml

# Create dummy source files to build dependencies
RUN mkdir -p quartz-core/src quartz-storage/src quartz-vector/src quartz-server/src \
    quartz-client/src quartz-network/src quartz-edge/src && \
    echo "fn main() {}" > quartz-server/src/main.rs && \
    echo "pub fn dummy() {}" > quartz-core/src/lib.rs && \
    echo "pub fn dummy() {}" > quartz-storage/src/lib.rs && \
    echo "pub fn dummy() {}" > quartz-vector/src/lib.rs && \
    echo "pub fn dummy() {}" > quartz-client/src/lib.rs && \
    echo "pub fn dummy() {}" > quartz-network/src/lib.rs && \
    echo "pub fn dummy() {}" > quartz-edge/src/lib.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release -p quartz-server && \
    rm -rf target/release/deps/quartz*

# Copy actual source code
COPY quartz-core quartz-core
COPY quartz-storage quartz-storage
COPY quartz-vector quartz-vector
COPY quartz-server quartz-server
COPY quartz-client quartz-client
COPY quartz-network quartz-network
COPY quartz-edge quartz-edge

# Build the actual application
RUN cargo build --release -p quartz-server

# Strip binary to reduce size
RUN strip target/release/quartz-server

# ============================================================================
# RUNTIME STAGE
# ============================================================================
FROM alpine:latest

# Install runtime dependencies only
RUN apk add --no-cache \
    ca-certificates \
    libgcc

# Create non-root user for security
RUN addgroup -g 1000 quartz && \
    adduser -D -u 1000 -G quartz quartz

# Create data directory
RUN mkdir -p /data && chown -R quartz:quartz /data

# Copy binary from builder
COPY --from=builder /app/target/release/quartz-server /usr/local/bin/quartz-server

# Set ownership
RUN chown quartz:quartz /usr/local/bin/quartz-server

# Switch to non-root user
USER quartz

# Set working directory
WORKDIR /data

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3000/api/v1/health || exit 1

# Environment variables
ENV RUST_LOG=info
ENV QUARTZ_HOST=0.0.0.0
ENV QUARTZ_PORT=3000
ENV QUARTZ_DATA_PATH=/data

# Run the server
CMD ["quartz-server"]
