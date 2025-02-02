use serde::{Deserialize, Serialize};

// A trait that the Validate derive will impl
use validator::Validate;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Validate)]
pub struct UrlEntry {
    #[validate(url(message = "Must be a valid URL"))]
    pub original_url: String,
    pub short_code: String,
}
