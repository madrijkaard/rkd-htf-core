use crate::dto::{Bias, Trade, TradeStatus};

pub fn update_status(mut trade: Trade, last: &Trade) -> Trade {
    if trade.bias != last.bias {
        trade.status = None;
        return trade;
    }

    let current_price = parse(&trade.current_price);
    let zone_1 = parse(&trade.zone_1);
    let zone_2 = parse(&trade.zone_2);
    let zone_3 = parse(&trade.zone_3);
    let zone_5 = parse(&trade.zone_5);
    let zone_6 = parse(&trade.zone_6);
    let zone_7 = parse(&trade.zone_7);

    match trade.bias {
        Bias::Bullish => handle_bullish_status(
            &mut trade,
            current_price,
            zone_1,
            zone_3,
            zone_5,
            zone_6,
            zone_7,
            last,
        ),
        Bias::Bearish => handle_bearish_status(
            &mut trade,
            current_price,
            zone_1,
            zone_2,
            zone_3,
            zone_5,
            zone_7,
            last,
        ),
        Bias::None => {
            trade.status = None;
        }
    }

    trade
}

fn handle_bullish_status(
    trade: &mut Trade,
    current_price: f64,
    zone_1: f64,
    zone_3: f64,
    zone_5: f64,
    zone_6: f64,
    zone_7: f64,
    last: &Trade,
) {
    use TradeStatus::*;

    match last.status {
        None if current_price >= zone_7 => trade.status = Some(InZone7),
        None if current_price <= zone_1 => trade.status = Some(PrepareZone1),

        Some(OutZone5) if current_price >= zone_7 => trade.status = Some(InZone7),
        Some(InZone7) if current_price > zone_5 => trade.status = Some(InZone7),
        Some(InZone7) if current_price <= zone_5 => trade.status = Some(OutZone5),

        Some(OutZone5) if current_price < zone_7 && current_price > zone_1 => {
            trade.status = Some(OutZone5)
        }
        Some(OutZone5) if current_price <= zone_1 => trade.status = Some(PrepareZone1),
        Some(PrepareZone1) if current_price < zone_3 => trade.status = Some(PrepareZone1),
        Some(PrepareZone1) if current_price >= zone_3 => trade.status = Some(InZone3),
        Some(InZone3) if current_price >= zone_7 => trade.status = Some(TargetZone7),
        Some(InZone3) if current_price < zone_7 && current_price > zone_1 => {
            trade.status = Some(InZone3)
        }
        Some(InZone3) if current_price <= zone_1 => trade.status = Some(PrepareZone1Long),
        Some(PrepareZone1Long) if current_price < zone_3 => {
            trade.status = Some(PrepareZone1Long)
        }
        Some(PrepareZone1Long) if current_price >= zone_3 => trade.status = Some(LongZone3),
        Some(LongZone3) if current_price > zone_1 && current_price < zone_7 => {
            trade.status = Some(LongZone3)
        }
        Some(LongZone3) if current_price <= zone_1 => trade.status = Some(PrepareZone1),
        Some(LongZone3) if current_price >= zone_7 => trade.status = Some(TargetZone7),
        Some(TargetZone7) if current_price > zone_6 => trade.status = Some(TargetZone7),
        Some(TargetZone7) if current_price <= zone_6 => trade.status = None,
        _ => {}
    }
}

fn handle_bearish_status(
    trade: &mut Trade,
    current_price: f64,
    zone_1: f64,
    zone_2: f64,
    zone_3: f64,
    zone_5: f64,
    zone_7: f64,
    last: &Trade,
) {
    use TradeStatus::*;

    match last.status {
        None if current_price <= zone_1 => trade.status = Some(InZone1),
        None if current_price >= zone_7 => trade.status = Some(PrepareZone7),

        Some(OutZone3) if current_price <= zone_1 => trade.status = Some(InZone1),
        Some(InZone1) if current_price < zone_3 => trade.status = Some(InZone1),
        Some(InZone1) if current_price >= zone_3 => trade.status = Some(OutZone3),
        Some(OutZone3) if current_price > zone_1 && current_price < zone_7 => {
            trade.status = Some(OutZone3)
        }
        Some(OutZone3) if current_price >= zone_7 => trade.status = Some(PrepareZone7),
        Some(PrepareZone7) if current_price > zone_5 => trade.status = Some(PrepareZone7),
        Some(PrepareZone7) if current_price <= zone_5 => trade.status = Some(InZone5),
        Some(InZone5) if current_price <= zone_1 => trade.status = Some(TargetZone1),
        Some(InZone5) if current_price > zone_1 && current_price < zone_7 => {
            trade.status = Some(InZone5)
        }
        Some(InZone5) if current_price >= zone_7 => trade.status = Some(PrepareZone7Short),
        Some(PrepareZone7Short) if current_price > zone_5 => {
            trade.status = Some(PrepareZone7Short)
        }
        Some(PrepareZone7Short) if current_price <= zone_5 => trade.status = Some(ShortZone5),
        Some(ShortZone5) if current_price < zone_7 && current_price > zone_1 => {
            trade.status = Some(ShortZone5)
        }
        Some(ShortZone5) if current_price >= zone_7 => trade.status = Some(PrepareZone7),
        Some(ShortZone5) if current_price <= zone_1 => trade.status = Some(TargetZone1),
        Some(TargetZone1) if current_price < zone_2 => trade.status = Some(TargetZone1),
        Some(TargetZone1) if current_price >= zone_2 => trade.status = None,
        _ => {}
    }
}

fn parse(value: &str) -> f64 {
    value.parse::<f64>().unwrap_or(0.0)
}
