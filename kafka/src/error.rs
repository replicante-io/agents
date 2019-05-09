use std::fmt;

use replicante_agent::Error;
use replicante_agent::ErrorKind as BaseKind;

/// Zookeeper specifc error kinds.
#[derive(Debug)]
pub enum ErrorKind {
    /// `FreeForm` wrapper for invalid broker ID in JMX name.
    BrokerIdFormat(String),

    /// `FreeForm` wrapper for no broker ID in JMX
    BrokerNoId,

    /// `FreeForm` wrapper for too many broker IDs in JMX.
    BrokerTooManyIds,

    /// Alias for `ConfigLoad`.
    ConfigLoad,

    /// Alias for `ConfigOption`.
    ConfigOption(&'static str),

    /// Alias for `Initialisation`.
    Initialisation(String),

    /// Alias for `Io`.
    Io(String),

    /// JMX specifc `Connection`.
    JmxConnection(String),

    /// JSON specifc `ResponseDecode`.
    JsonDecode(&'static str),

    /// `InvalidStoreState` wrapper for partitions without brokers.
    PartitionNoBrokers(String),

    /// Alias for `StoreOpFailed`.
    StoreOpFailed(&'static str),

    /// `FreeForm` wrapper for topics without offset metadata.
    TopicNoOffsets(String),

    /// Zookeeper specifc `Connection`.
    ZookeeperConnection(String),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Zookeeper Error")
    }
}

impl From<ErrorKind> for BaseKind {
    fn from(error: ErrorKind) -> BaseKind {
        match error {
            ErrorKind::BrokerIdFormat(name) => BaseKind::FreeForm(format!(
                "unable to extract broker id from JMX metric '{}'",
                name
            )),
            ErrorKind::BrokerNoId => {
                BaseKind::FreeForm("no broker id reported in through JMX".into())
            }
            ErrorKind::BrokerTooManyIds => {
                BaseKind::FreeForm("too many broker ids reported through JMX metric".into())
            }
            ErrorKind::ConfigLoad => BaseKind::ConfigLoad,
            ErrorKind::ConfigOption(option) => BaseKind::ConfigOption(option),
            ErrorKind::Initialisation(message) => BaseKind::Initialisation(message),
            ErrorKind::Io(path) => BaseKind::Io(path),
            ErrorKind::JmxConnection(address) => BaseKind::Connection("jmx server", address),
            ErrorKind::JsonDecode(op) => BaseKind::ResponseDecode("json", op),
            ErrorKind::PartitionNoBrokers(partition) => {
                BaseKind::InvalidStoreState(format!("partition {} has no brokers", partition))
            }
            ErrorKind::StoreOpFailed(op) => BaseKind::StoreOpFailed(op),
            ErrorKind::TopicNoOffsets(topic) => {
                BaseKind::FreeForm(format!("unable to find offsets for topic {}", topic))
            }
            ErrorKind::ZookeeperConnection(address) => BaseKind::Connection("zookeeper", address),
        }
    }
}

impl From<ErrorKind> for Error {
    fn from(error: ErrorKind) -> Error {
        Error::from(BaseKind::from(error))
    }
}
