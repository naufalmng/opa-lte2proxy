#!/usr/bin/env bash
set -e
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$DIR"

BINARY="$DIR/target/release/opa-modem2proxy"

if [[ ! -x "$BINARY" ]]; then
  echo "Building release binary..."
  source "$HOME/.cargo/env"
  cargo build --release
fi

exec "$BINARY" "$@"
