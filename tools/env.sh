# Source it (`. tools/env.sh`): Rust tools installed INSIDE the repository, nothing in the home directory.
# Tools live in `.toolchain/` (ignored by Git).
GYM_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.." && pwd)"
export GYM_ROOT
export RUSTUP_HOME="$GYM_ROOT/.toolchain/rustup"
export CARGO_HOME="$GYM_ROOT/.toolchain/cargo"
# Trunk downloads wasm-bindgen and wasm-opt into the XDG cache: keep it in the repository too.
export XDG_CACHE_HOME="$GYM_ROOT/.toolchain/cache"
export PATH="$CARGO_HOME/bin:$GYM_ROOT/.toolchain/bin:$PATH"
