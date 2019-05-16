use std::fs::File;
use std::io::Read;
use std::path::Path;

use failure::ResultExt;
use serde_yaml;

use replicante_agent::config::APIConfig;
use replicante_agent::config::Agent;
use replicante_agent::Result;

use super::error::ErrorKind;

/// Zookeeper Agent configuration
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct Config {
    /// Common agent options.
    #[serde(default)]
    pub agent: Agent,

    /// Zookeeper related options.
    pub zookeeper: Zookeeper,
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
        APIConfig::set_default_bind("127.0.0.1:3181".into())
    }
}

/// Zookeeper related options.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct Zookeeper {
    /// Name of the zookeeper cluster.
    pub cluster: String,

    /// Host and port (in host:port format) of the zookeeper 4lw server.
    #[serde(default = "Zookeeper::default_target")]
    pub target: String,
}

impl Zookeeper {
    pub fn default_target() -> String {
        "localhost:2181".into()
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
        let cursor = Cursor::new("{zookeeper: {cluster: test}}");
        Config::from_reader(cursor).unwrap();
    }
}
