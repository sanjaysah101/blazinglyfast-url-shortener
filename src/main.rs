mod model;
mod service;
mod utils;

use actix_web::{get, post, web, App, HttpResponse, HttpServer};
use mongodb::Client;
use serde::Deserialize;
use service::UrlService;

const DB_NAME: &str = "url_shortener";
const COLL_NAME: &str = "urls";

#[derive(Deserialize)]
struct CreateUrlRequest {
    url: String,
}

/// Create a short URL
#[post("/create")]
async fn create_short_url(
    service: web::Data<UrlService>,
    request: web::Json<CreateUrlRequest>,
) -> HttpResponse {
    match service.create_url(request.url.clone()).await {
        Ok(entry) => HttpResponse::Ok().json(entry),
        Err(err) => HttpResponse::BadRequest().body(err),
    }
}

/// Redirect to original URL using short code
#[get("/{short_code}")]
async fn redirect_url(
    service: web::Data<UrlService>,
    short_code: web::Path<String>,
) -> HttpResponse {
    match service.get_url_by_code(&short_code).await {
        Ok(Some(entry)) => HttpResponse::MovedPermanently()
            .append_header(("Location", entry.original_url))
            .finish(),
        Ok(None) => HttpResponse::NotFound().body("Short URL not found"),
        Err(err) => HttpResponse::InternalServerError().body(err),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let uri = std::env::var("MONGODB_URI").expect("MONGODB_URI not set");
    let client = Client::with_uri_str(uri)
        .await
        .expect("Failed to connect to MongoDB");

    let url_service = UrlService::new(&client, DB_NAME, COLL_NAME);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(url_service.clone()))
            .service(create_short_url)
            .service(redirect_url)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
