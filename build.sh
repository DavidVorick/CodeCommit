#!/bin/bash

script_status=0
handle_error() {
    script_status=1
}
trap 'handle_error' ERR

cargo fmt
cargo build

all_tests=$(cargo test -- --list 2>/dev/null | grep ': test$' | cut -d':' -f1 | sort)
ignored_tests=$(cargo test -- --list --ignored 2>/dev/null | grep ': test$' | cut -d':' -f1 | sort)
non_ignored_tests=$(comm -23 <(echo "$all_tests") <(echo "$ignored_tests"))

if [ -n "$non_ignored_tests" ]; then
    cargo nextest run
fi
if [ -n "$ignored_tests" ]; then
    cargo nextest run -- --ignored
fi

cargo clippy -- -D warnings
cargo build --release

exit $script_status
