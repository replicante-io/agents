# Kafka
[Kafka](https://kafka.apache.org/) is a distributed streaming platform.


## Supported versions
| Agent Version | Kafka Version |
| ------------- | ------------- |
| 0.1.0+        | 1.0+          |


## Installation from code
The Kafka agent uses an embedded JVM instance to access remote JMX data.
This means compiling the Kafka agent has extra requirements as compared to other agents.

To avoid the extra burdens of Java dependencies on other agents, the Kafka agent is not part
of the root workspace that includes all other agents.

The following instructions where executed on a clean Fedora 28 install
but should work for any Linux system:
```bash
# Install needed tools and dependencies.
dnf install cmake gcc git make openssl-devel java-1.8.0-openjdk-devel
export JAVA_HOME=$(find /usr/lib/jvm/java-1.8.0-openjdk-1.8.0* | head -n1)
export LD_LIBRARY_PATH="${JAVA_HOME}/jre/lib/amd64/server:$LD_LIBRARY_PATH"

# Install rust and cargo with rustup.
curl https://sh.rustup.rs -sSf | sh
source $HOME/.cargo/env

# Get the code and compile replicante.
git clone --recursive https://github.com/replicante-io/agents.git
cd agents/kafka
# To install a specific VERSION uncomment the command below.
# By default the latest DEVELOPMENT version is compiled.
# Production users should instead switch to the latest STABLE release.
#git checkout v<VERSION>
cargo build --release

# Ensure the built binaries work.
$ target/release/replicante-agent-kafka --help
Kafka Replicante Agent 0.1.0 [01c4992a74a7331810e92a808b5cc3fec2b02635; not tainted]
Replicante agent for Kafka

USAGE:
    replicante-agent-kafka [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --config <FILE>    Specifies the configuration file to use [default: agent-kafka.yaml]
```

## Configuration
[import, lang:"yaml"](../kafka/agent-kafka.example.yaml)


## Upgrades notes
See the [full changelog](https://github.com/replicante-io/agents/blob/master/kafka/CHANGELOG.md)
for all details.