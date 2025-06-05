use crate::dto::CryptoMetadata;
use once_cell::sync::Lazy;
use std::fs;
use std::sync::Arc;

pub static CRYPTO_METADATA: Lazy<Arc<Vec<CryptoMetadata>>> = Lazy::new(|| {
    let data = fs::read_to_string("assets/crypto_metadata.json")
        .expect("Falha ao ler o arquivo assets/crypto_metadata.json");

    let parsed: Vec<CryptoMetadata> = serde_json::from_str(&data)
        .expect("Falha ao fazer parse de crypto_metadata.json");

    Arc::new(parsed)
});

pub fn get_crypto_metadata() -> Arc<Vec<CryptoMetadata>> {
    CRYPTO_METADATA.clone()
}
