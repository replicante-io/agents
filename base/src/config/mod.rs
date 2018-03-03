use std::collections::HashMap;
use std::convert::From;

use config_crate::Value;

mod tracer;
pub use self::tracer::configure_tracer;
pub use self::tracer::TracerBackend;


/// Stores the base agent configuration options.
///
/// Configuration options used by the base agent utilities and structs.
/// Attributes are public to make it easier to use configuration values
/// but are not meant to be changed after the configuration is finialised.
///
/// New configuration values are created with `AgentConfig::default` and
/// changing the attributes as desired.
///
/// # Examples
///
/// ```
/// extern crate replicante_agent;
/// use replicante_agent::config::AgentConfig;
///
/// fn main() {
///     let mut agent = AgentConfig::default();
///     agent.server.bind = String::from("1.2.3.4:5678");
///
///     // Ready to use the configuration, make read-only.
///     let agent = agent;
/// }
/// ```
#[derive(Debug, Deserialize)]
pub struct AgentConfig {
    pub server: AgentServerConfig,
    pub tracer: TracerConfig,
}

impl Default for AgentConfig {
    /// Returns an `AgentConfig` filled with default values.
    ///
    /// Agent implementations should override defaults with their preferred
    /// values before loading user settings,
    ///
    /// To load user settings, the `AgentConfig` can be converted in a `Value`
    /// as profided by the rust `config` crate.
    ///
    /// # Examples
    ///
    /// ```
    /// extern crate config as config_crate;
    /// extern crate replicante_agent;
    ///
    /// use config_crate::Config;
    /// use replicante_agent::config::AgentConfig;
    ///
    /// fn main() {
    ///     let mut default = AgentConfig::default();
    ///     default.server.bind = String::from("127.0.0.1:80");
    ///
    ///     let mut conf = Config::new();
    ///     conf.set_default("agent.prefix", default);
    ///     conf.set("agent.prefix.server.bind", "127.0.0.1:1234");
    ///     assert_eq!(
    ///         "127.0.0.1:1234",
    ///         conf.get_str("agent.prefix.server.bind").unwrap()
    ///     );
    /// }
    /// ```
    fn default() -> AgentConfig {
        AgentConfig {
            server: AgentServerConfig {
                bind: String::from("127.0.0.1:8000"),
            },
            tracer: TracerConfig {
                backend: TracerBackend::NoopTracer.to_string(),
                zipkin: None,
            }
        }
    }
}

impl From<AgentConfig> for Value {
    /// Convert an `AgentConfig` into a `Value` for the `config` crate.
    ///
    /// # Examples
    ///
    /// ```
    /// extern crate config as config_crate;
    /// extern crate replicante_agent;
    ///
    /// use config_crate::Config;
    /// use replicante_agent::config::AgentConfig;
    ///
    /// fn main() {
    ///     let mut default = AgentConfig::default();
    ///     default.server.bind = String::from("127.0.0.1:80");
    ///
    ///     let mut conf = Config::new();
    ///     conf.set("agent", default);
    ///     assert_eq!("127.0.0.1:80", conf.get_str("agent.server.bind").unwrap());
    /// }
    /// ```
    fn from(agent: AgentConfig) -> Value {
        let mut server: HashMap<String, Value> = HashMap::new();
        server.insert(String::from("bind"), Value::new(None, agent.server.bind));

        let mut tracer: HashMap<String, Value> = HashMap::new();
        tracer.insert(
            String::from("backend"),
            Value::new(None, TracerBackend::NoopTracer.to_string())
        );

        let mut conf: HashMap<String, Value> = HashMap::new();
        conf.insert(String::from("server"), Value::new(None, server));
        conf.insert(String::from("tracer"), Value::new(None, tracer));
        Value::new(None, conf)
    }
}


/// Store the web server configuration options.
#[derive(Debug, Deserialize)]
pub struct AgentServerConfig {
    pub bind: String,
}


/// Configuration options for distributed tracing.
#[derive(Debug, Deserialize)]
pub struct TracerConfig {
    pub backend: String,
    pub zipkin: Option<TracerConfigZipkin>,
}


/// Configuration options specific to the Zipkin tracer.
#[derive(Debug, Deserialize)]
pub struct TracerConfigZipkin {
    pub service_name: String,
    pub kafka: Vec<String>,
    pub topic: Option<String>,
}
