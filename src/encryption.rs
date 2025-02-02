use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand_core::{OsRng, RngCore};

#[derive(Clone)]
pub struct Encryptor {
    cipher: Aes256Gcm,
}

impl Encryptor {
    pub fn new(key: &[u8; 32]) -> Self {
        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        Self { cipher }
    }

    pub fn encrypt(&self, text: &str) -> Result<String, String> {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, text.as_bytes())
            .map_err(|e| format!("Encryption error: {}", e))?;

        // Combine nonce and ciphertext and encode as base64
        let mut combined = nonce_bytes.to_vec();
        combined.extend(ciphertext);
        Ok(BASE64.encode(combined))
    }

    pub fn decrypt(&self, encrypted: &str) -> Result<String, String> {
        let data = BASE64
            .decode(encrypted)
            .map_err(|e| format!("Base64 decode error: {}", e))?;

        if data.len() < 12 {
            return Err("Invalid encrypted data".to_string());
        }

        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption error: {}", e))?;

        String::from_utf8(plaintext).map_err(|e| format!("UTF-8 decode error: {}", e))
    }
}
