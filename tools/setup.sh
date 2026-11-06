#!/bin/bash
# Installs the project toolchain into `.toolchain/` (idempotent, can be re-run).
# Hosts contacted: sh.rustup.rs, static.rust-lang.org, github.com (+ objects.githubusercontent.com).
set -euo pipefail

RUST_TOOLCHAIN="stable"
TRUNK_VERSION="v0.22.1"

. "$(dirname "$0")/env.sh"
mkdir -p "$RUSTUP_HOME" "$CARGO_HOME" "$XDG_CACHE_HOME" "$GYM_ROOT/.toolchain/bin"

if [ ! -x "$CARGO_HOME/bin/rustup" ]; then
  echo "==> Installing rustup into $CARGO_HOME"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --no-modify-path --profile minimal --default-toolchain "$RUST_TOOLCHAIN"
fi

echo "==> Toolchain $RUST_TOOLCHAIN, wasm target, rustfmt, clippy"
rustup toolchain install "$RUST_TOOLCHAIN" --profile minimal --component rustfmt,clippy
rustup default "$RUST_TOOLCHAIN"
rustup target add wasm32-unknown-unknown

TRUNK_BIN="$GYM_ROOT/.toolchain/bin/trunk"
if [ ! -x "$TRUNK_BIN" ] || ! "$TRUNK_BIN" --version | grep -q "${TRUNK_VERSION#v}"; then
  case "$(uname -s)-$(uname -m)" in
    Linux-x86_64)  TARGET="x86_64-unknown-linux-musl" ;;   # static: independent of glibc
    Linux-aarch64) TARGET="aarch64-unknown-linux-musl" ;;
    Darwin-arm64)  TARGET="aarch64-apple-darwin" ;;
    Darwin-x86_64) TARGET="x86_64-apple-darwin" ;;
    *) echo "Unsupported system: $(uname -s)-$(uname -m)" >&2; exit 1 ;;
  esac
  echo "==> Installing Trunk $TRUNK_VERSION ($TARGET)"
  ARCHIVE="trunk-$TARGET.tar.gz"
  URL="https://github.com/trunk-rs/trunk/releases/download/$TRUNK_VERSION/$ARCHIVE"
  TMP="$(mktemp -d)"
  curl -sSfL -o "$TMP/$ARCHIVE" "$URL"
  curl -sSfL -o "$TMP/$ARCHIVE.sha256" "$URL.sha256"
  EXPECTED="$(cut -d' ' -f1 "$TMP/$ARCHIVE.sha256")"
  ACTUAL="$( (sha256sum "$TMP/$ARCHIVE" 2>/dev/null || shasum -a 256 "$TMP/$ARCHIVE") | cut -d' ' -f1)"
  [ "$EXPECTED" = "$ACTUAL" ] || { echo "Invalid Trunk checksum" >&2; exit 1; }
  tar -xzf "$TMP/$ARCHIVE" -C "$GYM_ROOT/.toolchain/bin" trunk
  rm -rf "$TMP"
fi

echo "==> Installed versions"
rustc --version
cargo --version
cargo fmt --version
cargo clippy --version
rustup target list --installed
trunk --version
