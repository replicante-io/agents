#!/usr/bin/env bash
set -ex

cargo build --verbose
cargo test --verbose
cargo clippy --verbose -- -D warnings
# Code format is optional until we can make it work.
cargo fmt --verbose -- --check

# Kafka is special ...
cd kafka/
cargo build --verbose
cargo test --verbose
cargo clippy --verbose -- -D warnings
# Code format is optional until we can make it work.
cargo fmt --verbose -- --check || true
