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
///   * The count of responses by method, path, HTTP status code.
pub struct MetricsMiddleware {
    duration: HistogramVec,
    errors: CounterVec,
    requests: CounterVec,
}

impl MetricsMiddleware {
    /// Constructs a new [`MetricsMiddleware`] to record metrics about handlers.
    ///
    /// The metrics to record observations in are passed to this method
    /// and must match the below requirements:
    ///
    ///   * The `duration` [`HistogramVec`] must have exactly two variable labels:
    ///     `["method", "path"]`.
    ///   * The `errors` [`CounterVec`] must have exactly two variable labels:
    ///     `["method", "path"]`.
    ///   * The `requests` [`HistogramVec`] must have exactly three variable labels:
    ///     `["method", "path", "status"]`.
    ///   * None of the variable labels above can be constant labels.
    ///
    /// # Panics
    /// This method validates the given metrics against the requirements
    /// and panics if any is not met.
    pub fn new(
        duration: HistogramVec, errors: CounterVec, requests: CounterVec
    ) -> MetricsMiddleware {
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

        // Check requests counter.
        for desc in requests.desc() {
            match desc.const_label_pairs.iter().find(|label| label.get_name() == "path") {
                None => (),
                Some(_) => panic!("The requests counter cannot have a const 'path' label")
            };
            match desc.const_label_pairs.iter().find(|label| label.get_name() == "method") {
                None => (),
                Some(_) => panic!("The requests counter cannot have a const 'method' label")
            };
            match desc.const_label_pairs.iter().find(|label| label.get_name() == "status") {
                None => (),
                Some(_) => panic!("The requests counter cannot have a const 'status' label")
            };
            assert!(
                desc.variable_labels == vec!["method", "path", "status"],
                "The variable labels for the requests counter must be ['method', 'path', 'status']"
            );
        }

        // Store all needed values.
        MetricsMiddleware {
            duration,
            errors,
            requests,
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


/// Extracts the response status code as a string.
///
/// # Panics
/// If the response does not have a status set.
fn response_status(response: &Response) -> String {
    response.status.expect("Response instance does not have a status set").to_u16().to_string()
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
        // Processing of the request failed before it even begun.
        // Still obseve a duration for this request or the counts to be accurate.
        let method = request_method(&request);
        let path = request_path(&request);
        self.middlewere.errors.with_label_values(&[&method, &path]).inc();
        let timer = self.middlewere.duration.with_label_values(&[&method, &path]).start_timer();
        timer.observe_duration();

        // Record the request by status code.
        let status = response_status(&err.response);
        self.middlewere.requests.with_label_values(&[&method, &path, &status]).inc();
        Err(err)
    }
}


/// Iron AfterMiddleware to record metrics.
pub struct MetricsAfter {
    middlewere: Arc<MetricsMiddleware>,
}

impl AfterMiddleware for MetricsAfter {
    fn after(&self, request: &mut Request, response: Response) -> IronResult<Response> {
        let status = response_status(&response);
        let method = request_method(&request);
        let path = request_path(&request);
        self.middlewere.requests.with_label_values(&[&method, &path, &status]).inc();

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
        let status = response_status(&err.response);
        let method = request_method(&request);
        let path = request_path(&request);
        self.middlewere.errors.with_label_values(&[&method, &path]).inc();
        self.middlewere.requests.with_label_values(&[&method, &path, &status]).inc();

        let metrics = match request.extensions.remove::<MetricsExtension>() {
            Some(metrics) => metrics,
            None => {
                // TODO: use logging.
                println!("Unable to find MetricsExtension on the request");
                return Err(err);
            }
        };
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

        use prometheus::Collector;
        use prometheus::CounterVec;
        use prometheus::HistogramOpts;
        use prometheus::HistogramVec;
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

        fn make_requests() -> CounterVec {
            CounterVec::new(
                Opts::new(
                    "agent_enpoint_requests",
                    "Number of requests processed"
                ),
                &vec!["method", "path", "status"]
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

        fn mock_handler(
            duration: HistogramVec, errors: CounterVec, requests: CounterVec
        ) -> Chain {
            let router = mock_router();
            let metrics = MetricsMiddleware::new(duration, errors, requests);
            let mut handler = Chain::new(router);
            handler.link(metrics.into_middleware());
            handler
        }

        #[test]
        fn link_to_chain() {
            let router = mock_router();
            let duration = make_duration();
            let errors = make_errors();
            let requests = make_requests();
            let metrics = MetricsMiddleware::new(duration, errors, requests);
            let mut handler = Chain::new(router);
            handler.link(metrics.into_middleware());
        }

        #[test]
        fn count_errors() {
            let duration = make_duration();
            let errors = make_errors();
            let requests = make_requests();
            let handler = mock_handler(duration, errors.clone(), requests);
            match request::post("http://localhost:3000/error", Headers::new(), "", &handler) {
                Ok(_) => panic!("request should have failed!"),
                Err(_) => ()
            };
            let count = errors.with_label_values(&["POST", "/error"]).get();
            assert_eq!(count, 1 as f64);
        }

        #[test]
        fn observe_duration() {
            let duration = make_duration();
            let errors = make_errors();
            let requests = make_requests();
            let handler = mock_handler(duration.clone(), errors, requests);
            request::get("http://localhost:3000/", Headers::new(), &handler).unwrap();
            let metric = duration.with_label_values(&["GET", "/"]).collect();
            assert_eq!(1 as u64, metric[0].get_metric()[0].get_histogram().get_sample_count());
            let sum = metric[0].get_metric()[0].get_histogram().get_sample_sum();
            assert!(sum < 1 as f64);
            assert!(sum > 0 as f64);
        }

        #[test]
        fn count_by_status_code() {
            let duration = make_duration();
            let errors = make_errors();
            let requests = make_requests();
            let handler = mock_handler(duration, errors, requests.clone());
            request::get("http://localhost:3000/", Headers::new(), &handler).unwrap();
            match request::post("http://localhost:3000/error", Headers::new(), "", &handler) {
                Ok(_) => panic!("request should have failed!"),
                Err(_) => ()
            };
            let count_200 = requests.with_label_values(&["GET", "/", "200"]).get();
            let count_400 = requests.with_label_values(&["POST", "/error", "400"]).get();
            assert_eq!(1 as f64, count_200);
            assert_eq!(1 as f64, count_400);
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
            let requests = CounterVec::new(Opts::new("t3", "t3"), &vec![]).unwrap();
            MetricsMiddleware::new(duration, counter, requests);
        }

        #[test]
        #[should_panic(expected = "The variable labels for the duration histogram must be ['method', 'path']")]
        fn duration_with_rand_labels() {
            let duration = HistogramVec::new(
                HistogramOpts::new("t1", "t1"), &vec!["abc", "path"]
            ).unwrap();
            let counter = CounterVec::new(Opts::new("t2", "t2"), &vec![]).unwrap();
            let requests = CounterVec::new(Opts::new("t3", "t3"), &vec![]).unwrap();
            MetricsMiddleware::new(duration, counter, requests);
        }

        #[test]
        #[should_panic(expected = "The variable labels for the duration histogram must be ['method', 'path']")]
        fn duration_with_labels_out_of_order() {
            let duration = HistogramVec::new(
                HistogramOpts::new("t1", "t1"), &vec!["path", "method"]
            ).unwrap();
            let counter = CounterVec::new(Opts::new("t2", "t2"), &vec![]).unwrap();
            let requests = CounterVec::new(Opts::new("t3", "t3"), &vec![]).unwrap();
            MetricsMiddleware::new(duration, counter, requests);
        }

        #[test]
        #[should_panic(expected = "The duration histogram cannot have a const 'method' label")]
        fn duration_with_static_method_label() {
            let duration = HistogramVec::new(
                HistogramOpts::new("t1", "t1").const_label("method", "test"), &vec![]
            ).unwrap();
            let counter = CounterVec::new(Opts::new("t2", "t2"), &vec![]).unwrap();
            let requests = CounterVec::new(Opts::new("t3", "t3"), &vec![]).unwrap();
            MetricsMiddleware::new(duration, counter, requests);
        }

        #[test]
        #[should_panic(expected = "The duration histogram cannot have a const 'path' label")]
        fn duration_with_static_path_label() {
            let duration = HistogramVec::new(
                HistogramOpts::new("t1", "t1").const_label("path", "test"), &vec![]
            ).unwrap();
            let counter = CounterVec::new(Opts::new("t2", "t2"), &vec![]).unwrap();
            let requests = CounterVec::new(Opts::new("t3", "t3"), &vec![]).unwrap();
            MetricsMiddleware::new(duration, counter, requests);
        }

        #[test]
        #[should_panic(expected = "The variable labels for the errors counter must be ['method', 'path']")]
        fn errors_with_no_labels() {
            let duration = HistogramVec::new(
                HistogramOpts::new("t1", "t1"), &vec!["method", "path"]
            ).unwrap();
            let counter = CounterVec::new(Opts::new("t2", "t2"), &vec![]).unwrap();
            let requests = CounterVec::new(Opts::new("t3", "t3"), &vec![]).unwrap();
            MetricsMiddleware::new(duration, counter, requests);
        }

        #[test]
        #[should_panic(expected = "The variable labels for the errors counter must be ['method', 'path']")]
        fn errors_with_rand_labels() {
            let duration = HistogramVec::new(
                HistogramOpts::new("t1", "t1"), &vec!["method", "path"]
            ).unwrap();
            let counter = CounterVec::new(Opts::new("t2", "t2"), &vec!["a", "path"]).unwrap();
            let requests = CounterVec::new(Opts::new("t3", "t3"), &vec![]).unwrap();
            MetricsMiddleware::new(duration, counter, requests);
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
            let requests = CounterVec::new(Opts::new("t3", "t3"), &vec![]).unwrap();
            MetricsMiddleware::new(duration, counter, requests);
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
            let requests = CounterVec::new(Opts::new("t3", "t3"), &vec![]).unwrap();
            MetricsMiddleware::new(duration, counter, requests);
        }

        #[test]
        #[should_panic(expected = "The variable labels for the errors counter must be ['method', 'path']")]
        fn errors_with_labels_out_of_order() {
            let duration = HistogramVec::new(
                HistogramOpts::new("t1", "t1"), &vec!["method", "path"]
            ).unwrap();
            let counter = CounterVec::new(Opts::new("t2", "t2"), &vec!["path", "method"]).unwrap();
            let requests = CounterVec::new(Opts::new("t3", "t3"), &vec![]).unwrap();
            MetricsMiddleware::new(duration, counter, requests);
        }

        #[test]
        #[should_panic(expected = "The variable labels for the requests counter must be ['method', 'path', 'status']")]
        fn requests_with_no_labels() {
            let duration = HistogramVec::new(
                HistogramOpts::new("t1", "t1"), &vec!["method", "path"]
            ).unwrap();
            let counter = CounterVec::new(Opts::new("t2", "t2"), &vec!["method", "path"]).unwrap();
            let requests = CounterVec::new(Opts::new("t3", "t3"), &vec![]).unwrap();
            MetricsMiddleware::new(duration, counter, requests);
        }

        #[test]
        #[should_panic(expected = "The variable labels for the requests counter must be ['method', 'path', 'status']")]
        fn requests_with_rand_labels() {
            let duration = HistogramVec::new(
                HistogramOpts::new("t1", "t1"), &vec!["method", "path"]
            ).unwrap();
            let counter = CounterVec::new(Opts::new("t2", "t2"), &vec!["method", "path"]).unwrap();
            let requests = CounterVec::new(
                Opts::new("t3", "t3"), &vec!["a", "path", "status"]
            ).unwrap();
            MetricsMiddleware::new(duration, counter, requests);
        }

        #[test]
        #[should_panic(expected = "The requests counter cannot have a const 'method' label")]
        fn requests_with_static_method_label() {
            let duration = HistogramVec::new(
                HistogramOpts::new("t1", "t1"), &vec!["method", "path"]
            ).unwrap();
            let counter = CounterVec::new(Opts::new("t2", "t2"), &vec!["method", "path"]).unwrap();
            let requests = CounterVec::new(
                Opts::new("t3", "t3").const_label("method", "test"), &vec![]
            ).unwrap();
            MetricsMiddleware::new(duration, counter, requests);
        }

        #[test]
        #[should_panic(expected = "The requests counter cannot have a const 'path' label")]
        fn requests_with_static_path_label() {
            let duration = HistogramVec::new(
                HistogramOpts::new("t1", "t1"), &vec!["method", "path"]
            ).unwrap();
            let counter = CounterVec::new(Opts::new("t2", "t2"), &vec!["method", "path"]).unwrap();
            let requests = CounterVec::new(
                Opts::new("t3", "t3").const_label("path", "test"), &vec![]
            ).unwrap();
            MetricsMiddleware::new(duration, counter, requests);
        }

        #[test]
        #[should_panic(expected = "The requests counter cannot have a const 'status' label")]
        fn requests_with_static_code_label() {
            let duration = HistogramVec::new(
                HistogramOpts::new("t1", "t1"), &vec!["method", "path"]
            ).unwrap();
            let counter = CounterVec::new(Opts::new("t2", "t2"), &vec!["method", "path"]).unwrap();
            let requests = CounterVec::new(
                Opts::new("t3", "t3").const_label("status", "test"), &vec![]
            ).unwrap();
            MetricsMiddleware::new(duration, counter, requests);
        }

        #[test]
        #[should_panic(expected = "The variable labels for the requests counter must be ['method', 'path', 'status']")]
        fn requests_with_labels_out_of_order() {
            let duration = HistogramVec::new(
                HistogramOpts::new("t1", "t1"), &vec!["method", "path"]
            ).unwrap();
            let counter = CounterVec::new(Opts::new("t2", "t2"), &vec!["method", "path"]).unwrap();
            let requests = CounterVec::new(
                Opts::new("t3", "t3"), &vec!["path", "status", "method"]
            ).unwrap();
            MetricsMiddleware::new(duration, counter, requests);
        }

        #[test]
        fn creates_the_middlewere() {
            let duration = HistogramVec::new(
                HistogramOpts::new("t1", "t1"), &vec!["method", "path"]
            ).unwrap();
            let counter = CounterVec::new(Opts::new("t2", "t2"), &vec!["method", "path"]).unwrap();
            let requests = CounterVec::new(
                Opts::new("t3", "t3"), &vec!["method", "path", "status"]
            ).unwrap();
            MetricsMiddleware::new(duration, counter, requests);
        }
    }
}
