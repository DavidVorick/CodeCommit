#!/bin/bash
cargo fmt
cargo build
cargo nextest run
cargo nextest run -- --ignored
cargo clippy -- -D warnings
cargo build --release
