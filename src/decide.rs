use crate::blockchain::get_last_trade_for;
use crate::dto::{Bias, TradeStatus};
use crate::order::{execute_future_order, close_all_positions};
use crate::config::BinanceSettings;
use crate::leverage::set_leverage_with_value;

pub fn decide(symbol: &str, binance_settings: &BinanceSettings) {
    let trade = match get_last_trade_for(symbol) {
        Some(t) => t,
        None => {
            println!("No trades found for decision for symbol: {}", symbol);
            return;
        }
    };

    let bias = trade.bias.clone();
    let status = trade.status.clone();
    let symbol = &trade.symbol;

    match (bias, status) {
        (_, None) => {
            let binance = binance_settings.clone();
            let symbol = symbol.clone();
            tokio::spawn(async move {
                match close_all_positions(&binance, &symbol).await {
                    Ok(closed) => println!("All positions closed (status None): {:?}", closed),
                    Err(e) => eprintln!("Error closing positions (status None): {}", e),
                }
                if let Err(e) = set_leverage_with_value(&binance, &symbol, 1).await {
                    eprintln!("Error setting leverage to 1 (status None): {}", e);
                }
            });
        }

        (Bias::Bullish, Some(TradeStatus::InZone7))
        | (Bias::Bullish, Some(TradeStatus::InZone3))
        | (Bias::Bullish, Some(TradeStatus::LongZone3)) => {
            let binance = binance_settings.clone();
            let symbol = symbol.clone();
            tokio::spawn(async move {
                match execute_future_order(&binance, "BUY", &symbol).await {
                    Ok(order) => println!("BUY order executed: {:?}", order),
                    Err(e) => eprintln!("Error executing BUY order: {}", e),
                }
            });
        }

        (Bias::Bearish, Some(TradeStatus::InZone1))
        | (Bias::Bearish, Some(TradeStatus::InZone5))
        | (Bias::Bearish, Some(TradeStatus::ShortZone5)) => {
            let binance = binance_settings.clone();
            let symbol = symbol.clone();
            tokio::spawn(async move {
                match execute_future_order(&binance, "SELL", &symbol).await {
                    Ok(order) => println!("SELL order executed: {:?}", order),
                    Err(e) => eprintln!("Error executing SELL order: {}", e),
                }
            });
        }

        (Bias::Bullish, Some(TradeStatus::TargetZone7))
        | (Bias::Bearish, Some(TradeStatus::TargetZone1)) => {
            let binance = binance_settings.clone();
            let symbol = symbol.clone();
            tokio::spawn(async move {
                if let Err(e) = set_leverage_with_value(&binance, &symbol, 1).await {
                    eprintln!("Error setting leverage to 1 (target zone): {}", e);
                }
            });
        }

        (Bias::Bullish, Some(TradeStatus::OutZone5))
        | (Bias::Bullish, Some(TradeStatus::PrepareZone1))
        | (Bias::Bearish, Some(TradeStatus::OutZone3))
        | (Bias::Bearish, Some(TradeStatus::PrepareZone7)) => {
            let binance = binance_settings.clone();
            let symbol = symbol.clone();
            tokio::spawn(async move {
                match close_all_positions(&binance, &symbol).await {
                    Ok(closed) => println!("Closed positions (lev 1): {:?}", closed),
                    Err(e) => eprintln!("Error closing positions: {}", e),
                }
                if let Err(e) = set_leverage_with_value(&binance, &symbol, 1).await {
                    eprintln!("Error setting leverage to 1: {}", e);
                }
            });
        }

        (Bias::Bullish, Some(TradeStatus::PrepareZone1Long))
        | (Bias::Bearish, Some(TradeStatus::PrepareZone7Short)) => {
            let binance = binance_settings.clone();
            let symbol = symbol.clone();
            tokio::spawn(async move {
                match close_all_positions(&binance, &symbol).await {
                    Ok(closed) => println!("Closed positions (lev 2): {:?}", closed),
                    Err(e) => eprintln!("Error closing positions: {}", e),
                }
                if let Err(e) = set_leverage_with_value(&binance, &symbol, 2).await {
                    eprintln!("Error setting leverage to 2: {}", e);
                }
            });
        }

        _ => {
            println!(
                "No action taken for status: {:?} with bias: {:?} (symbol: {})",
                trade.status,
                trade.bias,
                trade.symbol
            );
        }
    }
}
