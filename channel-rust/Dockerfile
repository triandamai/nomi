FROM rust:1.95-alpine AS builder
RUN apk add --no-cache musl-dev pkgconfig perl make gcc g++
WORKDIR /app
COPY . .

RUN cargo build --release

FROM alpine:latest
RUN apk add --no-cache libgcc openssl ca-certificates sqlite-libs
WORKDIR /app
COPY --from=builder /app/target/release/channel-rust .
EXPOSE 8001
CMD ["./channel-rust"]
