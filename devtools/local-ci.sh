#!/bin/bash
set -e

for_version() {
  version="$1"
  full_mode=""
  if [ "${version}" == "stable" ]; then
    full_mode="--full"
  fi

  echo "Clean up workspaces for version ${version}"
  rustup run "${version}" cargo clean --manifest-path libs/rust/sdk/Cargo.toml
  rustup run "${version}" cargo clean --manifest-path agents/kafka/Cargo.toml
  rustup run "${version}" cargo clean --manifest-path agents/mongodb/Cargo.toml
  rustup run "${version}" cargo clean --manifest-path agents/zookeeper/Cargo.toml

  echo "Run CI for version ${version}"
  rustup run "${version}" ci/check-workspace.sh ${full_mode} "Legacy SDK" libs/rust/sdk/Cargo.toml
  rustup run "${version}" ci/check-workspace.sh ${full_mode} Kafka agents/kafka/Cargo.toml
  rustup run "${version}" ci/check-workspace.sh ${full_mode} MongoDB agents/mongodb/Cargo.toml
  rustup run "${version}" ci/check-workspace.sh ${full_mode} Zookeeper agents/zookeeper/Cargo.toml
}

for_version "stable"
for_version "1.60.0"
for_version "nightly"
