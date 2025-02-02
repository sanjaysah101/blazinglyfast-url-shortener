mod model;
#[cfg(test)]
mod test;

use actix_web::{get, post, web, App, HttpResponse, HttpServer};
use model::UrlEntry;
use mongodb::{bson::doc, options::IndexOptions, Client, Collection, IndexModel};

const DB_NAME: &str = "url_shortener";
const COLL_NAME: &str = "urls";

/// Create a short URL
#[post("/create")]
async fn create_short_url(
    client: web::Data<Client>,
    url_entry: web::Json<UrlEntry>,
) -> HttpResponse {
    let collection = client.database(DB_NAME).collection(COLL_NAME);
    
    match collection.insert_one(url_entry.into_inner()).await {
        Ok(_) => HttpResponse::Ok().body("Short URL created"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

/// Redirect to original URL using short code
#[get("/{short_code}")]
async fn redirect_url(
    client: web::Data<Client>,
    short_code: web::Path<String>,
) -> HttpResponse {
    let collection: Collection<UrlEntry> = client.database(DB_NAME).collection(COLL_NAME);
    
    match collection.find_one(doc! { "short_code": &short_code.into_inner() }).await {
        Ok(Some(entry)) => HttpResponse::MovedPermanently()
            .append_header(("Location", entry.original_url))
            .finish(),
        Ok(None) => HttpResponse::NotFound().body("Short URL not found"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

/// Create index on short_code field
async fn create_short_code_index(client: &Client) {
    let options = IndexOptions::builder().unique(true).build();
    let model = IndexModel::builder()
        .keys(doc! { "short_code": 1 })
        .options(options)
        .build();
    
    client
        .database(DB_NAME)
        .collection::<UrlEntry>(COLL_NAME)
        .create_index(model)
        .await
        .expect("Failed to create index");
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let uri = std::env::var("MONGODB_URI").expect("MONGODB_URI not set");

    let client = Client::with_uri_str(uri).await.expect("Failed to connect");
    create_short_code_index(&client).await;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone()))
            .service(create_short_url)
            .service(redirect_url)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
