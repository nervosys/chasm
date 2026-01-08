# Build stage
FROM rust:1.75-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release && rm -rf src

# Copy actual source code
COPY src ./src
COPY tests ./tests

# Touch main.rs to force rebuild of our code
RUN touch src/main.rs

# Build the actual binary
RUN cargo build --release --bin chasm

# Runtime stage
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache ca-certificates

# Create non-root user
RUN addgroup -g 1000 chasm && \
    adduser -u 1000 -G chasm -s /bin/sh -D chasm

# Copy binary from builder
COPY --from=builder /app/target/release/chasm /usr/local/bin/chasm

# Create data directory
RUN mkdir -p /data && chown chasm:chasm /data

# Switch to non-root user
USER chasm

# Set working directory
WORKDIR /data

# Default environment
ENV CSM_DATABASE_PATH=/data/csm.db

# Expose API port
EXPOSE 8787

# Default command
CMD ["chasm", "api", "serve", "--host", "0.0.0.0"]
