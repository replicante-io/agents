use serde_derive::Serialize;

use actix_web::HttpResponse;
use actix_web::Responder;
use humthreads::registered_threads;
use humthreads::ThreadStatus;

/// Expose a snaphot view of traked threads states.
pub async fn handler() -> impl Responder {
    let mut threads = registered_threads();
    threads.sort_unstable_by_key(|t| t.name.clone());
    let threads = ThreadsResponse::new(threads);
    HttpResponse::Ok().json(threads)
}

/// Wrap the `humthreads::registered_threads` list to expose as structured data.
#[derive(Debug, Serialize)]
struct ThreadsResponse {
    threads: Vec<ThreadStatus>,
    warning: &'static [&'static str],
}

impl ThreadsResponse {
    fn new(threads: Vec<ThreadStatus>) -> ThreadsResponse {
        let warning = &[
            "This list is NOT provided from an OS-layer instrumentation.",
            "As such, some threads may not be reported in this list.",
        ];
        ThreadsResponse { threads, warning }
    }
}
