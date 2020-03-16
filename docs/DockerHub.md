# Overview
[Replicante](https://www.replicante.io/) is a centralised datastore orchestrator.

Agents are the interface between the centralised platform and each datastore.

The following agents are packaged inside the images in this repo:

  * MongoDB
  * Zookeeper


## Usage
Agents can be started with the following command:
```bash
docker run --rm -it --init \
  -v "/path/to/config.yaml:/home/replicante/agent-SOFTWARE.yaml" \
  replicanteio/agents:v0 \
  replicante-agent-$SOFTWARE
```

See the tags for possible versions.
In addition to the exact `vX.Y.Z` version, tags in the format `vX.Y` and `vX` refer to the
latest release for a specific minor version or a specific major version.
The tag `latest` is also available.

The possible values of `SOFTWARE` are the supported datastores:

  * `mongodb`
  * `zookeeper`


## Init on Podman
Podman defaults to [catatonit](https://github.com/openSUSE/catatonit) as the `--init` process.
This package is currently [not packaged](https://github.com/containers/libpod/issues/4159), at least for fedora.

Until catatonit is packaged with podman you can use this work around:

  1. Install a catatonit [release](https://github.com/openSUSE/catatonit/releases) or any container init process.
  2. Place it in a place podman will find it:
     * Podman looks at `/usr/libexec/podman/catatonit`
     * Set the `init_path` configuration option to your location.


## Links
For more information, the following links may be useful:

  * [Official website](https://www.replicante.io/)
  * [GitHub repo](https://github.com/replicante-io/agents)
  * [Full documentation](https://www.replicante.io/docs/)
