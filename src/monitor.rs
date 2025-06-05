use crate::blockchain::get_current_blockchain_symbols;
use crate::config::Settings;
use crate::crypto_metadata::get_crypto_metadata;
use crate::dto::{Bias, Trade, TradeMonitorItem, TradeMonitorResponse, ZoneCount};
use chrono::Local;
use prettytable::{color, Attr, Cell, Row, Table};

pub fn monitor_cryptos(trades: &[Trade], settings: &Settings) -> TradeMonitorResponse {
    fn parse(value: &str) -> f64 {
        value.parse::<f64>().unwrap_or(0.0)
    }

    fn find_zone_index(trade: &Trade) -> Option<usize> {
        let price = parse(&trade.current_price);
        let zones = vec![
            parse(&trade.zone_1),
            parse(&trade.zone_2),
            parse(&trade.zone_3),
            parse(&trade.zone_4),
            parse(&trade.zone_5),
            parse(&trade.zone_6),
            parse(&trade.zone_7),
            f64::MAX,
        ];

        for i in 0..zones.len() {
            let lower = if i == 0 { 0.0 } else { zones[i - 1] };
            let upper = zones[i];
            if price > lower && price <= upper || (i == 0 && price <= lower) {
                return Some(i);
            }
        }
        None
    }

    fn zone_label_cell(index: Option<usize>, bias: &Bias) -> Cell {
        match index {
            Some(i) => {
                let zone_str = format!("Z{}", i + 1);
                match (i, bias) {
                    (0, Bias::Bullish) | (6, Bias::Bullish) => {
                        Cell::new(&zone_str).with_style(Attr::ForegroundColor(color::BLUE))
                    }
                    (1, Bias::Bearish) | (7, Bias::Bearish) => {
                        Cell::new(&zone_str).with_style(Attr::ForegroundColor(color::BLUE))
                    }
                    _ => Cell::new(&zone_str),
                }
            }
            None => Cell::new("-"),
        }
    }

    fn color_unique(index: usize, max_index: usize, min_index: usize) -> Option<Attr> {
        if index == max_index {
            Some(Attr::ForegroundColor(color::GREEN))
        } else if index == min_index {
            Some(Attr::ForegroundColor(color::RED))
        } else {
            None
        }
    }

    fn calc_linear_ampl(min: f64, max: f64) -> f64 {
        if min <= 0.0 || max <= 0.0 || min >= max {
            return 0.0;
        }
        ((max - min) / min) * 100.0
    }

    fn calc_linear_position(price: f64, min: f64, max: f64) -> f64 {
        if min <= 0.0 || max <= 0.0 || price <= 0.0 || min >= max {
            return 0.0;
        }
        ((price - min) / (max - min)) * 100.0
    }

    fn extract_column(values: &[(f64, f64, f64, f64, f64, f64, f64, f64, f64, f64)], index: usize) -> Vec<f64> {
        values.iter().map(|v| match index {
            0 => v.0,
            1 => v.1,
            2 => v.2,
            3 => v.3,
            4 => v.4,
            5 => v.5,
            6 => v.6,
            7 => v.7,
            8 => v.8,
            9 => v.9,
            _ => 0.0,
        }).collect()
    }

    fn max_index(v: &[f64]) -> usize {
        v.iter().enumerate().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).map(|(i, _)| i).unwrap_or(0)
    }

    fn min_index(v: &[f64]) -> usize {
        v.iter().enumerate().min_by(|a, b| a.1.partial_cmp(b.1).unwrap()).map(|(i, _)| i).unwrap_or(0)
    }

    print!("\x1B[2J\x1B[1;1H");
    let now = Local::now();
    println!("[{}] - Criptos monitoradas:", now.format("%Y-%m-%d %H:%M:%S"));

    let metadata_list = get_crypto_metadata();
    let active_symbols = get_current_blockchain_symbols();
    let mut table = Table::new();
    let show_details = settings.show_details_monitor;

    if show_details {
        table.add_row(Row::new(vec![
            Cell::new("Symbol"), Cell::new("Zone"), Cell::new("24h"), Cell::new("BTC"), Cell::new("MA200"),
            Cell::new("Ampl"), Cell::new("Pos%"), Cell::new("Volume"), Cell::new("Quote Volume"),
            Cell::new("Trades"), Cell::new("Taker Base"), Cell::new("Taker Quote"),
        ]));
    } else {
        table.add_row(Row::new(vec![
            Cell::new("Symbol"), Cell::new("Zone"), Cell::new("24h"), Cell::new("BTC"), Cell::new("MA200"),
        ]));
    }

    let values: Vec<_> = trades.iter().map(|t| {
        let min = parse(&t.zone_min);
        let max = parse(&t.zone_max);
        let current = parse(&t.current_price);
        (
            parse(&t.performance_24),
            parse(&t.performance_btc_24),
            parse(&t.amplitude_ma_200),
            calc_linear_ampl(min, max),
            calc_linear_position(current, min, max),
            parse(&t.volume),
            parse(&t.quote_asset_volume),
            parse(&t.number_of_trades),
            parse(&t.taker_buy_base_asset_volume),
            parse(&t.taker_buy_quote_asset_volume),
        )
    }).collect();

    let perf_col = extract_column(&values, 0);
    let btc_col = extract_column(&values, 1);
    let ma200_col = extract_column(&values, 2);
    let ampl_col = extract_column(&values, 3);
    let pos_col = extract_column(&values, 4);
    let volume_col = extract_column(&values, 5);
    let quote_col = extract_column(&values, 6);
    let trades_col = extract_column(&values, 7);
    let taker_base_col = extract_column(&values, 8);
    let taker_quote_col = extract_column(&values, 9);

    let max_perf = max_index(&perf_col);
    let min_perf = min_index(&perf_col);
    let max_btc = max_index(&btc_col);
    let min_btc = min_index(&btc_col);
    let max_ma200 = max_index(&ma200_col);
    let min_ma200 = min_index(&ma200_col);
    let max_ampl = max_index(&ampl_col);
    let min_ampl = min_index(&ampl_col);
    let max_pos = max_index(&pos_col);
    let min_pos = min_index(&pos_col);
    let max_vol = max_index(&volume_col);
    let min_vol = min_index(&volume_col);
    let max_quote = max_index(&quote_col);
    let min_quote = min_index(&quote_col);
    let max_trades = max_index(&trades_col);
    let min_trades = min_index(&trades_col);
    let max_taker_base = max_index(&taker_base_col);
    let min_taker_base = min_index(&taker_base_col);
    let max_taker_quote = max_index(&taker_quote_col);
    let min_taker_quote = min_index(&taker_quote_col);

    let mut json_items = vec![];
    let mut zone_counts = [0usize; 8];

    for (i, t) in trades.iter().enumerate() {
        let zone_index = find_zone_index(t);
        if let Some(z) = zone_index {
            zone_counts[z] += 1;
        }

        let mut symbol_cell = Cell::new(&t.symbol);
        if active_symbols.contains(&t.symbol) {
            symbol_cell = symbol_cell.with_style(Attr::ForegroundColor(color::YELLOW));
        }

        let mut row = vec![
            symbol_cell,
            zone_label_cell(zone_index, &t.bias),
        ];

        macro_rules! push_cell {
            ($vec:expr, $col:expr, $i:expr, $max:expr, $min:expr) => {
                {
                    let mut cell = Cell::new(&format!("{:.2}", $col[$i]));
                    if let Some(attr) = color_unique($i, $max, $min) {
                        cell = cell.with_style(attr);
                    }
                    $vec.push(cell);
                }
            };
        }

        push_cell!(row, perf_col, i, max_perf, min_perf);
        push_cell!(row, btc_col, i, max_btc, min_btc);
        push_cell!(row, ma200_col, i, max_ma200, min_ma200);

        if show_details {
            push_cell!(row, ampl_col, i, max_ampl, min_ampl);
            push_cell!(row, pos_col, i, max_pos, min_pos);
            push_cell!(row, volume_col, i, max_vol, min_vol);
            push_cell!(row, quote_col, i, max_quote, min_quote);
            push_cell!(row, trades_col, i, max_trades, min_trades);
            push_cell!(row, taker_base_col, i, max_taker_base, min_taker_base);
            push_cell!(row, taker_quote_col, i, max_taker_quote, min_taker_quote);
        }

        table.add_row(Row::new(row));

        let base_symbol = t.symbol.trim_end_matches("USDT");
        let metadata = metadata_list.iter().find(|m| m.symbol.eq_ignore_ascii_case(base_symbol));

        json_items.push(TradeMonitorItem {
            symbol: t.symbol.clone(),
            zone: zone_index.map(|i| format!("Z{}", i + 1)),
            performance_24: perf_col[i],
            performance_btc_24: btc_col[i],
            amplitude_ma_200: ma200_col[i],
            log_amplitude: ampl_col[i],
            log_position: pos_col[i],
            volume: volume_col[i],
            quote_volume: quote_col[i],
            trades_count: trades_col[i],
            taker_buy_base_volume: taker_base_col[i],
            taker_buy_quote_volume: taker_quote_col[i],
            is_active: active_symbols.contains(&t.symbol),
            logo: metadata.and_then(|m| m.logo.clone()),
            name: metadata.and_then(|m| m.name.clone()),
            description: metadata.and_then(|m| m.description.clone()),
            date_added: metadata.and_then(|m| m.date_added.clone()),
            website: metadata.and_then(|m| m.website.clone()),
            technical_doc: metadata.and_then(|m| m.technical_doc.clone()),
        });
    }

    table.printstd();

    println!(
        "\nDistribuicao por zona: {}",
        zone_counts
            .iter()
            .enumerate()
            .map(|(i, count)| format!("Z{}: {}", i + 1, count))
            .collect::<Vec<_>>()
            .join(" | ")
    );

    let json_distribution: Vec<ZoneCount> = zone_counts
        .iter()
        .enumerate()
        .map(|(i, count)| ZoneCount {
            zone: format!("Z{}", i + 1),
            count: *count,
        })
        .collect();

    TradeMonitorResponse {
        timestamp: now.to_rfc3339(),
        trades: json_items,
        zone_distribution: json_distribution,
    }
}
