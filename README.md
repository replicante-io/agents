# Replicante Agents
Replicante uses agents to interface with datastores.
This repository stores the core rust agent framework as well official agents.


## Supported agents

  * Kafka 1.0+
  * MongoDB 3.0+ (Replica Set mode)
  * MongoDB 3.2+ (Sharded Cluster mode)
  * Zookeeper 3.3+


## Building agents
This repo contains the base agent library (a cargo crate used to build agents)
as well as the official replicante agents.

Official agents are written in rust and built with cargo:
```bash
git clone https://github.com/replicante-io/agents.git .
cargo build --release
```

### Excluded agents
Agents that have build dependencies outside of the usual rust ecosystem are NOT part of the root
workspace but instead have their own workspace.
This is done to avoid extra burdens to the most common cases leaving extra dependencies only to
those that need them.

The following is a list of agents that have extra dependencies:

  * `kafka`: required Java (to act as a JMX client).


## Docker image
A docker image including most agents in this repo can be built with the following command:
```bash
docker build --force-rm --tag replicanteio/agents:v$VERSION .
```

Agents that require external dependencies or large runtimes, for example Java, are provided
as separate images:

  * For kafka use `replicanteio/agent-kafka:vVERSION`

The image can be used to run any of the agents as long as a configration file is provided:
```bash
docker run --rm -it \
  -v "$PWD/agent-mongodb.example.yaml:/home/replicante/agent-mongodb.yaml" \
  -w /home/replicante replicanteio/agents:v0.2.0 \
  replicante-agent-mongodb
```


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
