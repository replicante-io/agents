use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde_yaml;

use replicante_agent::Result;
use replicante_agent::config::Agent;
use replicante_agent::config::APIConfig;


/// Kafka Agent configuration
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Config {
    /// Common agent options.
    #[serde(default)]
    pub agent: Agent,

    /// Kafka related options.
    pub kafka: Kafka,
}

impl Config {
    /// Loads the configuration from the given [`std::fs::File`].
    ///
    /// [`std::fs::File`]: https://doc.rust-lang.org/std/fs/struct.File.html
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Config> {
        let config = File::open(path)?;
        Config::from_reader(config)
    }

    /// Loads the configuration from the given [`std::io::Read`].
    ///
    /// [`std::io::Read`]: https://doc.rust-lang.org/std/io/trait.Read.html
    pub fn from_reader<R: Read>(reader: R) -> Result<Config> {
        let conf = serde_yaml::from_reader(reader)?;
        Ok(conf)
    }
}

impl Config {
    /// Override the base agent default configuration options.
    pub fn override_defaults() {
        APIConfig::set_default_bind("127.0.0.1:10092".into())
    }
}


/// Kafka related options.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Kafka {
    /// Name of the kafka cluster.
    pub cluster: String,

    /// Addresses used to locate the kafka services.
    #[serde(default)]
    pub target: KafkaTarget,
}


/// Kafka server listening locations.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct KafkaTarget {
    /// Address "host:port" of the zafka broker.
    #[serde(default = "KafkaTarget::default_broker")]
    pub broker: String,

    /// Address "host:port" of the JMX server.
    #[serde(default = "KafkaTarget::default_jmx")]
    pub jmx: String,

    /// Addresses "host:port" of the zookeeper ensamble.
    #[serde(default = "KafkaTarget::default_zookeeper")]
    pub zookeeper: Vec<String>,
}

impl KafkaTarget {
    fn default_broker() -> String { "localhost:9092".into() }
    fn default_jmx() -> String { "localhost:9999".into() }
    fn default_zookeeper() -> Vec<String> { vec!["localhost:2818".into()] }
}

impl Default for KafkaTarget {
    fn default() -> Self {
        KafkaTarget {
            broker: KafkaTarget::default_broker(),
            jmx: KafkaTarget::default_jmx(),
            zookeeper: KafkaTarget::default_zookeeper(),
        }
    }
}


#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use super::Config;

    #[test]
    #[should_panic(expected = "invalid type: string")]
    fn from_reader_error() {
        let cursor = Cursor::new("some other text");
        Config::from_reader(cursor).unwrap();
    }

    #[test]
    fn from_reader_ok() {
        let cursor = Cursor::new("{kafka: {cluster: test}}");
        Config::from_reader(cursor).unwrap();
    }
}
