#!/usr/bin/env bash
set -euo pipefail

# Tooling used by the native server, browser WASM build, and vendored standards.
rustup component add clippy rustfmt
rustup target add \
  wasm32-unknown-unknown \
  wasm32-wasip2 \
  thumbv7em-none-eabihf

# Warm Cargo's dependency cache so rust-analyzer is useful immediately.
cargo fetch --locked

printf '\nDevelopment container ready.\n'
printf '  Native server: cargo run --bin server\n'
printf '  Tests:         cargo test\n'
printf '  Browser WASM:  wasm-pack build --target web\n'
