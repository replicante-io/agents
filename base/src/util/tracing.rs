use std::collections::HashMap;

use iron::Headers;

use opentracingrust::Result as OTResult;

use opentracingrust::ExtractFormat;
use opentracingrust::InjectFormat;
use opentracingrust::MapCarrier;

use opentracingrust::Span;
use opentracingrust::SpanContext;
use opentracingrust::Tracer;

/// Implement the MapCarrier trait for Iron's Headers.
///
/// # Examples
///
/// Inject a span context:
///
/// ```ignore
/// use replicante_agent::util::HeadersCarrier;
///
/// let mut response = Response::new();
/// HeadersCarrier::inject(span.context(), &mut response.headers, &tracer);
/// ```
///
/// Optionally extract a context:
///
/// ```ignore
/// use replicante_agent::util::HeadersCarrier;
///
/// let mut response = Response::new();
/// HeadersCarrier::extract(span.context(), &response.headers, &tracer);
/// ```
///
/// Create a new span, making it a child if the headers have a context:
///
/// ```ignore
/// use replicante_agent::util::HeadersCarrier;
///
/// let mut response = Response::new();
/// HeadersCarrier::child_of("span", &response.headers, &tracer);
/// ```
pub struct HeadersCarrier<'a> {
    headers: &'a mut Headers,
    // This is horrible, I am sorry.
    // The MapCarrier items function returns pointers which we don't have
    // because of how Iron's Header iteration works.
    // To work around this, we store a view of the iterator in a compatible
    // format so we can return references to valid memory.
    iter_stage: HashMap<String, String>,
}

impl<'a> HeadersCarrier<'a> {
    /// Fill the the iter_stage internal variable.
    ///
    /// Again ... sorry.
    fn prepare_iter(&mut self) {
        let items: HashMap<String, String> = {
            self.headers
                .iter()
                .map(|header| (String::from(header.name()), header.value_string()))
                .collect()
        };
        self.iter_stage = items;
    }
}

impl<'a> HeadersCarrier<'a> {
    /// Mutably borrow a response so it can be serialised.
    pub fn new(headers: &'a mut Headers) -> HeadersCarrier<'a> {
        let mut carrier = HeadersCarrier {
            iter_stage: HashMap::new(),
            headers,
        };
        carrier.prepare_iter();
        carrier
    }

    /// Inject a `SpanContext` into the given Iron headers.
    pub fn inject(context: &SpanContext, headers: &mut Headers, tracer: &Tracer) -> OTResult<()> {
        let mut carrier = HeadersCarrier::new(headers);
        let format = InjectFormat::HttpHeaders(Box::new(&mut carrier));
        tracer.inject(context, format)?;
        Ok(())
    }

    /// Create a new span as a child of the context in the headers.
    ///
    /// If the headers do not include any context the span will be a root span.
    pub fn child_of(name: &str, headers: &mut Headers, tracer: &Tracer) -> OTResult<Span> {
        let mut span = tracer.span(name);
        if let Some(context) = HeadersCarrier::context(headers, tracer)? {
            span.child_of(context);
        }
        Ok(span)
    }

    /// Checks the headers for a span context and extract it if possible.
    pub fn context(headers: &mut Headers, tracer: &Tracer) -> OTResult<Option<SpanContext>> {
        let carrier = HeadersCarrier::new(headers);
        let format = ExtractFormat::HttpHeaders(Box::new(&carrier));
        tracer.extract(format)
    }

    /// Create a new span as a followe of the context in the headers.
    ///
    /// If the headers do not include any context the span will be a root span.
    pub fn follows_from(name: &str, headers: &mut Headers, tracer: &Tracer) -> OTResult<Span> {
        let mut span = tracer.span(name);
        if let Some(context) = HeadersCarrier::context(headers, tracer)? {
            span.follows(context);
        }
        Ok(span)
    }
}

impl<'a> MapCarrier for HeadersCarrier<'a> {
    fn items(&self) -> Vec<(&String, &String)> {
        self.iter_stage.iter().collect()
    }

    fn get(&self, key: &str) -> Option<String> {
        match self.headers.get_raw(key) {
            Some(value) => Some(String::from_utf8(value[0].clone()).unwrap()),
            None => None,
        }
    }

    fn set(&mut self, key: &str, value: &str) {
        let value = String::from(value).into_bytes();
        self.headers.set_raw(String::from(key), vec![value]);
        self.prepare_iter();
    }
}

#[cfg(test)]
mod tests {
    use iron::Response;

    use opentracingrust::tracers::NoopTracer;
    use opentracingrust::MapCarrier;

    use super::HeadersCarrier;

    #[test]
    fn get_header() {
        let mut response = Response::new();
        response
            .headers
            .set_raw("X-Test-1", vec![String::from("Test 1").into_bytes()]);
        response
            .headers
            .set_raw("X-Test-2", vec![String::from("Test 2").into_bytes()]);
        let carrier = HeadersCarrier::new(&mut response.headers);
        let boxed: Box<MapCarrier> = Box::new(carrier);
        assert_eq!("Test 1", boxed.get("X-Test-1").unwrap());
        assert_eq!("Test 2", boxed.get("X-Test-2").unwrap());
        assert!(boxed.get("X-Test-3").is_none());
    }

    #[test]
    fn iter_headers() {
        let mut response = Response::new();
        response
            .headers
            .set_raw("X-Test-1", vec![String::from("Test 1").into_bytes()]);
        response
            .headers
            .set_raw("X-Test-2", vec![String::from("Test 2").into_bytes()]);
        let carrier = HeadersCarrier::new(&mut response.headers);
        let boxed: Box<MapCarrier> = Box::new(carrier);
        let items: Vec<(&String, &String)> = boxed.items();
        assert_eq!(2, items.len());
    }

    #[test]
    fn set_header() {
        let mut response = Response::new();
        {
            let carrier = HeadersCarrier::new(&mut response.headers);
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
        HeadersCarrier::inject(span.context(), &mut response.headers, &tracer).unwrap();
        // TODO: when mock tracer exists use it to check the headers.
    }

    // TODO: when mock tracer exists, test context extraction.
    // TODO: when mock tracer exists, test child_of extraction.
    // TODO: when mock tracer exists, test follows_from extraction.
}
