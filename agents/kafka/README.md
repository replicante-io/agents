# Kafka Agent
The Kafka agent uses a JMX client to fetch some information from the kafka processes.

To do so, it uses the [`jmx`](https://crates.io/crates/jmx) crate.
The `jmx` crate uses the [`j4rs`](https://crates.io/crates/j4rs) Java bindings to provide
rust code access to Java's built-in JMX client.

Because of this, the kafka agent needs access to the JDK at compile time
and to Java libraries at runtime.

As of `j4rs` version 0.5.1 (`jmx` 0.2.0) the JDK is located automatically.


## Docker image
Since Kafka requires Java at runtime, a dedicated image is provided to limit the impact
of the Java requirements on users that will not need it.

To build the Kafka agent image, from the root of the repo:
```bash
# When using podman, if you want to push to hub.docker.io, use --format docker.
docker build --force-rm -f kafka/Dockerfile --tag replicanteio/agent-kafka:v$VERSION .
```

The image can be used as long as a configration file is provided:
```bash
docker run --rm -it \
  -v "$PWD/kafka/agent-kafka.example.yaml:/home/replicante/agent-kafka.yaml" \
  -w /home/replicante replicanteio/agent-kafka:v0.4
```
