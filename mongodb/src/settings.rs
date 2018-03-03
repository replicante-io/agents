use std::collections::HashMap;
use std::convert::From;
use std::result::Result;
use std::vec::Vec;

use config::Config;
use config::ConfigError;
use config::File;
use config::Value;

use replicante_agent::config::AgentConfig;


/// Stores all settings for the MongoDB agent.
#[derive(Debug)]
pub struct MongoDBAgentSettings {
    conf: Config
}

impl Default for MongoDBAgentSettings {
    /// Generate a default configuration for the MongoDB agent.
    fn default() -> MongoDBAgentSettings {
        let mut agent = AgentConfig::default();
        agent.server.bind = String::from("localhost:37017");

        let mongo = MongoDBSettings::default();
        let mut settings = Config::default();
        settings.set_default("agent", agent).unwrap();
        settings.set_default("mongo", mongo).unwrap();

        MongoDBAgentSettings { conf: settings }
    }
}

impl MongoDBAgentSettings {
    /// Loads user configuration from files.
    ///
    /// Strings in the vector are paths to files to load.
    /// Files are loaded in order with the last overwriting the previous.
    ///
    /// All files are marked optional and it is not possible to know which
    /// files where loaded and which ones where not.
    pub fn load(&mut self, sources: Vec<&str>) -> Result<(), ConfigError> {
        for source in sources {
            self.conf.merge(File::with_name(source).required(false))?;
        }
        Ok(())
    }

    /// Deserialize the base agent configuration.
    pub fn agent(&self) -> AgentConfig {
        self.conf.get("agent").expect("Unable to parse agent configuration")
    }

    /// Deserialize the mongo specific configuration.
    pub fn mongo(&self) -> MongoDBSettings {
        self.conf.get("mongo").expect("Unable to parse MongoDB configuration")
    }
}


/// Container for MongoDB specific settings.
#[derive(Debug, Deserialize)]
pub struct MongoDBSettings {
    pub host: String,
    pub port: i64,
}

impl Default for MongoDBSettings {
    fn default() -> MongoDBSettings {
        MongoDBSettings {
            host: String::from("localhost"),
            port: 27017
        }
    }
}

impl From<MongoDBSettings> for Value {
    /// Convert a `MongoDBSettings` into a `Value` for the `config` crate.
    fn from(mongo: MongoDBSettings) -> Value {
        let mut conf: HashMap<String, Value> = HashMap::new();
        conf.insert(String::from("host"), Value::new(None, mongo.host));
        conf.insert(String::from("port"), Value::new(None, mongo.port));
        Value::new(None, conf)
    }
}


#[cfg(test)]
mod tests {
    use config::Config;
    use super::MongoDBAgentSettings;
    use super::MongoDBSettings;

    #[test]
    fn defaults() {
        let conf = MongoDBAgentSettings::default();
        let agent = conf.agent();
        let mongo = conf.mongo();
        assert_eq!("localhost:37017", agent.server.bind);
        assert_eq!("localhost", mongo.host);
        assert_eq!(27017, mongo.port);
    }

    #[test]
    fn into_value() {
        let default = MongoDBSettings::default();
        let mut conf = Config::new();
        conf.set_default("mongo", default).unwrap();
        assert_eq!("localhost", conf.get_str("mongo.host").unwrap());
        assert_eq!(27017, conf.get_int("mongo.port").unwrap());
    }
}
