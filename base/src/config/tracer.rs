use std::fmt;
use std::str::FromStr;

use opentracingrust::Tracer;
use opentracingrust::tracers::NoopTracer;
use opentracingrust::utils::ReporterThread;

use opentracingrust_zipkin::KafkaCollector;
use opentracingrust_zipkin::ZipkinEndpoint;
use opentracingrust_zipkin::ZipkinTracer;

use super::super::error::AgentError;
use super::super::error::AgentResult;

use super::TracerConfig;


/// Enumerate all supported tracer backends.
#[derive(Debug, Deserialize, Serialize)]
pub enum TracerBackend {
    NoopTracer,
    ZipkinTracer,
}

impl fmt::Display for TracerBackend {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tracer = match *self {
            TracerBackend::NoopTracer => "NoopTracer",
            TracerBackend::ZipkinTracer => "ZipkinTracer",
        };
        write!(f, "{}", tracer)
    }
}

impl FromStr for TracerBackend {
    type Err = AgentError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "NoopTracer" => Ok(TracerBackend::NoopTracer),
            "ZipkinTracer" => Ok(TracerBackend::ZipkinTracer),
            _ => Err(AgentError::ConfigError(format!("Unsupported tracer: {}", s)))
        }
    }
}


/// Configures the distributed tracer based on the given options.
///
/// The method returns a `Tracer` object and a `ReporterThread`.
/// The list of supported tracers is defined by the `TracerBackend` enum.
/// Different tracers support different options and features.
///
///
/// # Noop
/// The `NoopTracer` is the default tracer and discards all the spans.
/// Useful when you don't have a distributed tracer infrastructure.
///
/// ## Configuration:
/// ```yaml
/// agent:
///   tracer:
///     backend: NoopTracer
/// ```
///
///
/// # Zipkin
/// The `ZipkinTracer` sends spans to [Zipkin](https://zipkin.io/)
/// over the [Kafka](https://kafka.apache.org/) collector.
///
/// ## Configuration
/// Some extra options are required:
///
///   * A `service_name` to attach to spans.
///   * A list of `kafka` servers to send spans to.
///
/// ```yaml
/// agent:
///   tracer:
///     backend: ZipkinTracer
///     zipkin:
///       service_name: my-distributed-system
///       kafka:
///         - host1:9092
///         - host2:9092
///       topic: traces  # Default: zipkin
/// ```
pub fn configure_tracer(config: TracerConfig) -> AgentResult<(Tracer, ReporterThread)> {
    let backend = config.backend.parse::<TracerBackend>()?;
    match backend {
        TracerBackend::NoopTracer => {
            let (tracer, receiver) = NoopTracer::new();
            let reporter = ReporterThread::new(receiver, |span| {
                NoopTracer::report(span);
            });
            Ok((tracer, reporter))
        },
        TracerBackend::ZipkinTracer => {
            let zipkin = config.zipkin.ok_or_else(
                || AgentError::ConfigError(String::from("Missing Zipkin tracer configuration"))
            )?;
            let service_name = zipkin.service_name;
            let kafka = zipkin.kafka;
            let topic = zipkin.topic.unwrap_or_else(|| "zipkin".into());
            let mut collector = KafkaCollector::new(
                ZipkinEndpoint::new(None, None, Some(service_name), None),
                topic, kafka
            );
            let (tracer, receiver) = ZipkinTracer::new();
            let reporter = ReporterThread::new(receiver, move |span| {
                if let Err(err) = collector.collect(span) {
                    println!("Failed to report span: {:?}", err);
                }
            });
            Ok((tracer, reporter))
        },
    }
}


#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::AgentError;
    use super::TracerConfig;

    use super::TracerBackend;
    use super::configure_tracer;

    #[test]
    fn unsupported_tracer() {
        let config = TracerConfig {
            backend: "unsupported".into(),
            zipkin: None,
        };
        match configure_tracer(config) {
            Err(AgentError::ConfigError(msg)) => assert_eq!(
                "Unsupported tracer: unsupported", msg
            ),
            Err(err) => panic!("Expected ConfigError but got {:?}", err),
            Ok(_) => panic!("Expected Err but got Ok!")
        }
    }

    #[test]
    fn noop_tracer() {
        let config = TracerConfig {
            backend: TracerBackend::NoopTracer.to_string(),
            zipkin: None,
        };
        let (_tracer, mut reporter) = configure_tracer(config)
            .expect("Failed to configure NoopTracer");
        reporter.stop_delay(Duration::from_millis(10));
    }

    mod zipkin {
        use std::time::Duration;

        use super::super::super::TracerConfigZipkin;
        use super::super::AgentError;
        use super::super::TracerConfig;

        use super::super::TracerBackend;
        use super::super::configure_tracer;

        #[test]
        fn needs_options() {
            let config = TracerConfig {
                backend: TracerBackend::ZipkinTracer.to_string(),
                zipkin: None,
            };
            match configure_tracer(config) {
            Err(AgentError::ConfigError(msg)) => assert_eq!(
                "Missing Zipkin tracer configuration", msg
            ),
            Err(err) => panic!("Expected ConfigError but got {:?}", err),
                Ok((_, mut reporter)) => {
                    reporter.stop_delay(Duration::from_millis(10));
                    panic!("Expected configuration failure");
                }
            }
        }

        #[test]
        // For now, KafkaCollector requires a real kafka running which means
        // it can't really be tested.
        // Once the collector is re-writtien/improved (probably with official opentracing)
        // this test can be enabled.
        #[ignore]
        fn make_tracer() {
            let config = TracerConfig {
                backend: TracerBackend::ZipkinTracer.to_string(),
                zipkin: Some(TracerConfigZipkin {
                    service_name: "replicante-agent".into(),
                    kafka: vec![String::from("localhost:9092")],
                    topic: None
                }),
            };
            let (_tracer, mut reporter) = configure_tracer(config)
                .expect("Failed to configure ZipkinTracer");
            reporter.stop_delay(Duration::from_millis(10));
        }
    }
}
