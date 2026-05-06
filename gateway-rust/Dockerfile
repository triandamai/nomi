FROM rust:1.80-alpine AS builder
RUN apk add --no-cache musl-dev pkgconfig openssl-dev
WORKDIR /app
COPY . .
# We need to set SQLX_OFFLINE=true if we are not connecting to a DB during build
# Or we use sqlx-data.json
RUN cargo build --release

FROM alpine:latest
RUN apk add --no-cache libgcc openssl ca-certificates
WORKDIR /app
COPY --from=builder /app/target/release/gateway-rust .
COPY --from=builder /app/migrations ./migrations
EXPOSE 8000
CMD ["./gateway-rust"]
