ARG RUST_VERSION=1.97.1
FROM rust:${RUST_VERSION}-bookworm

WORKDIR /workspace/simulation

COPY rust-toolchain.toml ./rust-toolchain.toml
RUN rustup component add rustfmt clippy
