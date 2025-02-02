mod encryption;
mod error;
mod model;
mod routes;
mod service;
mod utils;

use actix_cors::Cors;
use actix_files as fs;
use actix_web::{web, App, HttpServer};
use mongodb::Client;
use service::UrlService;

const DB_NAME: &str = "url_shortener";
const COLL_NAME: &str = "urls";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let uri = std::env::var("MONGODB_URI").expect("MONGODB_URI not set");
    let client = Client::with_uri_str(uri)
        .await
        .expect("Failed to connect to MongoDB");

    let url_service = UrlService::new(&client, DB_NAME, COLL_NAME);

    HttpServer::new(move || {
        let cors = Cors::permissive(); // For development only

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(url_service.clone()))
            .service(routes::url::create)
            .service(routes::url::list)
            .service(routes::url::redirect)
            .service(fs::Files::new("/", "src/static").index_file("index.html"))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
