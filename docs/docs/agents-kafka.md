---
id: agents-kafka
title: Kafka
sidebar_label: Kafka
---

[Kafka](https://kafka.apache.org/) is a distributed streaming platform.


## Supported versions
| Agent Version | Kafka Version |
| ------------- | ------------- |
| 0.1.0+        | 1.0+          |


## Install
Follow the instructions in the [installation](intro-install.md) page.
Once the agent is available in `$PATH` check the usage:

```bash
$ replicante-agent-kafka --help
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
```yaml
# Common agents options described in agent.example.yaml
agent: {}
  # ... snip ...


# Kafka specific configuration.
kafka:
  # Addresses used to locate the kafka services.
  target:
    # Kafka broker configuration.
    broker:
      # Address "host:port" of the kafka broker.
      uri: 'localhost:9092'

      # Network timeout for requests to Kafka.
      timeout: 10

    # Address "host:port" of the JMX server.
    #
    # By default kafka does not expose the JMX server.
    # To do so, set the `JMX_PORT` environment variable before starting the server.
    # For additional options see:
    #   https://github.com/apache/kafka/blob/1.1.1/bin/kafka-run-class.sh#L166-L174
    jmx: 'localhost:9999'

    # Zookeeper ensamble for the Kafka cluster.
    zookeeper:
      # Addresses "host:port" of the zookeeper ensamble.
      uri: 'localhost:2181'

      # Zookeeper session timeout.
      timeout: 10
```


## Upgrades notes
See the [full changelog](https://github.com/replicante-io/agents/blob/master/kafka/CHANGELOG.md)
for all details.
