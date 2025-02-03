use crate::encryption::Encryptor;
use crate::error::UrlError;
use crate::model::UrlEntry;
use crate::utils::generate_short_code;
use base64::Engine;
use chrono::{Duration, Utc};
use futures::StreamExt;
use mongodb::{bson::doc, options::IndexOptions, Client, Collection, IndexModel};
use validator::{Validate, ValidationError, ValidationErrors};

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
        short_code: Option<String>,
    ) -> Result<(UrlEntry, bool), UrlError> {
        // Validate custom short code if provided
        if let Some(code) = &short_code {
            if code.len() < 3 || code.len() > 20 {
                let mut errors = ValidationErrors::new();
                errors.add(
                    "short_code",
                    ValidationError::new("length").with_message(std::borrow::Cow::Borrowed(
                        "Short code must be between 3 and 20 characters",
                    )),
                );
                return Err(UrlError::ValidationError(errors));
            }
            // Check if short code already exists
            if let Some(_) = self
                .collection
                .find_one(doc! { "short_code": code })
                .await?
            {
                let mut errors = ValidationErrors::new();
                errors.add(
                    "short_code",
                    ValidationError::new("already_exists").with_message(
                        std::borrow::Cow::Borrowed("This short code is already taken"),
                    ),
                );
                return Err(UrlError::ValidationError(errors));
            }
        }

        // Encrypt the URL first
        let encrypted_url = self
            .encryptor
            .encrypt(&original_url)
            .map_err(|e| UrlError::InternalError(e))?;

        // Check if URL already exists (using encrypted value)
        let existing_entries = self.collection.find(doc! {}).await?;

        let mut cursor = existing_entries;
        while let Some(url) = cursor.next().await {
            let mut entry = url.map_err(UrlError::from)?;
            let decrypted = self
                .encryptor
                .decrypt(&entry.encrypted_url)
                .map_err(|e| UrlError::InternalError(e))?;

            if decrypted == original_url {
                // Set the decrypted URL before returning
                entry.original_url = decrypted;
                // Return a specific error or message indicating the URL already exists
                let mut errors = ValidationErrors::new();
                errors.add(
                    "original_url",
                    ValidationError::new("already_exists")
                        .with_message(std::borrow::Cow::Borrowed("This URL is already shortened")),
                );
                return Err(UrlError::ValidationError(errors));
            }
        }

        // Create new entry if URL doesn't exist
        let url_entry = UrlEntry {
            encrypted_url,
            original_url: original_url.clone(), // Store decrypted URL for response
            short_code: short_code.unwrap_or_else(generate_short_code), // Use custom code or generate
            clicks: 0,
            created_at: Utc::now(),
            expires_at: expires_in_days.map(|days| Utc::now() + Duration::days(days)),
        };
        url_entry.validate()?;

        // Store only encrypted version in database
        let db_entry = UrlEntry {
            encrypted_url: url_entry.encrypted_url.clone(),
            original_url: String::new(), // This won't be used
            short_code: url_entry.short_code.clone(),
            clicks: url_entry.clicks,
            created_at: url_entry.created_at,
            expires_at: url_entry.expires_at,
        };

        self.collection.insert_one(&db_entry).await?;
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
            // Decrypt the URL before sending
            entry.original_url = self
                .encryptor
                .decrypt(&entry.encrypted_url)
                .map_err(|e| UrlError::InternalError(e))?;
            urls.push(entry);
        }

        Ok(urls)
    }
}
