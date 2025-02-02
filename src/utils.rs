use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};

const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
const SHORT_CODE_LENGTH: usize = 7;

pub fn generate_short_code() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let mut rng = rand::rng();
    let random_string: String = (0..SHORT_CODE_LENGTH)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    format!("{}{}", random_string, timestamp % 1000)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_generate_unique_codes() {
        let mut codes = HashSet::new();
        for _ in 0..1000 {
            let code = generate_short_code();
            assert!(!codes.contains(&code), "Generated duplicate code");
            codes.insert(code);
        }
    }
}
