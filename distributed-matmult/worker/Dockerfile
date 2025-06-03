# Use an official Rust image.
# builder stage
FROM rust:latest AS builder

# Create appuser
RUN groupadd -g 1000 appuser && \
    useradd -m -u 1000 -g 1000 appuser

WORKDIR /usr/src/worker

# Copy your manifests
COPY Cargo.toml Cargo.lock* ./
# Copy your source code
COPY src ./src

# Build for release.
# RUN mkdir src && echo "fn main(){}" > src/main.rs && echo "pub type Matrix = Vec<Vec<i32>>;" > src/types.rs
RUN cargo build --release

# final stage
FROM debian:bookworm-slim

# Create appuser (must match UID/GID from builder)
RUN groupadd -g 1000 appuser && \
    useradd -m -u 1000 -g 1000 appuser

# Copy the compiled binary from the builder stage.
COPY --from=builder /usr/src/worker/target/release/worker /usr/local/bin/worker

# Set the user to a non-root user.
USER appuser

# Set the entrypoint.
CMD ["worker"]