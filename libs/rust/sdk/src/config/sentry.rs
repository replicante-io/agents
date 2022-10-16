use serde::Deserialize;
use serde::Serialize;

/// Sentry integration configuration.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct SentryConfig {
    /// Capture API server errors in Sentry.
    #[serde(default = "SentryConfig::default_capture_api_errors")]
    pub capture_api_errors: bool,

    /// The DSN to use to configure sentry.
    pub dsn: String,
}

impl SentryConfig {
    fn default_capture_api_errors() -> bool {
        true
    }
}
