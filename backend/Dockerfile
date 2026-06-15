FROM rust:1-bookworm AS builder

WORKDIR /app
COPY backend/Cargo.toml backend/Cargo.toml
COPY backend/Cargo.lock backend/Cargo.lock
COPY backend/rust-toolchain.toml backend/rust-toolchain.toml
COPY backend/src backend/src
COPY database database
WORKDIR /app/backend
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/backend/target/release/tuluklar-erp-api /usr/local/bin/tuluklar-erp-api
EXPOSE 8080
CMD ["tuluklar-erp-api"]
