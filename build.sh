#!/bin/bash

BINARY="code-commit"

script_status=0
handle_error() {
    script_status=1
}
trap 'handle_error' ERR

cargo fmt
cargo build
cargo nextest run --no-tests=pass --no-fail-fast
cargo nextest run --no-tests=pass --no-fail-fast -- --ignored
cargo clippy -- -D warnings
cargo build --release

nohup bash -c '
src="$1"
dst="$2"
while [ ! -f "$src" ]; do
  sleep 0.2
done
while ! cp "$src" "$dst" 2>/dev/null; do
  sleep 0.25
done
' _ "target/debug/$BINARY" "./$BINARY" >/dev/null 2>&1 &

exit $script_status
