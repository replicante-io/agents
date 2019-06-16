---
id: version-0.4.0-intro-install
title: Installation
sidebar_label: Installation
original_id: intro-install
---

Official agents can be installed from code or from pre-built binaries as described below.

Docker images are also provided but mainly meant as an experimentation tool.
Access to the datastore process and file system will be needed for some features
to work and container isolation will interfere with them.


## 1. Install

### From pre-built binaries
Pre-built binaries are helpful for users to get up and running quickly.
Unfortunately they require a good deal of effort from the community to be available for all
popular distributions that people my want to use.
The Replicante Community cannot afford to provide pre-built binaries for all popular
Linux distributions at this stage but we do want to provide something to make things
easier on people.

Pre-built binaries are available but they may not work for your system, in which case
we suggest you use an alternative install method.

```bash
# Grab the binaries for the version of your choice from GitHub:
VERSION=vX.Y.Z
wget https://github.com/replicante-io/agents/releases/download/$VERSION/checksum.txt
wget https://github.com/replicante-io/agents/releases/download/$VERSION/replicante-agent-kafka.tar.gz-linux-64bits
wget https://github.com/replicante-io/agents/releases/download/$VERSION/replicante-agent-mongodb-linux-64bits
wget https://github.com/replicante-io/agents/releases/download/$VERSION/replicante-agent-zookeeper-linux-64bits

# Verify the integrity of the binaries:
sha256sum --check checksum.txt

# Unpack the kafka agent:
mkdir -p kafka
pushd kafka
tar --extract --file ../replicante-agent-kafka.tar.gz-linux-64bits
popd

# Verify the binaries work:
mv replicante-agent-mongodb-linux-64bits replicante-agent-mongodb
mv replicante-agent-zookeeper-linux-64bits replicante-agent-zookeeper
chmod +x replicante-agent-mongodb replicante-agent-zookeeper
./replicante-agent-mongodb --version
./replicante-agent-zookeeper --version
# NOTE: the kafka agent needs access to JVM libraries.
kafka/replicante-agent-kafka --version
```


### From code
The following instructions where executed on a clean Fedora 30 install
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

# Due to additional dependencies of the kafka agent alone,
# it is managed as a separate project in the same repo.
# To build it:
cd kafka/
cargo build --release
# NOTE: the kafka agent needs access to JVM libraries and build artifacts.
target/release/replicante-agent-kafka --version
```

You can now install the desired agents by copying the build target to your preferred location.


### With Docker
Docker images with the officail agents pre-compiled are also available.

  * For kafka agent use: https://hub.docker.com/r/replicanteio/agent-kafka
  * For other agents: https://hub.docker.com/r/replicanteio/agents

To check the image works as expected:
```bash
docker pull replicanteio/agent-kafka:v0
docker run --rm -it replicanteio/agent-kafka:v0 replicante-agent-kafka --version

docker pull replicanteio/agents:v0
docker run --rm -it replicanteio/agents:v0 replicante-agent-mongodb --version
docker run --rm -it replicanteio/agents:v0 replicante-agent-zookeeper --version
```
