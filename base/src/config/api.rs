use std::collections::HashMap;
use std::sync::RwLock;

use lazy_static::lazy_static;
use serde_derive::Deserialize;
use serde_derive::Serialize;

// Define some globals to hold the default overrides.
lazy_static! {
    static ref DEFAULT_BIND: RwLock<Option<String>> = RwLock::new(None);
}

/// Web server configuration options.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct APIConfig {
    /// Local addess to bind the API server to.
    #[serde(default = "APIConfig::default_bind")]
    pub bind: String,

    /// The number of request handling threads.
    #[serde(default)]
    pub threads_count: Option<usize>,

    /// API server timeouts.
    #[serde(default)]
    pub timeouts: Timeouts,

    /// Configure TLS (for HTTPS) certificates.
    #[serde(default)]
    pub tls: Option<TlsConfig>,

    /// Enable/disable entire API trees.
    #[serde(default)]
    pub trees: APITrees,
}

impl Default for APIConfig {
    fn default() -> Self {
        APIConfig {
            bind: Self::default_bind(),
            threads_count: None,
            timeouts: Timeouts::default(),
            tls: None,
            trees: APITrees::default(),
        }
    }
}

impl APIConfig {
    /// Default value for `bind` used by serde.
    fn default_bind() -> String {
        DEFAULT_BIND
            .read()
            .unwrap()
            .as_ref()
            .map(Clone::clone)
            .unwrap_or_else(|| String::from("127.0.0.1:8000"))
    }
}

impl APIConfig {
    /// Overrides the default bind attribute.
    ///
    /// This should be done at the very beginning of your agent and
    /// BEFORE ANY CONFIGURATION IS LOADED/INSTANTIATED.
    ///
    /// # Panics
    /// If the default is set more then once.
    pub fn set_default_bind(bind: String) {
        let mut default = DEFAULT_BIND.write().unwrap();
        if default.is_some() {
            panic!("cannot override the default api.bind option more than once");
        }
        *default = Some(bind);
    }
}

/// Enable/disable entire API trees.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct APITrees {
    /// Enable/disable the introspection APIs.
    #[serde(default = "APITrees::default_true")]
    pub introspect: bool,

    /// Enable/disable the unstable API.
    #[serde(default = "APITrees::default_true")]
    pub unstable: bool,
}

impl Default for APITrees {
    fn default() -> APITrees {
        APITrees {
            introspect: true,
            unstable: true,
        }
    }
}

impl APITrees {
    fn default_true() -> bool {
        true
    }
}

// We can's fulfill the wish of the implicit-hasher clippy because
// we do not use the genieric hasher parameter in any LOCAL type.
#[allow(clippy::implicit_hasher)]
impl From<APITrees> for HashMap<&'static str, bool> {
    fn from(trees: APITrees) -> HashMap<&'static str, bool> {
        let mut flags = HashMap::default();
        flags.insert("introspect", trees.introspect);
        flags.insert("unstable", trees.unstable);
        flags
    }
}

/// API server timeouts.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct Timeouts {
    /// Control the timeout, in seconds, for keep alive connections.
    #[serde(default = "Timeouts::default_keep_alive")]
    pub keep_alive: Option<usize>,

    /// Control the timeout, in seconds, for reads on existing connections.
    #[serde(default = "Timeouts::default_read")]
    pub read: Option<u64>,

    /// Control the timeout, in seconds, for writes on existing connections.
    #[serde(default = "Timeouts::default_write")]
    pub write: Option<u64>,
}

impl Default for Timeouts {
    fn default() -> Timeouts {
        Timeouts {
            keep_alive: Self::default_keep_alive(),
            read: Self::default_read(),
            write: Self::default_write(),
        }
    }
}

impl Timeouts {
    fn default_keep_alive() -> Option<usize> {
        Some(5)
    }

    fn default_read() -> Option<u64> {
        Some(5)
    }

    fn default_write() -> Option<u64> {
        Some(1)
    }
}

/// TLS (for HTTPS) certificates configuration.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Path to a PEM bundle of trusted CAs for client authentication.
    #[serde(default)]
    pub clients_ca_bundle: Option<String>,

    /// Path to a PEM file with the server's public certificate.
    pub server_cert: String,

    /// Path to a PEM file with the server's PRIVATE certificate.
    pub server_key: String,
}
