# ============================================================================
# Bounty Challenge - Multi-stage Docker Build with Cargo Chef
# ============================================================================
# This image is used by platform validators to run the bounty-challenge server
# Image: ghcr.io/platformnetwork/bounty-challenge:latest
# ============================================================================

# Stage 1: Chef - prepare recipe for dependency caching
FROM rust:1.92.0-slim-bookworm AS chef
RUN cargo install cargo-chef --locked
WORKDIR /build

# Stage 2: Planner - analyze dependencies
FROM chef AS planner

COPY Cargo.toml Cargo.lock config.toml ./
COPY src ./src
COPY migrations ./migrations

RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Build Rust binaries
FROM chef AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    git \
    && rm -rf /var/lib/apt/lists/*

# Build dependencies first (cached layer)
COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Copy source and build
COPY Cargo.toml Cargo.lock config.toml ./
COPY src ./src
COPY migrations ./migrations

# Build release binaries
RUN cargo build --release --bin bounty-server --bin bounty-health-server

# Stage 4: Runtime image
FROM debian:12.12-slim

ENV DEBIAN_FRONTEND=noninteractive

# Install runtime dependencies + gh CLI
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    curl \
    tini \
    gpg \
    && rm -rf /var/lib/apt/lists/* \
    && rm -rf /var/cache/apt/*

# Install GitHub CLI (gh)
RUN curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | gpg --dearmor -o /usr/share/keyrings/githubcli-archive-keyring.gpg \
    && echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" > /etc/apt/sources.list.d/github-cli.list \
    && apt-get update \
    && apt-get install -y --no-install-recommends gh \
    && rm -rf /var/lib/apt/lists/* \
    && rm -rf /var/cache/apt/*

# Create non-root user for security
RUN groupadd --gid 1000 bounty && \
    useradd --uid 1000 --gid 1000 --create-home bounty

WORKDIR /app

# Copy binaries from builder
COPY --from=builder /build/target/release/bounty-server /usr/local/bin/
COPY --from=builder /build/target/release/bounty-health-server /usr/local/bin/

# Copy entrypoint and MOTD
COPY docker/entrypoint.sh /entrypoint.sh
COPY docker/motd.txt /etc/motd
RUN chmod +x /entrypoint.sh

# Set ownership for app directory
RUN chown -R bounty:bounty /app

# Environment
ENV RUST_LOG=info,bounty_challenge=debug
ENV CHALLENGE_HOST=0.0.0.0
ENV CHALLENGE_PORT=8080

# Switch to non-root user
USER bounty

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

EXPOSE 8080

# Use tini for proper signal handling
ENTRYPOINT ["/usr/bin/tini", "--", "/entrypoint.sh"]

# Labels
LABEL org.opencontainers.image.source="https://github.com/PlatformNetwork/bounty-challenge"
LABEL org.opencontainers.image.description="Bounty Challenge - Reward miners for valid GitHub issues"
LABEL org.opencontainers.image.licenses="Apache-2.0"
LABEL org.opencontainers.image.vendor="PlatformNetwork"
