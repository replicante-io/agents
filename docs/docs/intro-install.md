---
id: intro-install
title: Installation
sidebar_label: Installation
---

Official agents can be installed from code as described below.
This is currently the only officially supported method.


## 1. Install

### From code
The following instructions where executed on a clean Fedora 28 install
but should work for any Linux system:
```bash
# Install needed tools and dependencies.
dnf install cmake gcc git make openssl-devel

# Install rust and cargo with rustup.
curl https://sh.rustup.rs -sSf | sh
source $HOME/.cargo/env

# Get the code and compile replicante.
git clone --recursive https://github.com/replicante-io/agents.git
cd agents
# To install a specific VERSION uncomment the command below.
# By default the latest DEVELOPMENT version is compiled.
# Production users should instead switch to the latest STABLE release.
#git checkout v<VERSION>
cargo build --release

# Ensure the built binaries work.
target/release/replicante-agent-mongodb --version
target/release/replicante-agent-zookeeper --version
```

You can now install the desired agents by copying the build target to your preferred location.


### With Docker
A docker image with the officail agents pre-compiled is also available:
https://hub.docker.com/r/replicanteio/agents

To check the image works as expected:
```bash
docker pull replicanteio/agents:v0
docker run --rm -it replicanteio/agents:v0 replicante-agent-kafka --version
docker run --rm -it replicanteio/agents:v0 replicante-agent-mongodb --version
docker run --rm -it replicanteio/agents:v0 replicante-agent-zookeeper --version
```
