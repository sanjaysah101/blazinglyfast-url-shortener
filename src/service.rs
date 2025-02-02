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

    async fn ensure_indexes(&self) {
        let options = IndexOptions::builder().unique(true).build();
        let model = IndexModel::builder()
            .keys(doc! { "short_code": 1 })
            .options(options)
            .build();

        if let Err(e) = self.collection.create_index(model).await {
            eprintln!("Failed to create index: {}", e);
        }
    }

    pub async fn create_url(&self, original_url: String) -> Result<UrlEntry, String> {
        let url_entry = UrlEntry {
            original_url,
            short_code: generate_short_code(),
        };

        // Validate the URL
        if let Err(errors) = url_entry.validate() {
            return Err(errors.to_string());
        }

        // Check if URL already exists
        if let Ok(Some(existing)) = self
            .collection
            .find_one(doc! { "original_url": &url_entry.original_url })
            .await
        {
            return Ok(existing);
        }

        // Insert new URL
        match self.collection.insert_one(&url_entry).await {
            Ok(_) => Ok(url_entry),
            Err(e) => Err(e.to_string()),
        }
    }

    pub async fn get_url_by_code(&self, short_code: &str) -> Result<Option<UrlEntry>, String> {
        self.collection
            .find_one(doc! { "short_code": short_code })
            .await
            .map_err(|e| e.to_string())
    }
}
