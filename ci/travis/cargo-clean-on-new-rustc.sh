#!/bin/bash
# From crates.io:
# https://github.com/rust-lang/crates.io/blob/master/script/ci/cargo-clean-on-new-rustc-version.sh
set -ex

# Support cleaning multiple projects
function cargo_clean {
  cargo clean --manifest-path libs/rust/sdk/Cargo.toml
  cargo clean --manifest-path agents/kafka/Cargo.toml
  cargo clean --manifest-path agents/mongodb/Cargo.toml
  cargo clean --manifest-path agents/zookeeper/Cargo.toml
}


# Change this to force a clean build on CI
manual_stamp=2020-04-29-0
manual_stamp_file=target/ci_manual_stamp

if [ -f $manual_stamp_file ]; then
    if echo "$manual_stamp" | cmp -s $manual_stamp_file -; then
        : # Do nothing, fall through to version check below
    else
        echo "A clean build has been requested, running cargo clean"
        cargo_clean
    fi
else
    echo "Existing stamp not found, running cargo clean"
    cargo_clean
fi


# If `cargo clean` was run above, then the target/ directory is now
# gone and the messages below will not be printed
stamp_file=target/rustc_version_stamp
current_version=$(rustc --version)

if [ -f $stamp_file ]; then
    # Compare the current version against the previous version
    if echo "$current_version" | cmp -s $stamp_file -; then
        echo "Version of rustc hasn't changed, keeping the cache intact"
    else
        echo "The version of rustc has changed, running cargo clean"
        cargo_clean
    fi
fi


# Save the version stamps for next time
mkdir -p target/
echo $current_version > $stamp_file
echo $manual_stamp > $manual_stamp_file
