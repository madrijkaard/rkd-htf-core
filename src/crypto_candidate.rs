use crate::blockchain::{
    add_trade_block, get_current_blockchain_symbols, is_blockchain_limit_reached,
};
use crate::config::Settings;
use crate::decide::decide;
use crate::dto::{Bias, Trade};
use crate::swap::remove_if_out_of_zone;

use rand::seq::SliceRandom;
use rand::thread_rng;

fn parse(value: &str) -> f64 {
    value.parse::<f64>().unwrap_or(0.0)
}

pub async fn process_existing_cryptos(trades: &[Trade], settings: &Settings) {
    let current_symbols = get_current_blockchain_symbols();
    let existing_trades: Vec<Trade> = trades
        .iter()
        .filter(|t| current_symbols.contains(&t.symbol))
        .cloned()
        .collect();

    for trade in &existing_trades {
        let was_added = add_trade_block(trade.clone());
        if was_added && settings.binance.decide {
            decide(&trade.symbol, &settings.binance);
            remove_if_out_of_zone(trade, settings, &settings.binance).await;
        }
    }
}

pub async fn choose_candidate_cryptos(trades: Vec<Trade>, settings: &Settings) {
    let current_symbols = get_current_blockchain_symbols();

    if is_blockchain_limit_reached() {
        return;
    }

    let filtered: Vec<Trade> = trades
        .into_iter()
        .filter(|t| !current_symbols.contains(&t.symbol))
        .filter(|t| {
            let p = parse(&t.current_price);
            match t.bias {
                Bias::Bullish => {
                    let z1 = parse(&t.zone_1);
                    let z6 = parse(&t.zone_6);
                    let z7 = parse(&t.zone_7);
                    p < z1 || (p > z6 && p < z7)
                }
                Bias::Bearish => {
                    let z1 = parse(&t.zone_1);
                    let z2 = parse(&t.zone_2);
                    let z7 = parse(&t.zone_7);
                    p > z7 || (p < z2 && p > z1)
                }
                _ => false,
            }
        })
        .collect();

    let mut bullish_z7 = filtered
        .iter()
        .filter(|t| matches!(t.bias, Bias::Bullish) && {
            let p = parse(&t.current_price);
            p > parse(&t.zone_6) && p < parse(&t.zone_7)
        })
        .max_by(|a, b| parse(&a.performance_btc_24).partial_cmp(&parse(&b.performance_btc_24)).unwrap_or(std::cmp::Ordering::Equal))
        .cloned();

    let mut bullish_z1 = filtered
        .iter()
        .filter(|t| matches!(t.bias, Bias::Bullish) && parse(&t.current_price) < parse(&t.zone_1))
        .min_by(|a, b| parse(&a.amplitude_ma_200).partial_cmp(&parse(&b.amplitude_ma_200)).unwrap_or(std::cmp::Ordering::Equal))
        .cloned();

    let mut bearish_z2 = filtered
        .iter()
        .filter(|t| matches!(t.bias, Bias::Bearish) && {
            let p = parse(&t.current_price);
            p < parse(&t.zone_2) && p > parse(&t.zone_1)
        })
        .min_by(|a, b| parse(&a.performance_btc_24).partial_cmp(&parse(&b.performance_btc_24)).unwrap_or(std::cmp::Ordering::Equal))
        .cloned();

    let mut bearish_z8 = filtered
        .iter()
        .filter(|t| matches!(t.bias, Bias::Bearish) && parse(&t.current_price) > parse(&t.zone_7))
        .max_by(|a, b| parse(&a.amplitude_ma_200).partial_cmp(&parse(&b.amplitude_ma_200)).unwrap_or(std::cmp::Ordering::Equal))
        .cloned();

    let mut final_candidates = vec![];
    if let Some(t) = bullish_z7.take() { final_candidates.push(t); }
    if let Some(t) = bullish_z1.take() { final_candidates.push(t); }
    if let Some(t) = bearish_z2.take() { final_candidates.push(t); }
    if let Some(t) = bearish_z8.take() { final_candidates.push(t); }

    if final_candidates.is_empty() {
        return;
    }

    if let Some(selected) = {
        let mut rng = thread_rng();
        final_candidates.choose(&mut rng).cloned()
    } {
        let was_added = add_trade_block(selected.clone());
        if was_added && settings.binance.decide {
            decide(&selected.symbol, &settings.binance);
            remove_if_out_of_zone(&selected, settings, &settings.binance).await;
        }
    }
}
