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
export JAVA_HOME="/usr/lib/jvm/java-1.8.0-openjdk-1.8.0.181-7.b13.fc28.x86_64"
export LD_LIBRARY_PATH="${JAVA_HOME}/jre/lib/amd64/server:$LD_LIBRARY_PATH"
cargo test
```
