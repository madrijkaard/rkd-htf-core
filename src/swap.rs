use crate::dto::{Bias, Trade, TradeStatus};
use crate::blockchain::{remove_blockchain, get_blockchain_for};
use crate::config::{Settings, BinanceSettings};
use crate::order::close_all_positions;
use crate::credential::get_credentials;
use crate::binance::get_unrealized_profit;

pub async fn remove_if_out_of_zone(
    trade: &Trade,
    settings: &Settings,
    binance_settings: &BinanceSettings,
) {
    let credentials = get_credentials();
    if let Ok(Some(pnl)) = get_unrealized_profit(
        binance_settings,
        &trade.symbol,
        &credentials.key,
        &credentials.secret,
    )
    .await
    {
        if pnl >= settings.gain {
            match close_all_positions(binance_settings, &trade.symbol).await {
                Ok(_) => println!(
                    "[{}] Lucro {:.2} ≥ alvo ({:.2}) - posição fechada para {}",
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                    pnl,
                    settings.gain,
                    trade.symbol
                ),
                Err(e) => eprintln!("Erro ao fechar posição {}: {}", trade.symbol, e),
            }
            remove_blockchain(&trade.symbol);
            return;
        }
    }

    use TradeStatus::*;

    if matches!(trade.bias, Bias::Bullish) && matches!(trade.status, Some(OutZone5))
        || matches!(trade.bias, Bias::Bearish) && matches!(trade.status, Some(OutZone3))
    {
        remove_blockchain(&trade.symbol);
        return;
    }

    if let Some(blocks) = get_blockchain_for(&trade.symbol) {
        if blocks.len() >= 2 {
            let last_status = blocks[blocks.len() - 1].trade.status.clone();
            let previous_status = blocks[blocks.len() - 2].trade.status.clone();

            match trade.bias {
                Bias::Bullish => {
                    if (last_status == Some(PrepareZone1) && previous_status == Some(LongZone3))
                        || (last_status == None && previous_status == Some(TargetZone7))
                    {
                        remove_blockchain(&trade.symbol);
                        return;
                    }
                }
                Bias::Bearish => {
                    if (last_status == Some(PrepareZone7) && previous_status == Some(ShortZone5))
                        || (last_status == None && previous_status == Some(TargetZone1))
                    {
                        remove_blockchain(&trade.symbol);
                        return;
                    }
                }
                _ => {}
            }
        }
    }

    if trade.status.is_none() {
        let price = parse(&trade.current_price);
        let z4 = parse(&trade.zone_4);
        let z5 = parse(&trade.zone_5);

        if price > z4 && price <= z5 {
            remove_blockchain(&trade.symbol);
        }
    }
}

fn parse(value: &str) -> f64 {
    value.parse::<f64>().unwrap_or(0.0)
}
