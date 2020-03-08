#!/usr/bin/env bash
set -ex

function ci_crate {
  cargo build --manifest-path "$1"
  cargo test --manifest-path "$1"
  cargo clippy --manifest-path "$1" -- -D warnings
  cargo fmt --manifest-path "$1" --verbose -- --check
}


ci_crate 'libs/rust/sdk/Cargo.toml'
ci_crate 'agents/kafka/Cargo.toml'
ci_crate 'agents/mongodb/Cargo.toml'
ci_crate 'agents/zookeeper/Cargo.toml'
