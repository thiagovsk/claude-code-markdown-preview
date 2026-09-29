#!/usr/bin/env bash
set -euo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
bin="$repo/target/release/md-preview"
export PATH="$HOME/.cargo/bin:$PATH"

needs_build() {
  [ -x "$bin" ] || return 0
  [ -n "$(find "$repo/src" "$repo/assets" "$repo/Cargo.toml" "$repo/Cargo.lock" -newer "$bin" -print -quit 2>/dev/null)" ]
}

if needs_build; then
  if ! command -v cargo >/dev/null 2>&1; then
    echo "render.sh: md-preview is not built and cargo was not found. Install Rust from https://rustup.rs, then run again." >&2
    exit 1
  fi
  echo "render.sh: building md-preview, about a minute on a first build" >&2
  cargo build --release --quiet --manifest-path "$repo/Cargo.toml" >&2
fi

exec "$bin" "$@"
