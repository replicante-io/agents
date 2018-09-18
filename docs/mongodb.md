# MongoDB
[MongoDB](https://www.mongodb.com/) is a felxible document NoSQL database.


## Supported versions
| Agent Version | MongoDB Version | MongoDB Mode            |
| ------------- | --------------- | ----------------------- |
| 0.2.0+        | 3.0+ / 3.2+     | Replica Set / Clustered |
| 0.1.0+        | 3.2+            | Replica Set             |


## Installation from code
Follow the instructions in the [installation](base/install.md) page.
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
[import, lang:"yaml"](../agent-mongodb.example.yaml)


## Upgrades notes
See the [full changelog](https://github.com/replicante-io/agents/blob/master/mongodb/CHANGELOG.md)
for all details.

### Upgrading to 0.2.0
- The API format for `/api/v1/shards` has changed (this would be a breaking change afer the 1.0 release).
- The configuration format was changed and existing files may not work.
