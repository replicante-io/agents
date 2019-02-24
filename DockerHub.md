# Overview
[Replicante](https://www.replicante.io/) is a centralised monitoring and management tool.

Agents are the interface between the centralised platform and each datastore.

The following agents are packaged inside the images in this repo:

  * Kafka
  * MongoDB
  * Zookeeper


## Usage
Agents can be started with the following command:
```bash
docker run --rm -it \
  -v "/path/to/config.yaml:/home/replicante/agent-SOFTWARE.yaml" \
  -w /home/replicante replicanteio/agents:v0 \
  replicante-agent-SOFTWARE
```

See the tags for possible versions.
In addition to specific version, tags in the format `vX.Y` and `vX` refer to the latest
release for a specific minor version or a specific major version.
The tag `latest` is also available.

The possible values of `SOFTWARE` are the supported datastores:

  * `kafka`
  * `mongodb`
  * `zookeeper`


## More
For more information, the following links may be useful:

  * [Official website](https://www.replicante.io/)
  * [GitHub repo](https://github.com/replicante-io/agents)
  * [Full documentation](https://www.replicante.io/docs/agents/docs/intro/)
