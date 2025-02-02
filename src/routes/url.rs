use crate::error::UrlError;
use crate::service::UrlService;
use actix_web::{get, post, web, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateUrlRequest {
    url: String,
    expires_in_days: Option<i64>,
}

/// Create a short URL
#[post("/api/urls")]
pub async fn create(
    service: web::Data<UrlService>,
    request: web::Json<CreateUrlRequest>,
) -> Result<HttpResponse, UrlError> {
    let (entry, is_new) = service
        .create_url(request.url.clone(), request.expires_in_days)
        .await?;

    Ok(if is_new {
        HttpResponse::Created().json(entry)
    } else {
        HttpResponse::Ok().json(entry)
    })
}

/// Redirect to original URL using short code
#[get("/r/{short_code}")]
pub async fn redirect(
    service: web::Data<UrlService>,
    short_code: web::Path<String>,
) -> Result<HttpResponse, UrlError> {
    match service.get_url_by_code(&short_code).await? {
        Some(entry) => {
            let url = if !entry.original_url.starts_with("http://")
                && !entry.original_url.starts_with("https://")
            {
                format!("http://{}", entry.original_url)
            } else {
                entry.original_url
            };

            Ok(HttpResponse::TemporaryRedirect()
                .append_header(("Location", url))
                .append_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
                .finish())
        }
        None => Err(UrlError::NotFound),
    }
}
