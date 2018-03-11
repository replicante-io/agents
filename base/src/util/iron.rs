use std::sync::Arc;

use iron::prelude::*;
use iron::typemap::Key;
use iron::AfterMiddleware;
use iron::BeforeMiddleware;

use prometheus::Collector;
use prometheus::CounterVec;
use prometheus::HistogramTimer;
use prometheus::HistogramVec;


/// An Iron middlewere to collect metrics about endpoints.
///
/// This middlewere collects the following information:
///
///   * The duration of endpoints as an histogram.
///   * The number of requests that return an error.
///   * TODO: The count of responses by method, path, HTTP status code.
pub struct MetricsMiddleware {
    duration: HistogramVec,
    errors: CounterVec,
}

impl MetricsMiddleware {
    /// TODO
    ///
    /// # Panics
    /// TODO: if ["method", "path"] are not variable labels.
    /// TODO: if "method" is a constant label.
    /// TODO: if "path" is a constant label.
    pub fn new(duration: HistogramVec, errors: CounterVec) -> MetricsMiddleware {
        // Check duration Histogram.
        for desc in duration.desc() {
            match desc.const_label_pairs.iter().find(|label| label.get_name() == "path") {
                None => (),
                Some(_) => panic!("The duration histogram cannot have a const 'path' label")
            };
            match desc.const_label_pairs.iter().find(|label| label.get_name() == "method") {
                None => (),
                Some(_) => panic!("The duration histogram cannot have a const 'method' label")
            };
            assert!(
                desc.variable_labels == vec!["method", "path"],
                "The variable labels for the duration histogram must be ['method', 'path']"
            );
        }

        // Check errors counter.
        for desc in errors.desc() {
            match desc.const_label_pairs.iter().find(|label| label.get_name() == "path") {
                None => (),
                Some(_) => panic!("The errors counter cannot have a const 'path' label")
            };
            match desc.const_label_pairs.iter().find(|label| label.get_name() == "method") {
                None => (),
                Some(_) => panic!("The errors counter cannot have a const 'method' label")
            };
            assert!(
                desc.variable_labels == vec!["method", "path"],
                "The variable labels for the errors counter must be ['method', 'path']"
            );
        }

        // Store all needed values.
        MetricsMiddleware {
            duration,
            errors,
        }
    }

    /// Converts the middlewere into Iron's BeforeMiddleware and AfterMiddleware.
    pub fn into_middleware(self) -> (MetricsBefore, MetricsAfter) {
        let me = Arc::new(self);
        let before = MetricsBefore { middlewere: Arc::clone(&me) };
        let after = MetricsAfter { middlewere: me };
        (before, after)
    }
}


/// Extracts the request method as a string.
fn request_method(request: &Request) -> String {
    request.method.to_string()
}


/// Extracts the request path as a string.
fn request_path(request: &Request) -> String {
    format!("/{}", request.url.path().join("/"))
}


/// An Iron extension to store per-request metric data.
struct MetricsExtension {
    duration: HistogramTimer,
}

impl Key for MetricsExtension {
    type Value = MetricsExtension;
}


/// Iron BeforeMiddleware to prepare request tracking.
pub struct MetricsBefore {
    middlewere: Arc<MetricsMiddleware>,
}

impl BeforeMiddleware for MetricsBefore {
    fn before(&self, request: &mut Request) -> IronResult<()> {
        let method = request_method(&request);
        let path = request_path(&request);
        let timer = self.middlewere.duration.with_label_values(&[&method, &path]).start_timer();
        let extension = MetricsExtension {
            duration: timer,
        };
        request.extensions.insert::<MetricsExtension>(extension);
        Ok(())
    }

    fn catch(&self, request: &mut Request, err: IronError) -> IronResult<()> {
        let method = request_method(&request);
        let path = request_path(&request);
        // Processing of the request failed before it even begun.
        // Still obseve a duration for this request or the counts to be accurate.
        self.middlewere.errors.with_label_values(&[&method, &path]).inc();
        let timer = self.middlewere.duration.with_label_values(&[&method, &path]).start_timer();
        timer.observe_duration();
        Err(err)
    }
}


/// Iron AfterMiddleware to record metrics.
pub struct MetricsAfter {
    middlewere: Arc<MetricsMiddleware>,
}

