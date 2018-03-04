use std::collections::HashMap;

use iron::Response;

use opentracingrust;
use opentracingrust::InjectFormat;
use opentracingrust::MapCarrier;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;


/// Implement the MapCarrier trait for Iron's Response.
///
/// # Examples
///
/// ```ignore
/// use replicante_agent::util::ResponseCarrier;
///
/// let mut response = Response::new();
/// ResponseCarrier::inject(span.context(), &mut response, &tracer);
/// ```
pub struct ResponseCarrier<'a> {
    response: &'a mut Response,
    // This is horrible, I am sorry.
    // The MapCarrier items function returns pointers which we don't have
    // because of how Iron's Header iteration works.
    // To work around this, we store a view of the iterator in a compatible
    // format so we can return references to valid memory.
    iter_stage: HashMap<String, String>,
}

impl<'a> ResponseCarrier<'a> {
    /// Mutably borrow a response so it can be serialised.
    pub fn new(response: &'a mut Response) -> ResponseCarrier<'a> {
        let items: HashMap<String, String> = {
            response.headers.iter()
                .map(|header|(String::from(header.name()), header.value_string()))
                .collect()
        };
        ResponseCarrier {
            iter_stage: items,
            response
        }
    }

    /// Inject a `SpanContext` into the given Iron response.
    pub fn inject(
        context: &SpanContext, response: &mut Response, tracer: &Tracer
    ) -> opentracingrust::Result<()> {
        let mut carrier = ResponseCarrier::new(response);
        let format = InjectFormat::HttpHeaders(Box::new(&mut carrier));
        tracer.inject(context, format)?;
        Ok(())
    }
}

impl<'a> MapCarrier for ResponseCarrier<'a> {
    fn items(&self) -> Vec<(&String, &String)> {
        self.iter_stage.iter().collect()
    }

    fn get(&self, key: &str) -> Option<String> {
        match self.response.headers.get_raw(key) {
            Some(value) => Some(String::from_utf8(value[0].clone()).unwrap()),
            None => None
        }
    }

    fn set(&mut self, key: &str, value: &str) {
        let value = String::from(value).into_bytes();
        self.response.headers.set_raw(String::from(key), vec![value]);
    }
}


#[cfg(test)]
mod tests {
    use iron::Response;

    use opentracingrust::MapCarrier;
    use opentracingrust::tracers::NoopTracer;

    use super::ResponseCarrier;

    #[test]
    fn get_header() {
        let mut response = Response::new();
        response.headers.set_raw("X-Test-1", vec![String::from("Test 1").into_bytes()]);
        response.headers.set_raw("X-Test-2", vec![String::from("Test 2").into_bytes()]);
        let carrier = ResponseCarrier::new(&mut response);
        let boxed: Box<MapCarrier> = Box::new(carrier);
        assert_eq!("Test 1", boxed.get("X-Test-1").unwrap());
        assert_eq!("Test 2", boxed.get("X-Test-2").unwrap());
        assert!(boxed.get("X-Test-3").is_none());
    }

    #[test]
    fn iter_headers() {
        let mut response = Response::new();
        response.headers.set_raw("X-Test-1", vec![String::from("Test 1").into_bytes()]);
        response.headers.set_raw("X-Test-2", vec![String::from("Test 2").into_bytes()]);
        let carrier = ResponseCarrier::new(&mut response);
        let boxed: Box<MapCarrier> = Box::new(carrier);
        let items: Vec<(&String, &String)> = boxed.items();
        assert_eq!(2, items.len());
    }

    #[test]
    fn set_header() {
        let mut response = Response::new();
        {
            let carrier = ResponseCarrier::new(&mut response);
            let mut boxed: Box<MapCarrier> = Box::new(carrier);
            boxed.set("X-Some-Header", "Some header value");
        }
        let value = response.headers.get_raw("X-Some-Header").unwrap();
        let value = String::from_utf8(value[0].clone()).unwrap();
        assert_eq!(value, "Some header value");
    }

    #[test]
    fn inject() {
        let (tracer, _receiver) = NoopTracer::new();
        let span = tracer.span("test");
        let mut response = Response::new();
        ResponseCarrier::inject(span.context(), &mut response, &tracer).unwrap();
        // TODO: when mock tracer exists use it to check the headers.
    }
}
