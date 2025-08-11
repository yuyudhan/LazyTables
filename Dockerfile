# FilePath: Dockerfile

# Multi-stage build for optimized image size

# Build stage
FROM rust:1.82-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev pkgconfig

# Create app directory
WORKDIR /usr/src/lazytables

# Copy Cargo files first for better caching
COPY Cargo.toml Cargo.lock ./

# Build dependencies only (for caching)
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache \
    libgcc \
    openssl \
    ca-certificates

# Create non-root user
RUN adduser -D -u 1000 lazytables

# Copy binary from builder
COPY --from=builder /usr/src/lazytables/target/release/lazytables /usr/local/bin/lazytables

# Set ownership
RUN chown lazytables:lazytables /usr/local/bin/lazytables

# Switch to non-root user
USER lazytables

# Set entrypoint
ENTRYPOINT ["lazytables"]

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD lazytables --version || exit 1