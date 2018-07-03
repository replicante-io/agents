use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde_yaml;

use replicante_agent::Result;
use replicante_agent::config::Agent;
use replicante_agent::config::APIConfig;


/// MongoDB Agent configuration
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Config {
    /// Common agent options.
    #[serde(default)]
    pub agent: Agent,

    /// MongoDB options.
    #[serde(default)]
    pub mongo: MongoDB,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            agent: Agent::default(),
            mongo: MongoDB::default(),
        }
    }
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
        APIConfig::set_default_bind("127.0.0.1:37017".into())
    }
}


/// MongoDB rlated options.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct MongoDB {
    /// MongoDB connection URI.
    #[serde(default = "MongoDB::default_uri")]
    pub uri: String,
}

impl Default for MongoDB {
    fn default() -> Self {
        MongoDB {
            uri: Self::default_uri(),
        }
    }
}

impl MongoDB {
    /// Default value for `bind` used by serde.
    fn default_uri() -> String { String::from("mongodb://localhost:27017") }
}


#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use replicante_agent::Error;
    use replicante_agent::ErrorKind;

    use super::Config;

    #[test]
    fn from_reader_error() {
        let cursor = Cursor::new("some other text");
        match Config::from_reader(cursor) {
            Err(Error(ErrorKind::YamlError(_), _)) => (),
            Err(err) => panic!("Unexpected error: {:?}", err),
            Ok(_) => panic!("Unexpected success!"),
        };
    }

    #[test]
    fn from_reader_ok() {
        let cursor = Cursor::new("{}");
        Config::from_reader(cursor).unwrap();
    }
}
