use thiserror::Error;
use validator::ValidationErrors;

#[derive(Error, Debug)]
pub enum UrlError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] mongodb::error::Error),

    #[error("Validation error: {0}")]
    ValidationError(#[from] ValidationErrors),

    #[error("URL not found")]
    NotFound,

    #[error("Internal server error: {0}")]
    InternalError(String),
}

impl actix_web::ResponseError for UrlError {
    fn error_response(&self) -> actix_web::HttpResponse {
        match self {
            UrlError::ValidationError(_) => {
                actix_web::HttpResponse::BadRequest().json(self.to_string())
            }
            UrlError::NotFound => actix_web::HttpResponse::NotFound().json(self.to_string()),
            UrlError::DatabaseError(_) | UrlError::InternalError(_) => {
                actix_web::HttpResponse::InternalServerError().json(self.to_string())
            }
        }
    }
}
