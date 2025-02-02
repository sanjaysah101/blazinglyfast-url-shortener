use crate::error::UrlError;
use crate::model::UrlEntry;
use crate::utils::generate_short_code;
use mongodb::{bson::doc, options::IndexOptions, Client, Collection, IndexModel};
use validator::Validate;

#[derive(Clone)]
pub struct UrlService {
    collection: Collection<UrlEntry>,
}

impl UrlService {
    pub fn new(client: &Client, db_name: &str, coll_name: &str) -> Self {
        let service = Self {
            collection: client.database(db_name).collection(coll_name),
        };

        let service_clone = service.clone();
        tokio::spawn(async move { service_clone.ensure_indexes().await });

        service
    }

    async fn ensure_indexes(&self) -> Result<(), UrlError> {
        let options = IndexOptions::builder().unique(true).build();
        let model = IndexModel::builder()
            .keys(doc! { "short_code": 1 })
            .options(options)
            .build();

        self.collection
            .create_index(model)
            .await
            .map_err(|e| UrlError::InternalError(format!("Failed to create index: {}", e)))?;
        Ok(())
    }

    pub async fn create_url(&self, original_url: String) -> Result<UrlEntry, UrlError> {
        let url_entry = UrlEntry {
            original_url,
            short_code: generate_short_code(),
        };

        // Validate the URL
        url_entry.validate()?;

        // Check if URL already exists
        if let Some(existing) = self
            .collection
            .find_one(doc! { "original_url": &url_entry.original_url })
            .await?
        {
            return Ok(existing);
        }

        // Insert new URL
        self.collection.insert_one(&url_entry).await?;
        Ok(url_entry)
    }

    pub async fn get_url_by_code(&self, short_code: &str) -> Result<Option<UrlEntry>, UrlError> {
        self.collection
            .find_one(doc! { "short_code": short_code })
            .await
            .map_err(UrlError::from)
    }
}
