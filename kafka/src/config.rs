use std::fs::File;
use std::io::Read;
use std::path::Path;

use failure::ResultExt;
use serde_yaml;

use replicante_agent::config::APIConfig;
use replicante_agent::config::Agent;
use replicante_agent::Result;

use super::error::ErrorKind;

/// Kafka Agent configuration
#[derive(Clone, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Config {
    /// Common agent options.
    #[serde(default)]
    pub agent: Agent,

    /// Kafka related options.
    #[serde(default)]
    pub kafka: Kafka,
}

impl Config {
    /// Loads the configuration from the given [`std::fs::File`].
    ///
    /// [`std::fs::File`]: https://doc.rust-lang.org/std/fs/struct.File.html
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Config> {
        let path_for_error = path.as_ref().to_str().unwrap_or("<utf8 error>").to_string();
        let config = File::open(path).with_context(|_| ErrorKind::Io(path_for_error))?;
        Config::from_reader(config)
    }

    /// Loads the configuration from the given [`std::io::Read`].
    ///
    /// [`std::io::Read`]: https://doc.rust-lang.org/std/io/trait.Read.html
    pub fn from_reader<R: Read>(reader: R) -> Result<Config> {
        let conf = serde_yaml::from_reader(reader).with_context(|_| ErrorKind::ConfigLoad)?;
        Ok(conf)
    }

    /// Apply transformations to the configuration to derive some parameters.
    ///
    /// Transvormation:
    ///
    ///   * Apply verbose debug level logic.
    pub fn transform(mut self) -> Self {
        self.agent = self.agent.transform();
        self
    }
}

impl Config {
    /// Override the base agent default configuration options.
    pub fn override_defaults() {
        APIConfig::set_default_bind("127.0.0.1:10092".into())
    }
}

/// Kafka related options.
#[derive(Clone, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Kafka {
    /// Addresses used to locate the kafka services.
    #[serde(default)]
    pub target: KafkaTarget,
}

/// Kafka server listening locations.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct KafkaTarget {
    /// Kafka broker configuration.
    #[serde(default)]
    pub broker: BrokerTarget,

    /// Address "host:port" of the JMX server.
    #[serde(default = "KafkaTarget::default_jmx")]
    pub jmx: String,

    /// Zookeeper ensamble for the Kafka cluster.
    #[serde(default)]
    pub zookeeper: ZookeeperTarget,
}

impl KafkaTarget {
    fn default_jmx() -> String {
        "localhost:9999".into()
    }
}

impl Default for KafkaTarget {
    fn default() -> Self {
        KafkaTarget {
            broker: BrokerTarget::default(),
            jmx: KafkaTarget::default_jmx(),
            zookeeper: ZookeeperTarget::default(),
        }
    }
}

/// Kafka server location.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct BrokerTarget {
    /// Addresses "host:port" of the zookeeper ensamble.
    #[serde(default = "BrokerTarget::default_uri")]
    pub uri: String,

    /// Network timeout for requests to Kafka.
    #[serde(default = "BrokerTarget::default_timeout")]
    pub timeout: u64,
}

impl BrokerTarget {
    fn default_uri() -> String {
        "localhost:9092".into()
    }
    fn default_timeout() -> u64 {
        10
    }
}

impl Default for BrokerTarget {
    fn default() -> Self {
        BrokerTarget {
            uri: BrokerTarget::default_uri(),
            timeout: BrokerTarget::default_timeout(),
        }
    }
}

/// Kafka's cluster Zookeeper server location.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct ZookeeperTarget {
    /// Addresses "host:port" of the zookeeper ensamble.
    #[serde(default = "ZookeeperTarget::default_uri")]
    pub uri: String,

    /// Zookeeper session timeout.
    #[serde(default = "ZookeeperTarget::default_timeout")]
    pub timeout: u64,
}

impl ZookeeperTarget {
    fn default_uri() -> String {
        "localhost:2818".into()
    }
    fn default_timeout() -> u64 {
        10
    }
}

impl Default for ZookeeperTarget {
    fn default() -> Self {
        ZookeeperTarget {
            uri: ZookeeperTarget::default_uri(),
            timeout: ZookeeperTarget::default_timeout(),
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
