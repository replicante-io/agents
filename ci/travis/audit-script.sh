#!/usr/bin/env bash
set -ex

cargo audit

# Kafka is special ...
cd kafka/
cargo audit
