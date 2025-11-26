# syntax=docker/dockerfile:1

# ==============================================================================
# Phone Manager Backend - Multi-stage Dockerfile
# ==============================================================================

# ------------------------------------------------------------------------------
# Stage 1: Build (Production)
# ------------------------------------------------------------------------------
FROM rust:1.83-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace manifests first for better caching
COPY Cargo.toml Cargo.lock ./
COPY crates/api/Cargo.toml crates/api/
COPY crates/domain/Cargo.toml crates/domain/
COPY crates/persistence/Cargo.toml crates/persistence/
COPY crates/shared/Cargo.toml crates/shared/

# Create dummy source files for dependency caching
RUN mkdir -p crates/api/src crates/domain/src crates/persistence/src crates/shared/src && \
    echo "fn main() {}" > crates/api/src/main.rs && \
    echo "pub fn lib() {}" > crates/api/src/lib.rs && \
    echo "pub fn lib() {}" > crates/domain/src/lib.rs && \
    echo "pub fn lib() {}" > crates/persistence/src/lib.rs && \
    echo "pub fn lib() {}" > crates/shared/src/lib.rs

# Build dependencies only (this layer is cached)
RUN cargo build --release --workspace && \
    rm -rf crates/*/src

# Copy actual source code
COPY crates/ crates/
COPY config/ config/

# Touch source files to invalidate cache and rebuild with actual code
RUN touch crates/api/src/main.rs crates/api/src/lib.rs \
    crates/domain/src/lib.rs crates/persistence/src/lib.rs crates/shared/src/lib.rs

# Build the actual application
RUN cargo build --release --bin phone-manager

# ------------------------------------------------------------------------------
# Stage 2: Runtime (Production)
# ------------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -u 1000 -U appuser

# Copy binary from builder
COPY --from=builder /app/target/release/phone-manager /app/phone-manager

# Copy config files
COPY config/ /app/config/

# Set ownership
RUN chown -R appuser:appuser /app

USER appuser

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/api/health/live || exit 1

# Default command
CMD ["/app/phone-manager"]

# ------------------------------------------------------------------------------
# Stage 3: Development (with cargo-watch)
# ------------------------------------------------------------------------------
FROM rust:1.83-slim-bookworm AS development

WORKDIR /app

# Install development dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install cargo-watch for live reloading
RUN cargo install cargo-watch

# Create non-root user
RUN useradd -r -u 1000 -U appuser && \
    mkdir -p /home/appuser && \
    chown -R appuser:appuser /home/appuser

# Set cargo home for caching
ENV CARGO_HOME=/app/.cargo

# Expose port
EXPOSE 8080

# Default command for development - watch and rebuild
CMD ["cargo", "watch", "-x", "run --bin phone-manager"]
