use crate::blockchain::get_last_trade_for;
use crate::dto::{Bias, Candlestick, Trade};
use crate::status_trade::update_status;
use chrono::{Local, Timelike};

pub fn generate_trade(symbol: String, candlesticks: Vec<Candlestick>, reference_candles: Vec<Candlestick>) -> Trade {
    let of = candlesticks.len();
    let reference_of = reference_candles.len();

    if of < 271 || reference_of < 271 {
        return Trade {
            symbol,
            current_price: "0.0".into(),
            cma: "0.0".into(),
            oma: "0.0".into(),
            bias: Bias::None,
            status: None,
            zone_max: "0.0".into(),
            zone_7: "0.0".into(),
            zone_6: "0.0".into(),
            zone_5: "0.0".into(),
            zone_4: "0.0".into(),
            zone_3: "0.0".into(),
            zone_2: "0.0".into(),
            zone_1: "0.0".into(),
            zone_min: "0.0".into(),
            of,
            performance_24: "0.0".into(),
            amplitude_ma_200: "0.0".into(),
            performance_btc_24: "0.0".into(),
            volume: "0.0".into(),
            quote_asset_volume: "0.0".into(),
            number_of_trades: "0.0".into(),
            taker_buy_base_asset_volume: "0.0".into(),
            taker_buy_quote_asset_volume: "0.0".into(),
        };
    }

    let cma_valor = calculate_moving_average(&reference_candles[71..]);
    let oma_valor = calculate_moving_average(&reference_candles[..200]);

    let bias = if cma_valor > oma_valor {
        Bias::Bullish
    } else if cma_valor < oma_valor {
        Bias::Bearish
    } else {
        Bias::None
    };

    let analysis_slice = &candlesticks[71..];

    let max_high = analysis_slice
        .iter()
        .filter_map(|c| c.high_price.parse::<f64>().ok())
        .fold(f64::MIN, f64::max);

    let min_low = analysis_slice
        .iter()
        .filter_map(|c| c.low_price.parse::<f64>().ok())
        .fold(f64::MAX, f64::min);

    let current_price = analysis_slice
        .iter()
        .max_by_key(|c| c.close_time)
        .map(|c| c.close_price.clone())
        .unwrap_or_else(|| "0.0".to_string());

    let (volume, quote_asset_volume, number_of_trades, taker_buy_base_asset_volume, taker_buy_quote_asset_volume) =
    match candlesticks.last() {
        Some(candle) => (
            candle.volume.clone(),
            candle.quote_asset_volume.clone(),
            candle.number_of_trades.to_string(),
            candle.taker_buy_base_asset_volume.clone(),
            candle.taker_buy_quote_asset_volume.clone(),
        ),
        None => (
            "0.0".into(),
            "0.0".into(),
            "0".into(),
            "0.0".into(),
            "0.0".into(),
        ),
    };

    let log_min = min_low.ln();
    let log_max = max_high.ln();
    let log_zone_4 = (log_min + log_max) / 2.0;
    let log_zone_2 = (log_min + log_zone_4) / 2.0;
    let log_zone_6 = (log_max + log_zone_4) / 2.0;
    let log_zone_3 = (log_zone_2 + log_zone_4) / 2.0;
    let log_zone_5 = (log_zone_6 + log_zone_4) / 2.0;
    let log_zone_1 = (log_min + log_zone_2) / 2.0;
    let log_zone_7 = (log_max + log_zone_6) / 2.0;

    let performance_24_val = calculate_performance_24(&candlesticks);
    let performance_24 = format!("{:.2}", performance_24_val);
    let amplitude_ma_200 = calculate_amplitude_ma_200(&candlesticks, &current_price);
    let performance_btc_24 = calculate_performance_btc_24(&reference_candles, performance_24_val);

    let trade = Trade {
        symbol: symbol.clone(),
        current_price: current_price.clone(),
        cma: format!("{:.8}", cma_valor),
        oma: format!("{:.8}", oma_valor),
        bias,
        status: None,
        zone_max: format!("{:.8}", max_high),
        zone_7: format!("{:.8}", log_zone_7.exp()),
        zone_6: format!("{:.8}", log_zone_6.exp()),
        zone_5: format!("{:.8}", log_zone_5.exp()),
        zone_4: format!("{:.8}", log_zone_4.exp()),
        zone_3: format!("{:.8}", log_zone_3.exp()),
        zone_2: format!("{:.8}", log_zone_2.exp()),
        zone_1: format!("{:.8}", log_zone_1.exp()),
        zone_min: format!("{:.8}", min_low),
        of,
        performance_24,
        performance_btc_24,
        amplitude_ma_200,
        volume,
        quote_asset_volume,
        number_of_trades,
        taker_buy_base_asset_volume,
        taker_buy_quote_asset_volume,
    };

    match get_last_trade_for(&symbol) {
        Some(ref last) => update_status(trade, last),
        None => trade,
    }
}

fn calculate_amplitude_ma_200(candles: &[Candlestick], current_price_str: &str) -> String {
    if candles.len() < 200 {
        return "0.0".into();
    }
    let oma = calculate_moving_average(&candles[candles.len() - 200..]);
    let current_price = current_price_str.parse::<f64>().unwrap_or(0.0);
    if current_price == 0.0 || oma == 0.0 {
        return "0.0".into();
    }
    let amplitude = (current_price.ln() - oma.ln()) * 100.0;
    format!("{:.2}", amplitude)
}

fn calculate_performance_24(candles: &[Candlestick]) -> f64 {
    if candles.len() < 25 {
        return 0.0;
    }

    let close_now = candles.last().unwrap()
        .close_price
        .parse::<f64>()
        .unwrap_or(0.0);

    let hora_atual = Local::now().hour();
    let horas_ate_21h = (hora_atual + 24 - 21) % 24;
    let horas_ate_21h = horas_ate_21h as usize;

    let index_21h = candles.len().saturating_sub(horas_ate_21h + 1);

    if index_21h >= candles.len() {
        return 0.0;
    }

    let candle_21h = &candles[index_21h];

    if let Ok(open_21h) = candle_21h.open_price.parse::<f64>() {
        if open_21h != 0.0 {
            return ((close_now / open_21h) - 1.0) * 100.0;
        }
    }

    0.0
}

fn calculate_performance_btc_24(candles: &[Candlestick], altcoin_perf_24: f64) -> String {
    if candles.len() < 25 {
        return "0.0".into();
    }

    let close_now = candles.last().unwrap()
        .close_price
        .parse::<f64>()
        .unwrap_or(0.0);

    let hora_atual = Local::now().hour();
    let horas_ate_21h = (hora_atual + 24 - 21) % 24;
    let horas_ate_21h = horas_ate_21h as usize;

    let index_21h = candles.len().saturating_sub(horas_ate_21h + 1);

    if index_21h >= candles.len() {
        return "0.0".into();
    }

    let candle_21h = &candles[index_21h];

    if let Ok(open_21h) = candle_21h.open_price.parse::<f64>() {
        if open_21h != 0.0 && close_now != 0.0 {
            let btc_perf_24 = ((close_now / open_21h) - 1.0) * 100.0;
            let diff = altcoin_perf_24 - btc_perf_24;
            return format!("{:.2}", diff);
        }
    }

    "0.0".into()
}

pub fn calculate_moving_average(candles: &[Candlestick]) -> f64 {
    let soma: f64 = candles
        .iter()
        .filter_map(|c| c.close_price.parse::<f64>().ok())
        .sum();

    soma / candles.len() as f64
}
