use actix_web::HttpResponse;
use actix_web::Responder;

pub async fn index() -> impl Responder {
    HttpResponse::Ok().body("Replicante Agent API endpoints")
}

#[cfg(test)]
mod tests {
    use actix_web::http::StatusCode;
    use actix_web::test::call_service;
    use actix_web::test::init_service;
    use actix_web::test::read_body;
    use actix_web::test::TestRequest;
    use actix_web::web;
    use actix_web::App;

    #[actix_rt::test]
    async fn index_points_to_api() {
        let app = init_service(App::new().route("/", web::get().to(super::index)));
        let mut app = app.await;
        let request = TestRequest::default().to_request();
        let response = call_service(&mut app, request).await;
        assert_eq!(response.status(), StatusCode::OK);
        let body = read_body(response).await;
        assert_eq!(
            String::from_utf8(body.to_vec()).unwrap(),
            "Replicante Agent API endpoints"
        );
    }
}
