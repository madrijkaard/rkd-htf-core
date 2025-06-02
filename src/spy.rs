use crate::binance::get_candlesticks;
use crate::trade::generate_trade;
use crate::dto::Trade;
use futures::future::join_all;

pub async fn spy_cryptos(
    base_url: &str,
    interval: &str,
    limit: u32,
    symbols: Vec<String>,
) -> Vec<Trade> {
    let tasks = symbols.into_iter().map(|symbol| {
        let base_url = base_url.to_string();
        let interval = interval.to_string();
        let symbol_clone = symbol.clone();

        tokio::spawn(async move {
            let candles = get_candlesticks(&base_url, &symbol_clone, &interval, limit).await?;
            let ref_data = get_candlesticks(&base_url, "BTCUSDT", &interval, limit).await?;
            let trade = generate_trade(symbol_clone, candles, ref_data);
            Ok::<_, String>(trade)
        })
    });

    let results = join_all(tasks).await;

    results
        .into_iter()
        .filter_map(|r| r.ok().and_then(|res| res.ok()))
        .collect()
}
