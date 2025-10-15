# Multi-stage build for optimal image size
FROM rust:1.70-slim-bullseye as builder

# Install system dependencies for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy source files
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

# Install runtime dependencies including yt-dlp and ffmpeg
RUN apt-get update && apt-get install -y \
    curl \
    python3 \
    python3-pip \
    ffmpeg \
    && rm -rf /var/lib/apt/lists/*

# Install yt-dlp
RUN curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o /usr/local/bin/yt-dlp \
    && chmod a+rx /usr/local/bin/yt-dlp

# Create non-root user
RUN useradd -m -u 1000 appuser
USER appuser
WORKDIR /app

# Copy built binary from builder stage
COPY --from=builder --chown=appuser:appuser /app/target/release/social-downloader-bot /app/

# Create downloads directory
RUN mkdir -p /app/downloads

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8000/health || exit 1

# Expose port (if you add health endpoint)
EXPOSE 8000

# Run the application
CMD ["./social-downloader-bot"]