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
cargo clippy -- -D warnings
cargo build --release
cargo install --path .

exit $script_status
