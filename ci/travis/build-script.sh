#!/usr/bin/env bash
set -ex

cargo build
cargo test 

# lint `clippy::cyclomatic_complexity` has been renamed to `clippy::cognitive_complexity`
# in clippy for rust 1.34.0.
# Supporting both 1.34 and 1.35 is a pain/impossible so upgrade to 1.35.
sed -i 's/cyclomatic-complexity-threshold/cognitive-complexity-threshold/' \
  ~/.cargo/registry/src/github.com-1ecc6299db9ec823/mongodb-0.3.12/clippy.toml
cargo clippy -- -D warnings

cargo fmt --verbose -- --check

# Kafka is special ...
cd kafka/
cargo build
cargo test
cargo clippy -- -D warnings
cargo fmt --verbose -- --check
