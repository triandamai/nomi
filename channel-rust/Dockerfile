FROM rust:1.80-alpine AS builder
RUN apk add --no-cache musl-dev pkgconfig openssl-dev sqlite-dev
WORKDIR /app
COPY . .

RUN SQLX_OFFLINE=true cargo build --release

FROM alpine:latest
RUN apk add --no-cache libgcc openssl ca-certificates sqlite-libs
WORKDIR /app
COPY --from=builder /app/target/release/channel-rust .
EXPOSE 8001
CMD ["./channel-rust"]
