---
id: agents-mongodb
title: MongoDB
sidebar_label: MongoDB
---

[MongoDB](https://www.mongodb.com/) is a felxible document NoSQL database.


## Supported versions
| Agent Version | MongoDB Version | MongoDB Mode            |
| ------------- | --------------- | ----------------------- |
| 0.2.0+        | 3.0+ / 3.2+     | Replica Set / Clustered |
| 0.1.0+        | 3.2+            | Replica Set             |


## Installation from code
Follow the instructions in the [installation](intro-install.md) page.
Once the agent is available in `$PATH` check the usage:

```bash
$ replicante-agent-mongodb --help
MongoDB Replicante Agent 0.2.0 [4cc413ece34e9492e2c61f46ed91243c7d4d57b4; working directory tainted]
Replicante agent for MongoDB

USAGE:
    replicante-agent-mongodb [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --config <FILE>    Specifies the configuration file to use [default: agent-mongodb.yaml]
```


## Configuration
```yaml
# Common agents options described in agent.example.yaml
agent: {}
  # ... snip ...


# MongoDB specific configuration.
mongo:
  # MongoDB connection URI.
  uri: "mongodb://localhost:27017"

  # Timeout (in milliseconds) for selecting an appropriate server for operations.
  timeout: 30000

  # Configure the agent to operate in sharded cluster mode.
  #
  # This section is optional.
  # If missing, sharding mode is disabled.
  # If present, sharding mode is enabled by default but can be disabled.
  sharding:
    # The identifier of the MongoDB sharded cluster.
    # *** Required ***
    #
    # In replica set mode the cluster name is detected as the replica set.
    # In sharded mode this attribute cannot be auto-detected and must be specified.
    cluster_name: 'user-defined-name'

    # Enable or disable sharded mode.
    enable: true

    # Name of the `mongos` node name.
    #
    # If set, the node is expected to be a mongos instance.
    # If null (the default), the node is expected to be a mongod instance.
    mongos_node_name: ~
```


## Upgrades notes
See the [full changelog](https://github.com/replicante-io/agents/blob/master/mongodb/CHANGELOG.md)
for all details.

### Upgrading to 0.2.0
- The API format for `/api/v1/shards` has changed (this would be a breaking change afer the 1.0 release).
- The configuration format was changed and existing files may not work.
