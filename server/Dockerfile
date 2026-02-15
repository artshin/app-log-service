# Multi-stage Dockerfile for Rust log server
# Build: docker build -t log-server-rust .
# NOTE: Run 'make build-web' before 'docker build' so static/app/ contains the SPA.

# Stage 1: Build
FROM rust:1.85-alpine AS builder

# Install build dependencies
RUN apk add --no-cache \
    musl-dev

WORKDIR /build

# Copy Cargo files for dependency caching
COPY Cargo.toml Cargo.lock* ./

# Create dummy src to cache dependencies
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs

# Build dependencies only (cached layer)
RUN cargo build --release || true

# Remove dummy files
RUN rm -rf src

# Copy actual source code and templates (templates needed at compile time for Askama)
COPY src ./src
COPY templates ./templates
COPY askama.toml ./

# Build the actual application
RUN touch src/main.rs && cargo build --release

# Stage 2: Runtime
FROM alpine:3.20 AS runtime

# Install runtime dependencies (minimal)
RUN apk --no-cache add ca-certificates tzdata

WORKDIR /app

# Copy binary from builder
COPY --from=builder /build/target/release/log-server /app/log-server

# Copy static files (CSS/JS + built SPA in static/app/)
COPY static ./static

# Create non-root user and uploads directory with proper permissions
RUN addgroup -S appgroup && adduser -S appuser -G appgroup && \
    mkdir -p /app/uploads && \
    chown -R appuser:appgroup /app/uploads
USER appuser

# Expose port
EXPOSE 9006

# Environment defaults
ENV PORT=9006
ENV CAPACITY=1000
ENV VERBOSE=false
ENV UPLOAD_DIR=/app/uploads

# Set metadata
LABEL description="Log Server (Rust) - Development logging service"
LABEL version="1.0"

# Run the server
ENTRYPOINT ["/app/log-server"]
