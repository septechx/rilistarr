FROM rust:1.93-slim AS builder

WORKDIR /app
COPY . .

RUN apt-get update && apt-get install -y pkg-config libssl-dev
RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/rilistarr /app/rilistarr
COPY --from=builder /app/assets /app/assets

CMD ["/app/rilistarr"]
