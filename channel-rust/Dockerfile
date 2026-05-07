FROM rust:1.95-alpine AS builder
RUN apk add --no-cache musl-dev pkgconfig perl make gcc g++ binutils sqlite-dev
WORKDIR /app
COPY . .
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN cargo build --release

FROM alpine:latest
RUN apk add --no-cache libgcc openssl ca-certificates tzdata sqlite-libs && update-ca-certificates


ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
ENV SSL_CERT_DIR=/etc/ssl/certs

WORKDIR /app
COPY --from=builder /app/target/release/channel-rust .
EXPOSE 8001
CMD ["./channel-rust"]
