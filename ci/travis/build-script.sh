#!/usr/bin/env bash
set -ex

cargo build
cargo test 
cargo clippy -- -D warnings
cargo fmt --verbose -- --check

# Kafka is special ...
cd kafka/
cargo build
cargo test
cargo clippy -- -D warnings
cargo fmt --verbose -- --check
