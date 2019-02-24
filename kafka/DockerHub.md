# Overview
[Replicante](https://www.replicante.io/) is a centralised monitoring and management tool.

Agents are the interface between the centralised platform and each datastore.

This image is for the [Kafka](https://kafka.apache.org/) agent.


## Usage
The agent can be started with the following command:
```bash
docker run --rm -it \
  -v "/path/to/config.yaml:/home/replicante/agent-kafka.yaml" \
  -w /home/replicante replicanteio/agent-kafka:v0.2
```

See the tags for possible versions.
In addition to specific version, tags in the format `vX.Y` and `vX` refer to the latest
release for a specific minor version or a specific major version.
The tag `latest` is also available.


## More
For more information, the following links may be useful:

  * [Official website](https://www.replicante.io/)
  * [GitHub repo](https://github.com/replicante-io/agents)
  * [Full documentation](https://www.replicante.io/docs/agents/docs/intro/)
