# syntax=docker/dockerfile:1.7
# dlaunch — launcher CLI.
#
# Single-binary Rust tool, no native libraries, no path dependencies.
# Build:
#   docker build -t dlaunch:local .
# Run:
#   docker run --rm dlaunch:local --help

ARG RUST_IMAGE=rust:1.99-bookworm
ARG RUNTIME_IMAGE=debian:bookworm-slim

FROM ${RUST_IMAGE} AS chef
RUN cargo install cargo-chef --locked
WORKDIR /workspace/dlaunch

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
ENV CARGO_TERM_COLOR=always
ENV RUSTFLAGS="-C strip=symbols"
COPY --from=planner /workspace/dlaunch/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/workspace/dlaunch/target \
    cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/workspace/dlaunch/target \
    cargo build --release --bin dlaunch \
 && cp target/release/dlaunch /usr/local/bin/dlaunch

FROM ${RUNTIME_IMAGE} AS runtime
RUN apt-get update \
 && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
      ca-certificates \
 && rm -rf /var/lib/apt/lists/* \
 && useradd --create-home --uid 10001 --shell /usr/sbin/nologin dlaunch

COPY --from=builder /usr/local/bin/dlaunch /usr/local/bin/dlaunch

USER dlaunch
WORKDIR /home/dlaunch
ENV HOME=/home/dlaunch

ENTRYPOINT ["dlaunch"]
CMD ["--help"]
