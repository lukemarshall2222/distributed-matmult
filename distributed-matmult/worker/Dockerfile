FROM rust:latest AS builder
RUN groupadd -g 1000 appuser && \
    useradd -m -u 1000 -g 1000 appuser
WORKDIR /usr/src/worker
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo build --release
FROM debian:bookworm-slim
RUN groupadd -g 1000 appuser && \
    useradd -m -u 1000 -g 1000 appuser
COPY --from=builder /usr/src/worker/target/release/worker /usr/local/bin/worker
USER appuser
CMD ["worker"]