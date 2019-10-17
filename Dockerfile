FROM clux/muslrust:stable AS build

WORKDIR /build
COPY Cargo.lock Cargo.toml ./
RUN cargo fetch -v --locked

COPY src ./src
RUN cargo build --release -v --locked --all

FROM guangie88/releaser:alpine_upx-3_ghr-0.12 AS misc
WORKDIR /build
ARG ARCH=amd64
ARG OS=linux
COPY --from=build /build/target/x86_64-unknown-linux-musl/release/httpose ./httpose_${ARCH}_${OS}
RUN upx --lzma --best ./httpose_${ARCH}_${OS}

FROM scratch AS release
WORKDIR /app
ARG ARCH=amd64
ARG OS=linux
COPY --from=misc /build/httpose_${ARCH}_${OS} ./httpose
CMD ["./httpose"]
