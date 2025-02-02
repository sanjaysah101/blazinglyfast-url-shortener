use crate::encryption::Encryptor;
use crate::error::UrlError;
use crate::model::UrlEntry;
use crate::utils::generate_short_code;
use base64::Engine;
use chrono::{Duration, Utc};
use futures::StreamExt;
use mongodb::{bson::doc, options::IndexOptions, Client, Collection, IndexModel};
use validator::Validate;

#[derive(Clone)]
pub struct UrlService {
    collection: Collection<UrlEntry>,
    encryptor: Encryptor,
}

impl UrlService {
    pub fn new(client: &Client, db_name: &str, coll_name: &str) -> Self {
        // Decode base64 key and convert to 32 bytes
        let encryption_key = base64::engine::general_purpose::STANDARD
            .decode(std::env::var("ENCRYPTION_KEY").expect("ENCRYPTION_KEY must be set"))
            .expect("Invalid base64 in ENCRYPTION_KEY")
            .try_into()
            .expect("ENCRYPTION_KEY must be 32 bytes when decoded");

        let service = Self {
            collection: client.database(db_name).collection(coll_name),
            encryptor: Encryptor::new(&encryption_key),
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

    pub async fn create_url(
        &self,
        original_url: String,
        expires_in_days: Option<i64>,
    ) -> Result<(UrlEntry, bool), UrlError> {
        // Validate first
        let url_entry = UrlEntry {
            original_url: original_url.clone(),
            encrypted_url: self
                .encryptor
                .encrypt(&original_url)
                .map_err(|e| UrlError::InternalError(e))?,
            short_code: generate_short_code(),
            clicks: 0,
            created_at: Utc::now(),
            expires_at: expires_in_days.map(|days| Utc::now() + Duration::days(days)),
        };
        url_entry.validate()?;

        // Check if URL already exists (using encrypted value)
        if let Some(existing) = self
            .collection
            .find_one(doc! { "encrypted_url": &url_entry.encrypted_url })
            .await?
        {
            // Decrypt the URL before returning
            let mut decrypted = existing.clone();
            decrypted.original_url = self
                .encryptor
                .decrypt(&existing.encrypted_url)
                .map_err(|e| UrlError::InternalError(e))?;
            return Ok((decrypted, false));
        }

        self.collection.insert_one(&url_entry).await?;
        Ok((url_entry, true))
    }

    pub async fn get_url_by_code(&self, short_code: &str) -> Result<Option<UrlEntry>, UrlError> {
        if let Some(mut entry) = self
            .collection
            .find_one_and_update(
                doc! { "short_code": short_code },
                doc! { "$inc": { "clicks": 1 } },
            )
            .await
            .map_err(UrlError::from)?
        {
            // Decrypt the URL before returning
            entry.original_url = self
                .encryptor
                .decrypt(&entry.encrypted_url)
                .map_err(|e| UrlError::InternalError(e))?;
            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }

    pub async fn get_urls(&self) -> Result<Vec<UrlEntry>, UrlError> {
        let mut cursor = self
            .collection
            .find(doc! {})
            .await
            .map_err(UrlError::from)?;

        let mut urls = Vec::new();
        while let Some(url) = cursor.next().await {
            let mut entry = url.map_err(UrlError::from)?;
            entry.original_url = self
                .encryptor
                .decrypt(&entry.encrypted_url)
                .map_err(|e| UrlError::InternalError(e))?;
            urls.push(entry);
        }

        Ok(urls)
    }
}
