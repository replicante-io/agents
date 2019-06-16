---
id: version-0.4.0-agents-zookeeper
title: Zookeeper
sidebar_label: Zookeeper
original_id: agents-zookeeper
---

[Zookeeper](https://zookeeper.apache.org/) is a centralized service for maintaining configuration
information, naming, providing distributed synchronization, and providing group services.


## Supported versions
| Agent Version | Zookeeper Version |
| ------------- | ----------------- |
| 0.1.0+        | 3.3+              |


## Install
Follow the instructions in the [installation](intro-install.md) page.
Once the agent is available in `$PATH` check the usage:

```bash
$ replicante-agent-zookeeper --help
Zookeeper Replicante Agent 0.1.0 [01c4992a74a7331810e92a808b5cc3fec2b02635; working directory tainted]
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
```yaml
# Common agents options described in agent.example.yaml
agent: {}
  # ... snip ...


# Zookeeper specific configuration.
zookeeper:
  # Name of the zookeeper cluster.
  # *** Required ***
  #cluster: <CLUSTER_NAME>

  # Host and port (in host:port format) of the zookeeper 4lw server.
  target: "localhost:2181"
```


## Upgrades notes
See the [full changelog](https://github.com/replicante-io/agents/blob/master/zookeeper/CHANGELOG.md)
for all details.
