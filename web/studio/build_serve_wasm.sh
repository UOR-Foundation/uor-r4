#!/bin/sh
# Build the Studio's WASM serving bundle and its demo fixtures, then run the Studio
# worker verification. The individual steps are the recipe in web/README.md.
#
#   web/studio/build_serve_wasm.sh [--no-verify] [SEED]
#
# Writes into the gitignored web/studio/pkg/:
#   uor_r4_integer.js + uor_r4_integer_bg.wasm   crates/uor-r4-integer (serving-wasm)
#   synthetic-stack.lut                          seed <SEED> mk-synthetic-stack demo artifact
#   tokenizer.json                               stack-bound byte-bpe tokenizer artifact
#
# The page makes no network requests: every file above is served from the same origin.
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
pkg_dir="$repo_root/web/studio/pkg"
# wasm-pack shells out to `cargo`; put the rustup toolchain first so a PATH
# wrapper (the cargo-ai shim some hosts install) cannot answer `cargo metadata`.
PATH="$HOME/.cargo/bin:$PATH"
export PATH
cargo_bin=${CARGO_BIN:-"$HOME/.cargo/bin/cargo"}
wasm_pack_bin=${WASM_PACK_BIN:-"$HOME/.cargo/bin/wasm-pack"}
seed=5
verify=1

for arg in "$@"; do
    case "$arg" in
        --no-verify) verify=0 ;;
        '' | *[!0-9]*)
            echo "usage: web/studio/build_serve_wasm.sh [--no-verify] [SEED]" >&2
            exit 2
            ;;
        *) seed="$arg" ;;
    esac
done

mkdir -p "$pkg_dir"

echo "== wasm bundle (crates/uor-r4-integer, --features serving-wasm) =="
cd "$repo_root/crates/uor-r4-integer"
RUSTFLAGS='-C target-feature=-simd128' "$wasm_pack_bin" build --target web --no-opt \
    --out-dir "$pkg_dir" -- --features serving-wasm

echo "== synthetic stack fixture (seed $seed) =="
cd "$repo_root"
"$cargo_bin" run -q --locked -p uor-r4-integer --features serving-wasm --bin mk-synthetic-stack -- \
    "$seed" "$pkg_dir/synthetic-stack.lut"

echo "== stack-bound tokenizer artifact =="
node "$repo_root/web/studio/tools/make_tokenizer.mjs" "$pkg_dir/synthetic-stack.lut" \
    --out "$pkg_dir/tokenizer.json"

echo "== hashes =="
if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$pkg_dir/uor_r4_integer_bg.wasm" "$pkg_dir/synthetic-stack.lut"
else
    sha256sum "$pkg_dir/uor_r4_integer_bg.wasm" "$pkg_dir/synthetic-stack.lut"
fi

if [ "$verify" = 1 ]; then
    echo "== Studio worker verification =="
    UOR_WASM_PATH="$pkg_dir/uor_r4_integer_bg.wasm" \
    UOR_STACK_PATH="$pkg_dir/synthetic-stack.lut" \
        node "$repo_root/web/studio/verify_studio_worker.mjs"
fi
