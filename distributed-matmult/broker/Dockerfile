FROM rust:latest AS builder

RUN groupadd -g 1000 appuser && \
    useradd -m -u 1000 -g 1000 appuser
WORKDIR /usr/src/broker
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo build --release
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
RUN groupadd -g 1000 appuser && \
    useradd -m -u 1000 -g 1000 appuser
COPY --from=builder /usr/src/broker/target/release/broker /usr/local/bin/broker
USER appuser
CMD ["broker"]
