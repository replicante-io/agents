use std::fs::File;
use std::io::Read;
use std::path::Path;

use failure::ResultExt;
use serde_derive::Deserialize;
use serde_derive::Serialize;

use replicante_agent::config::APIConfig;
use replicante_agent::config::Agent;
use replicante_agent::Result;

use super::error::ErrorKind;

/// MongoDB Agent configuration
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct Config {
    /// Common agent options.
    pub agent: Agent,

    /// MongoDB options.
    #[serde(default)]
    pub mongo: MongoDB,
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

    /// Return a mocked configuration.
    #[cfg(test)]
    pub fn mock() -> Config {
        Config {
            agent: Agent::mock(),
            mongo: MongoDB::default(),
        }
    }
}

impl Config {
    /// Override the base agent default configuration options.
    pub fn override_defaults() {
        APIConfig::set_default_bind("127.0.0.1:37017".into())
    }
}

/// MongoDB related options.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct MongoDB {
    /// MongoDB connection URI.
    #[serde(default = "MongoDB::default_uri")]
    pub uri: String,

    /// Configure MongoDB sharding mode.
    #[serde(default)]
    pub sharding: Option<Sharding>,

    /// Timeout (in milliseconds) for selecting an appropriate server for operations.
    #[serde(default = "MongoDB::default_timeout")]
    pub timeout: i64,
}

impl Default for MongoDB {
    fn default() -> Self {
        MongoDB {
            uri: Self::default_uri(),
            sharding: None,
            timeout: Self::default_timeout(),
        }
    }
}

impl MongoDB {
    /// Default value for `uri` used by serde.
    fn default_uri() -> String {
        String::from("mongodb://localhost:27017")
    }

    /// Default value for `timeout` used by serde.
    fn default_timeout() -> i64 {
        1000
    }
}

/// Configure the agent to operate in sharded cluster mode.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct Sharding {
    /// The identifier of the MongoDB sharded cluster.
    pub cluster_name: String,

    /// Enable or disable sharded mode.
    #[serde(default = "Sharding::default_enable")]
    pub enable: bool,

    /// Name of the `mongos` node name.
    ///
    /// If set, the node is expected to be a mongos instance.
    /// If null (the default), the node is expected to be a mongod instance.
    #[serde(default)]
    pub mongos_node_name: Option<String>,
}

impl Sharding {
    /// Default value for `enable` used by serde.
    fn default_enable() -> bool {
        true
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
        let cursor = Cursor::new("agent: {db: 'test.db'}");
        Config::from_reader(cursor).unwrap();
    }
}
