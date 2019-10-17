FROM rust:1-alpine AS builder
COPY Cargo.lock Cargo.toml ./
RUN cargo fetch --locked
COPY src/ ./src/
RUN cargo build --frozen --release

FROM alpine:3.10
COPY --from=builder ./target/release/ ./
