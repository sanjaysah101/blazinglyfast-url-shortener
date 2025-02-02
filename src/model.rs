use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct UrlEntry {
    pub original_url: String,
    pub short_code: String,
}
