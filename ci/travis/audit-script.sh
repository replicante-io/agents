#!/usr/bin/env sh
set -ex

cargo audit

# Kafka is special ...
cd kafka/
cargo audit
