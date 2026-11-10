FROM rust:1.94-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release -p e-isp-collector

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/e-isp-collector /usr/local/bin/e-isp-collector
ENV RUST_LOG=info
CMD ["/usr/local/bin/e-isp-collector"]
