use crate::credential::get_credentials;
use crate::config::BinanceSettings;
use serde::{Deserialize, Serialize};
use reqwest::{Client, header::{HeaderMap, HeaderValue, CONTENT_TYPE}};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Deserialize, Serialize)]
pub struct LeverageResponse {
    pub leverage: u32,
    pub symbol: String,
}

fn get_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

fn sign_query(query: &str, secret: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(query.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

pub async fn set_leverage(
    settings: &BinanceSettings,
    symbol: &str,
) -> Result<LeverageResponse, Box<dyn std::error::Error>> {
    set_leverage_with_value(settings, symbol, settings.leverage).await
}

pub async fn set_leverage_with_value(
    settings: &BinanceSettings,
    symbol: &str,
    leverage: u32,
) -> Result<LeverageResponse, Box<dyn std::error::Error>> {
    let credentials = get_credentials();
    let timestamp = get_timestamp();

    let query = format!(
        "symbol={}&leverage={}&recvWindow=10000&timestamp={}",
        symbol, leverage, timestamp
    );
    let signature = sign_query(&query, &credentials.secret);
    let full_url = format!("{}/leverage?{}&signature={}", settings.future_url, query, signature);

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert("X-MBX-APIKEY", HeaderValue::from_str(&credentials.key)?);

    let client = Client::new();
    let res = client.post(&full_url).headers(headers).send().await?;

    if res.status().is_success() {
        let response = res.json::<LeverageResponse>().await?;
        println!(
            "Leverage successfully applied: {}x to {}",
            response.leverage, response.symbol
        );
        Ok(response)
    } else {
        let error_text = res.text().await?;
        eprintln!("Error applying leverage: {}", error_text);
        Err(format!("Binance error: {}", error_text).into())
    }
}
