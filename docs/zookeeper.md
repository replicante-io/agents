# Zookeeper
[Zookeeper](https://zookeeper.apache.org/) is a centralized service for maintaining configuration
information, naming, providing distributed synchronization, and providing group services.


## Supported versions
| Zookeeper Agent Version | Zookeeper Version |
| ----------------------- | ----------------- |
| 0.1.0+                  | 3.3+              |


## Installation from code
Follow the instructions in the [installation](base/install.md) page.
Once the agent is available in `$PATH` check the usage:

```bash
$ replicante-agent-zookeeper --help
MongoDB Replicante Agent 0.1.0 [4cc413ece34e9492e2c61f46ed91243c7d4d57b4; working directory tainted]
Replicante agent for Zookeeper

USAGE:
    replicante-agent-zookeeper [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --config <FILE>    Specifies the configuration file to use [default: agent-zookeeper.yaml]
```

## Configuration
[import, lang:"yaml"](../agent-zookeeper.example.yaml)


## Upgrades notes
See the [full changelog](https://github.com/replicante-io/agents/blob/master/zookeeper/CHANGELOG.md)
for all details.
