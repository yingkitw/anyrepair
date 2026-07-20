# Multi-stage build for anyrepair CLI and MCP server
FROM rust:1-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
COPY benches ./benches
COPY examples ./examples

RUN cargo build --release --bin anyrepair --bin anyrepair-mcp

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/anyrepair /usr/local/bin/anyrepair
COPY --from=builder /app/target/release/anyrepair-mcp /usr/local/bin/anyrepair-mcp

ENTRYPOINT ["anyrepair"]
CMD ["--help"]
