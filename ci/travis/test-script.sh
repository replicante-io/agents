#!/usr/bin/env bash
set -ex

cargo build --verbose
cargo test --verbose

# Kafka is special ...
cd kafka/
export JAVA_HOME="/usr/lib/jvm/java-8-openjdk-amd64"
export LD_LIBRARY_PATH="${JAVA_HOME}/jre/lib/amd64/server:$LD_LIBRARY_PATH"
cargo build --verbose
cargo test --verbose
