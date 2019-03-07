#!/usr/bin/env sh
set -ex

export DEBIAN_FRONTEND=noninteractive
apt-get update
apt-get install -y default-jdk
