FROM debian:12@sha256:6ebd97fa83deb272194a2cf015b3d26a4d538e9ad3a7a79d544c8af5b0a01443

ENV DEBIAN_FRONTEND=noninteractive \
    CARGO_HOME=/usr/local/cargo \
    RUSTUP_HOME=/usr/local/rustup \
    RUST_VERSION=1.98.0 \
    PATH=/usr/local/cargo/bin:$PATH

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl build-essential pkg-config \
    && rm -rf /var/lib/apt/lists/* \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain "$RUST_VERSION" \
    && rustup component add rustfmt clippy llvm-tools-preview \
    && rm -rf /root/.cache

WORKDIR /src
COPY . .
RUN cargo fmt --all -- --check \
    && cargo check --all-targets --all-features \
    && cargo test --all-targets --all-features \
    && cargo clippy --all-targets --all-features -- -D warnings \
    && cargo build --release --locked

ENTRYPOINT ["/src/target/release/galera-check"]
