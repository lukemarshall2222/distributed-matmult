# Use an official Rust image.
# builder stage
FROM rust:latest AS builder

# Create appuser
RUN groupadd -g 1000 appuser && \
    useradd -m -u 1000 -g 1000 appuser

WORKDIR /usr/src/broker

# Copy your manifests
COPY Cargo.toml Cargo.lock* ./
# Copy your source code
COPY src ./src

# Build for release.
# Ensure a dummy types.rs and main.rs exist if src isn't copied yet for layer caching.
# RUN mkdir src && echo "fn main(){}" > src/main.rs && echo "pub type Matrix = Vec<Vec<i32>>;" > src/types.rs
RUN cargo build --release

# final stage
FROM debian:bookworm-slim

# Install libssl3 which is required by reqwest (used in broker's main.rs) for HTTPS/HTTP client functionality
# Also install ca-certificates for reqwest to be able to verify TLS certificates
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

# Create appuser (must match UID/GID from builder)
RUN groupadd -g 1000 appuser && \
    useradd -m -u 1000 -g 1000 appuser

# Copy the compiled binary from the builder stage.
COPY --from=builder /usr/src/broker/target/release/broker /usr/local/bin/broker

# Set the user to a non-root user.
USER appuser

# Set the entrypoint.
CMD ["broker"]
