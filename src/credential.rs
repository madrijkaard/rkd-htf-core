use std::env;
use once_cell::sync::Lazy;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Credential {
    pub key: String,
    pub secret: String,
}

pub static CREDENTIAL: Lazy<Arc<Credential>> = Lazy::new(|| {
    let key = env::var("BINANCE_API_KEY").expect("BINANCE_API_KEY nao definida");
    let secret = env::var("BINANCE_API_SECRET").expect("BINANCE_API_SECRET nao definida");
    Arc::new(Credential { key, secret })
});

pub fn get_credentials() -> Arc<Credential> {
    CREDENTIAL.clone()
}
