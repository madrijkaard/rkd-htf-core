use std::env;
use once_cell::sync::Lazy;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Credential {
    pub key: String,
    pub secret: String,
    pub open_ai_key: String,
}

pub static CREDENTIAL: Lazy<Arc<Credential>> = Lazy::new(|| {
    let key = env::var("BINANCE_API_KEY").expect("BINANCE_API_KEY nao definida");
    let secret = env::var("BINANCE_API_SECRET").expect("BINANCE_API_SECRET nao definida");
    let open_ai_key = env::var("OPEN_API_KEY").expect("OPEN_API_KEY nao definida");
    Arc::new(Credential { key, secret, open_ai_key })
});

pub fn get_credentials() -> Arc<Credential> {
    CREDENTIAL.clone()
}
