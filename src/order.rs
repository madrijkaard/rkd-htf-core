use crate::binance::{get_current_price, get_lot_size_info};
use crate::credential::get_credentials;
use crate::dto::OrderResponse;
use crate::config::{BinanceSettings, Settings};
use hmac::{Hmac, Mac};
use reqwest::{
    header::{HeaderMap, HeaderValue, CONTENT_TYPE},
    Client,
};
use sha2::Sha256;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use url::form_urlencoded;

type HmacSha256 = Hmac<Sha256>;

fn round_quantity(value: f64, step: f64) -> f64 {
    (value / step).floor() * step
}

fn get_timestamp() -> u64 {
    let start = SystemTime::now();
    let since = start.duration_since(UNIX_EPOCH).unwrap();
    since.as_millis() as u64
}

fn sign_query(query: &str, secret: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(query.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

async fn get_server_time_offset(settings: &BinanceSettings) -> Result<i64, String> {
    let client = Client::new();
    let time_url = format!("{}/time", settings.future_url);

    let res = client
        .get(&time_url)
        .send()
        .await
        .map_err(|e| format!("Error querying /time: {:?}", e))?;

    let json: serde_json::Value = res
        .json()
        .await
        .map_err(|e| format!("Error parsing /time: {:?}", e))?;

    let server_time = json["serverTime"].as_i64().ok_or("serverTime field missing")?;
    let local_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "Local clock error")?
        .as_millis() as i64;

    Ok(server_time - local_time)
}

pub async fn execute_future_order(
    settings: &BinanceSettings,
    side: &str,
    symbol: &str,
) -> Result<OrderResponse, String> {
    let credentials = get_credentials();
    let api_key = &credentials.key;
    let secret_key = &credentials.secret;

    let base_url = format!("{}/order", settings.future_url);

    let offset = get_server_time_offset(settings).await.unwrap_or(0);
    let timestamp = (get_timestamp() as i64 + offset) as u64;
    let timestamp_str = timestamp.to_string();

    let preco_btc = get_current_price(settings, symbol).await?;
    let lot_size_info = get_lot_size_info(settings, symbol).await?;

    let config = Settings::load();
    let money = config.money;

    let quantity_raw = money / preco_btc;
    let quantity = round_quantity(quantity_raw, lot_size_info.step_size);

    let precision = (1.0 / lot_size_info.step_size).log10().round() as usize;
    let quantity_str = format!("{:.*}", precision, quantity)
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string();

    println!(
        "Sending order with side: '{}', quantity: '{}' (USDT: {}, Cryptocurrency Price: {}, StepSize: {})",
        side, quantity_str, money, preco_btc, lot_size_info.step_size
    );

    let notional = quantity * preco_btc;
    if notional < 20.0 {
        return Err(format!(
            "Total order value ({:.2} USDT) is less than the minimum required (20 USDT)",
            notional
        ));
    }

    let mut params = HashMap::new();
    params.insert("symbol", symbol);
    params.insert("side", side);
    params.insert("type", "MARKET");
    params.insert("quantity", &quantity_str);
    params.insert("recvWindow", "10000");
    params.insert("timestamp", &timestamp_str);

    let query_string = form_urlencoded::Serializer::new(String::new())
        .extend_pairs(&params)
        .finish();

    let signature = sign_query(&query_string, secret_key);
    let signed_query = format!("{}&signature={}", query_string, signature);

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert("X-MBX-APIKEY", HeaderValue::from_str(api_key).unwrap());

    let client = Client::new();

    let res = client
        .post(format!("{}?{}", base_url, signed_query))
        .headers(headers)
        .send()
        .await
        .map_err(|e| format!("Request error: {:?}", e))?;

    if res.status().is_success() {
        res.json::<OrderResponse>()
            .await
            .map_err(|e| format!("Error interpreting JSON: {:?}", e))
    } else {
        let err = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        Err(format!("Binance Error: {}", err))
    }
}

pub async fn close_all_positions(
    settings: &BinanceSettings,
    symbol: &str,
) -> Result<Vec<OrderResponse>, String> {
    let credentials = get_credentials();
    let api_key = &credentials.key;
    let secret_key = &credentials.secret;

    let offset = get_server_time_offset(settings).await.unwrap_or(0);
    let timestamp = (get_timestamp() as i64 + offset) as u64;
    let query = format!("timestamp={}", timestamp);
    let signature = sign_query(&query, secret_key);
    let full_query = format!("{}&signature={}", query, signature);

    let position_risk_url = format!("{}/positionRisk?{}", settings.future_url_v2, full_query);

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert("X-MBX-APIKEY", HeaderValue::from_str(api_key).unwrap());

    let client = Client::new();

    let res = client
        .get(&position_risk_url)
        .headers(headers.clone())
        .send()
        .await
        .map_err(|e| format!("Error when querying positions: {:?}", e))?;

    let status = res.status();
    if !status.is_success() {
        let err_text = res.text().await.unwrap_or_default();
        return Err(format!(
            "HTTP error {} when querying positions: {}",
            status,
            err_text
        ));
    }

    let positions: Vec<serde_json::Value> = res
        .json()
        .await
        .map_err(|e| format!("Error interpreting JSON response: {:?}", e))?;

    let mut results = Vec::new();

    for position in positions.into_iter().filter(|p| p["symbol"] == symbol) {
        let amt = position["positionAmt"].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
        let symbol = position["symbol"].as_str().unwrap_or("");

        if amt.abs() < 1e-8 {
            continue;
        }

        let side = if amt > 0.0 { "SELL" } else { "BUY" };
        let quantity = amt.abs();

        let lot_size_info = get_lot_size_info(settings, symbol).await?;
        let quantity_rounded = round_quantity(quantity, lot_size_info.step_size);
        let precision = (1.0 / lot_size_info.step_size).log10().round() as usize;
        let quantity_str = format!("{:.*}", precision, quantity_rounded)
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string();

        let order_url = format!("{}/order", settings.future_url);
        let timestamp = get_timestamp();
        let timestamp_str = timestamp.to_string();

        let mut params = HashMap::new();
        params.insert("symbol", symbol);
        params.insert("side", side);
        params.insert("type", "MARKET");
        params.insert("reduceOnly", "true");
        params.insert("quantity", &quantity_str);
        params.insert("recvWindow", "10000");
        params.insert("timestamp", &timestamp_str);

        let query_string = form_urlencoded::Serializer::new(String::new())
            .extend_pairs(&params)
            .finish();

        let signature = sign_query(&query_string, secret_key);
        let signed_query = format!("{}&signature={}", query_string, signature);

        let response = client
            .post(format!("{}?{}", order_url, signed_query))
            .headers(headers.clone())
            .send()
            .await
            .map_err(|e| format!("Error sending closing order: {:?}", e))?;

        if response.status().is_success() {
            let parsed = response
                .json::<OrderResponse>()
                .await
                .map_err(|e| format!("Error interpreting order: {:?}", e))?;
            results.push(parsed);
        } else {
            let err_text = response.text().await.unwrap_or_default();
            return Err(format!("Error closing position {}: {}", symbol, err_text));
        }
    }

    Ok(results)
}
