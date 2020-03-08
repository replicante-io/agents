#!/usr/bin/env bash
set -ex

cargo audit --file libs/rust/sdk/Cargo.lock
cargo audit --file agents/kafka/Cargo.lock
cargo audit --file agents/mongodb/Cargo.lock
cargo audit --file agents/zookeeper/Cargo.lock
