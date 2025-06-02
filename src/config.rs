use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct BinanceSettings {
    pub base_url: String,
    pub future_url: String,
    pub future_url_v2: String,
    pub interval: String,
    pub limit: u32,
    pub leverage: u32,
    pub decide: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub binance: BinanceSettings,
    pub spy: bool,
    pub limit_operations: usize,
    pub cryptos: Vec<String>,
    pub money: f64,
    pub gain: f64,
    pub show_details_monitor: bool,
}

impl Settings {
    pub fn load() -> Self {
        config::Config::builder()
            .add_source(config::File::with_name("config/Settings").required(true))
            .build()
            .expect("Failed to load configuration file")
            .try_deserialize()
            .expect("Failed to deserialize configuration")
    }
}
