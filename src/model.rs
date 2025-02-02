use serde::{Deserialize, Serialize};

// A trait that the Validate derive will impl
use chrono::{DateTime, Utc};
use validator::Validate;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Validate)]
pub struct UrlEntry {
    #[validate(url(message = "Must be a valid URL"))]
    #[serde(default)]
    #[serde(skip_serializing)]
    pub original_url: String,
    #[serde(rename = "encrypted_url")]
    pub encrypted_url: String,
    pub short_code: String,
    pub clicks: i64,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}
