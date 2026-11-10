FROM rust:1.94-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release -p e-isp-api

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libssl3 curl && rm -rf /var/lib/apt/lists/*
WORKDIR /app/apps/api
COPY --from=builder /app/target/release/e-isp-api /usr/local/bin/e-isp-api
COPY --from=builder /app/apps/api/config ./config
ENV RUST_LOG=info
EXPOSE 5150
CMD ["/usr/local/bin/e-isp-api", "start"]
