use actix_web::{
    test::{call_and_read_body, init_service, TestRequest},
    web::Bytes,
};

use super::*;

#[actix_web::test]
#[ignore = "requires MongoDB instance running"]
async fn test_url_creation_and_redirection() {
    dotenv::dotenv().ok();
    let uri = std::env::var("MONGODB_URI").expect("MONGODB_URI not set");

    let client = Client::with_uri_str(uri).await.expect("Failed to connect");

    // Clear existing data
    client
        .database(DB_NAME)
        .collection::<UrlEntry>(COLL_NAME)
        .drop()
        .await
        .expect("Failed to clear collection");

    let app = init_service(
        App::new()
            .app_data(web::Data::new(client))
            .service(create_short_url)
            .service(redirect_url),
    )
    .await;

    let test_url = "https://example.com";
    let test_code = "abc123";

    // Test URL creation
    let create_req = TestRequest::post()
        .uri("/create")
        .set_json(UrlEntry {
            original_url: test_url.to_string(),
            short_code: test_code.to_string(),
        })
        .to_request();

    let response = call_and_read_body(&app, create_req).await;
    assert_eq!(response, Bytes::from_static(b"Short URL created"));

    // Test redirection
    let redirect_req = TestRequest::get()
        .uri(&format!("/{}", test_code))
        .to_request();

    let response = call_and_read_body(&app, redirect_req).await;
    assert!(response.starts_with(b"Moved Permanently"));
}