impl AfterMiddleware for MetricsAfter {
    fn after(&self, request: &mut Request, response: Response) -> IronResult<Response> {
        let metrics = match request.extensions.remove::<MetricsExtension>() {
            Some(metrics) => metrics,
            None => {
                // TODO: use logging.
                println!("Unable to find MetricsExtension on the request");
                return Ok(response);
            }
        };
        metrics.duration.observe_duration();
        Ok(response)
    }

    fn catch(&self, request: &mut Request, err: IronError) -> IronResult<Response> {
        let method = request_method(&request);
        let path = request_path(&request);
        let metrics = match request.extensions.remove::<MetricsExtension>() {
            Some(metrics) => metrics,
            None => {
                // TODO: use logging.
                println!("Unable to find MetricsExtension on the request");
                return Err(err);
            }
        };
        self.middlewere.errors.with_label_values(&[&method, &path]).inc();
        metrics.duration.observe_duration();
        Err(err)
    }
}


#[cfg(test)]
mod tests {
    mod observations {
        use std::env::VarError;

        use iron::prelude::*;
        use iron::status;
        use iron::Headers;
        use iron_test::request;
        use router::Router;

        use prometheus::CounterVec;
        use prometheus::HistogramVec;
        use prometheus::HistogramOpts;
        use prometheus::Opts;

        use super::super::MetricsMiddleware;

        fn make_duration() -> HistogramVec {
            HistogramVec::new(
                HistogramOpts::new(
                    "agent_endpoint_duration",
                    "Observe the duration (in seconds) of agent endpoints"
                ),
                &vec!["method", "path"]
            ).unwrap()
        }

        fn make_errors() -> CounterVec {
            CounterVec::new(
                Opts::new(
                    "agent_enpoint_errors",
                    "Number of errors encountered while handling requests"
                ),
                &vec!["method", "path"]
            ).unwrap()
        }

        fn mock_router() -> Router {
            let mut router = Router::new();
            router.get("/", |_: &mut Request| -> IronResult<Response> {
                Ok(Response::with((status::Ok, "Test")))
            }, "index");
            router.post("/error", |_: &mut Request| -> IronResult<Response> {
                let error = IronError {
                    error: Box::new(VarError::NotPresent),
                    response: Response::with((status::BadRequest, "Test"))
                };
                Err(error)
            }, "error");
            router
        }

        fn mock_handler(duration: HistogramVec, errors: CounterVec) -> Chain {
            let router = mock_router();
            let metrics = MetricsMiddleware::new(duration, errors);
            let mut handler = Chain::new(router);
            handler.link(metrics.into_middleware());
            handler
        }

        #[test]
        fn link_to_chain() {
            let router = mock_router();
            let duration = make_duration();
            let errors = make_errors();
            let metrics = MetricsMiddleware::new(duration, errors);
            let mut handler = Chain::new(router);
            handler.link(metrics.into_middleware());
        }

        #[test]
        fn count_errors() {
            let duration = make_duration();
            let errors = make_errors();
            let handler = mock_handler(duration, errors.clone());
            match request::post("http://localhost:3000/error", Headers::new(), "", &handler) {
                Ok(_) => panic!("request should have failed!"),
                Err(_) => ()
            };
            let count = errors.with_label_values(&["POST", "/error"]).get();
            assert_eq!(count, 1 as f64);
        }
    }

    mod validation {
        use prometheus::CounterVec;
        use prometheus::HistogramVec;
        use prometheus::HistogramOpts;
        use prometheus::Opts;

        use super::super::MetricsMiddleware;

        #[test]
        #[should_panic(expected = "The variable labels for the duration histogram must be ['method', 'path']")]
        fn duration_with_no_labels() {
            let duration = HistogramVec::new(HistogramOpts::new("t1", "t1"), &vec![]).unwrap();
            let counter = CounterVec::new(Opts::new("t2", "t2"), &vec![]).unwrap();
            MetricsMiddleware::new(duration, counter);
        }

