# Kafka Agent
The Kafka agent uses a JMX client to fetch some information from the kafka processes.

To do so, it uses the [`jmx`](https://crates.io/crates/jmx) crate.
The `jmx` crate uses the [`j4rs`](https://crates.io/crates/j4rs) Java bindings to provide
rust code access to Java's built-in JMX client.

Because of this, the kafka agent needs access to the JDK at compile time
and to Java libraries at runtime.

To use any cargo command `JAVA_HOME` and `LD_LIBRARY_PATH` must be set.
To execute the agent only `LD_LIBRARY_PATH` is required.

```bash
# Fedora 28
dnf install java-1.8.0-openjdk-devel
export JAVA_HOME=$(find /usr/lib/jvm/java-1.8.0-openjdk-1.8.0* -maxdepth 0 | tail -n1)
export LD_LIBRARY_PATH="${JAVA_HOME}/jre/lib/amd64/server:$LD_LIBRARY_PATH"
cargo test
```


## Docker image
Since Kafka requires Java at runtime, a dedicated image is provided to limit the impact
of the Java requirements on users that will not need it.

To build the Kafka agent image, from the root of the repo:
```bash
docker build --force-rm -f kafka/Dockerfile --tag replicanteio/agent-kafka:v$VERSION .
```

The image can be used as long as a configration file is provided:
```bash
docker run --rm -it \
  -v "$PWD/kafka/agent-kafka.example.yaml:/home/replicante/agent-kafka.yaml" \
  -w /home/replicante replicanteio/agent-kafka:v0.2
```


## Playground DNS resolution
Kafka provides an internal service discovery based on broadcast of host names.
The playground nodes broadcast themselves as `node1`, `node2`, and `node3`.

For a client to be able to use kafka in the playgrounds, it needs to be able to resolve
those names to the correct nodes.
This happens correctly for nodes inside the playground network but not for the docker host.

One possible solution is to add the IP/hostname to `/etc/hosts` temporarely.
The below command prints the mapping that should be added:

```bash
docker inspect \
  --format '{{ .NetworkSettings.Networks.playgrounds_kafka.IPAddress }} {{ .Config.Hostname }}' \
  kafka_node1_1 kafka_node2_1 kafka_node3_1 kafka_zoo_1

# Add to hosts file
sudo vim /etc/hosts
```
