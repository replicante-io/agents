#!/usr/bin/env sh
set -ex

cargo clippy --verbose

# Kafka is special ...
cd kafka/
cargo clippy --verbose
