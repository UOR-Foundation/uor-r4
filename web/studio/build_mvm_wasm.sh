#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
out_dir="$repo_root/web/studio/node_modules/.cache/uor-r4-mvm"
mkdir -p "$out_dir"

PATH="$HOME/.cargo/bin:$PATH"
export PATH
cd "$repo_root/crates/uor-r4-mvm"
RUSTFLAGS='-C target-feature=-simd128' "$HOME/.cargo/bin/wasm-pack" build --target web --no-opt --out-dir "$out_dir"

test -s "$out_dir/uor_r4_mvm.js"
test -s "$out_dir/uor_r4_mvm_bg.wasm"
test -s "$out_dir/uor_r4_mvm.d.ts"
printf 'Built MVM WASM bundle in %s\n' "$out_dir"
