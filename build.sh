#!/bin/bash

BINARY="code-commit"

script_status=0
handle_error() {
    script_status=1
}
trap 'handle_error' ERR

# --no-test=pass tells nextest to exit with code 0 if there are no tests
cargo fmt
cargo build
cargo nextest run --no-tests=pass --no-fail-fast
cargo nextest run --no-tests=pass --no-fail-fast -- --ignored
# reminder: println! calls must use inline named arguments
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
cargo install --path .

exit $script_status
