FROM rust:1.95-alpine AS builder
RUN apk add --no-cache musl-dev pkgconfig perl make gcc g++
WORKDIR /app
COPY . .

RUN SQLX_OFFLINE=true cargo build --release

FROM alpine:latest
RUN apk add --no-cache libgcc openssl ca-certificates

WORKDIR /app
COPY --from=builder /app/target/release/gateway-rust .
COPY --from=builder /app/migrations ./migrations
COPY README.md ./README.md
EXPOSE 8000
CMD ["./gateway-rust"]