        #[test]
        #[should_panic(expected = "The variable labels for the duration histogram must be ['method', 'path']")]
        fn duration_with_rand_labels() {
            let duration = HistogramVec::new(
                HistogramOpts::new("t1", "t1"), &vec!["abc", "path"]
            ).unwrap();
            let counter = CounterVec::new(Opts::new("t2", "t2"), &vec![]).unwrap();
            MetricsMiddleware::new(duration, counter);
        }

        #[test]
        #[should_panic(expected = "The variable labels for the duration histogram must be ['method', 'path']")]
        fn duration_with_labels_out_of_order() {
            let duration = HistogramVec::new(
                HistogramOpts::new("t1", "t1"), &vec!["path", "method"]
            ).unwrap();
            let counter = CounterVec::new(Opts::new("t2", "t2"), &vec![]).unwrap();
            MetricsMiddleware::new(duration, counter);
        }

        #[test]
        #[should_panic(expected = "The duration histogram cannot have a const 'method' label")]
        fn duration_with_static_method_label() {
            let duration = HistogramVec::new(
                HistogramOpts::new("t1", "t1").const_label("method", "test"), &vec![]
            ).unwrap();
            let counter = CounterVec::new(Opts::new("t2", "t2"), &vec![]).unwrap();
            MetricsMiddleware::new(duration, counter);
        }

        #[test]
        #[should_panic(expected = "The duration histogram cannot have a const 'path' label")]
        fn duration_with_static_path_label() {
            let duration = HistogramVec::new(
                HistogramOpts::new("t1", "t1").const_label("path", "test"), &vec![]
            ).unwrap();
            let counter = CounterVec::new(Opts::new("t2", "t2"), &vec![]).unwrap();
            MetricsMiddleware::new(duration, counter);
        }

        #[test]
        #[should_panic(expected = "The variable labels for the errors counter must be ['method', 'path']")]
        fn errors_with_no_labels() {
            let duration = HistogramVec::new(
                HistogramOpts::new("t1", "t1"), &vec!["method", "path"]
            ).unwrap();
            let counter = CounterVec::new(Opts::new("t2", "t2"), &vec![]).unwrap();
            MetricsMiddleware::new(duration, counter);
        }

        #[test]
        #[should_panic(expected = "The variable labels for the errors counter must be ['method', 'path']")]
        fn errors_with_rand_labels() {
            let duration = HistogramVec::new(
                HistogramOpts::new("t1", "t1"), &vec!["method", "path"]
            ).unwrap();
            let counter = CounterVec::new(Opts::new("t2", "t2"), &vec!["a", "path"]).unwrap();
            MetricsMiddleware::new(duration, counter);
        }

        #[test]
        #[should_panic(expected = "The errors counter cannot have a const 'method' label")]
        fn errors_with_static_method_label() {
            let duration = HistogramVec::new(
                HistogramOpts::new("t1", "t1"), &vec!["method", "path"]
            ).unwrap();
            let counter = CounterVec::new(
                Opts::new("t2", "t2").const_label("method", "test"), &vec![]
            ).unwrap();
            MetricsMiddleware::new(duration, counter);
        }

        #[test]
        #[should_panic(expected = "The errors counter cannot have a const 'path' label")]
        fn errors_with_static_path_label() {
            let duration = HistogramVec::new(
                HistogramOpts::new("t1", "t1"), &vec!["method", "path"]
            ).unwrap();
            let counter = CounterVec::new(
                Opts::new("t2", "t2").const_label("path", "path"), &vec![]
            ).unwrap();
            MetricsMiddleware::new(duration, counter);
        }

        #[test]
        #[should_panic(expected = "The variable labels for the errors counter must be ['method', 'path']")]
        fn errors_with_labels_out_of_order() {
            let duration = HistogramVec::new(
                HistogramOpts::new("t1", "t1"), &vec!["method", "path"]
            ).unwrap();
            let counter = CounterVec::new(Opts::new("t2", "t2"), &vec!["path", "method"]).unwrap();
            MetricsMiddleware::new(duration, counter);
        }

        #[test]
        fn creates_the_middlewere() {
            let duration = HistogramVec::new(
                HistogramOpts::new("t1", "t1"), &vec!["method", "path"]
            ).unwrap();
            let counter = CounterVec::new(Opts::new("t2", "t2"), &vec!["method", "path"]).unwrap();
            MetricsMiddleware::new(duration, counter);
        }
    }
}
