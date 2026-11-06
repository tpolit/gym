#!/bin/bash
# Builds the release .wasm like Trunk does, in a separate target directory, and prints its sizes.
# Usage (inside the container: tools/dev.sh tools/wasm_size_experiment.sh <name> [extra RUSTFLAGS]).
# Used to compare size options without disturbing the main build cache. `out/named.wasm` keeps the
# function names for `tools/wasm_size.py`.
set -euo pipefail

NAME="$1"
shift
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TARGET_DIR="$ROOT/target/size-$NAME"
OUT="$TARGET_DIR/out"
TRUNK_CACHE="$ROOT/.toolchain/cache/trunk"

# An exported RUSTFLAGS, even empty, would replace the flags of .cargo/config.toml.
if [ $# -gt 0 ]; then
  export RUSTFLAGS="$*"
fi
CARGO_TARGET_DIR="$TARGET_DIR" cargo build --quiet --release --target wasm32-unknown-unknown -p ui_leptos
mkdir -p "$OUT"
"$TRUNK_CACHE"/wasm-bindgen-*/wasm-bindgen --target web --no-typescript --out-dir "$OUT" \
  "$TARGET_DIR/wasm32-unknown-unknown/release/ui_leptos.wasm"
WASM_OPT_FEATURES=(--enable-reference-types --enable-bulk-memory --enable-nontrapping-float-to-int
  --enable-sign-ext --enable-multivalue)
"$TRUNK_CACHE"/wasm-opt-*/bin/wasm-opt -Oz "${WASM_OPT_FEATURES[@]}" \
  "$OUT/ui_leptos_bg.wasm" -o "$OUT/optimized.wasm"
"$TRUNK_CACHE"/wasm-opt-*/bin/wasm-opt -Oz -g "${WASM_OPT_FEATURES[@]}" \
  "$OUT/ui_leptos_bg.wasm" -o "$OUT/named.wasm"
printf '%s: raw %s, optimized %s, gzip -9 %s\n' "$NAME" \
  "$(wc -c <"$TARGET_DIR/wasm32-unknown-unknown/release/ui_leptos.wasm")" \
  "$(wc -c <"$OUT/optimized.wasm")" \
  "$(gzip -9 -c "$OUT/optimized.wasm" | wc -c)"
