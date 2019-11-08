##### Stage 0
FROM rust:1.39-slim-buster
WORKDIR /app

RUN apt-get update && \
    apt-get install -y \
        libsodium23 \
        libsodium-dev \
        make \
        clang

# Required for libsodium-sys crate
RUN rustup component add rustfmt

# Copy and build
COPY . .
RUN cargo build -p regiusmark-server --release

##### Stage 1
FROM debian:buster-slim
WORKDIR /app

ENV REGIUSMARK_HOME="/data"

COPY --from=0 /app/target/release/regiusmark-server /app

STOPSIGNAL SIGINT
ENTRYPOINT ["/app/regiusmark-server"]
