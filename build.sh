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

exit $script_status
