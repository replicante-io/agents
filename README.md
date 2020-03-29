# Replicante Agents
Replicante uses agents to interface with datastores.
This repository stores the core rust agent framework as well official agents.


## Code of Conduct
Our aim is to build a thriving, healthy and diverse community.  
To help us get there we decided to adopt the [Contributor Covenant Code of Conduct](https://www.contributor-covenant.org/)
for all our projects.

Any issue should be reported to [stefano-pogliani](https://github.com/stefano-pogliani)
by emailing [conduct@replicante.io](mailto:conduct@replicante.io).  
Unfortunately, as the community lucks members, we are unable to provide a second contact to report incidents to.  
We would still encourage people to report issues, even anonymously.

In addition to the Code Of Conduct below the following documents are relevant:

  * The [Reporting Guideline](https://www.replicante.io/conduct/reporting), especially if you wish to report an incident.
  * The [Enforcement Guideline](https://www.replicante.io/conduct/enforcing)


## Supported agents

  * Kafka 1.0+
  * MongoDB 3.0+ (Replica Set mode)
  * MongoDB 3.2+ (Sharded Cluster mode)
  * Zookeeper 3.3+


## Building agents
This repo contains:

  * Base agent libraries: SDKs style libraries to build agents.
    * [Rust]: `replicante_agent` SDK crate (`libs/rust/sdk`).

  * Official Replicante agents:
    * [Kafka]: `agents/kafka`.
    * [MongoDB]: `agents/mongodb`.
    * [Zookeeper]: `agents/zookeeper`.

Official agents are written in rust and built with cargo:
```bash
git clone --recursive https://github.com/replicante-io/agents.git .
cargo build --manifest-path=agents/kafka/Cargo.toml --release
cargo build --manifest-path=agents/mongodb/Cargo.toml --release
cargo build --manifest-path=agents/zookeeper/Cargo.toml --release
```


## Container image
A docker image including most agents in this repo can be built with the following command:
```bash
# When using podman, if you want to push to hub.docker.io, use --format docker.
docker build --force-rm --tag replicanteio/agents:v$VERSION .
```

Agents that require external dependencies or large runtimes, for example Java, are provided
as separate images:

  * For kafka use `replicanteio/agent-kafka:v$VERSION`

The image can be used to run any of the agents as long as a configration file is provided:
```bash
docker run --rm -it --it \
  -v "$PWD/agent-mongodb.example.yaml:/home/replicante/agent-mongodb.yaml" \
  replicanteio/agents:v0.4.1 \
  replicante-agent-mongodb
```


[Rust]: https://www.rust-lang.org/
[Kafka]: https://kafka.apache.org/
[MongoDB]: https://www.mongodb.com/
[Zookeeper]: https://zookeeper.apache.org/
