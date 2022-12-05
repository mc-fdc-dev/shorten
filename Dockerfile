FROM rust:1 AS builder

WORKDIR /tmp
COPY . .

RUN cargo build --release

FROM debian:bullseye

WORKDIR /app
COPY --from=builder /tmp/target/release/tinyurl .

CMD ["./target/release/tinyurl"]
