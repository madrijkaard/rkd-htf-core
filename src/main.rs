mod dto;
mod api;
mod trade;
mod config;
mod blockchain;
mod order;
mod balance;
mod binance;
mod credential;
mod schedule;
mod leverage;
mod decide;
mod monitor;
mod status_trade;
mod spy;
mod swap;
mod crypto_candidate;
mod open_ai;

use actix_cors::Cors;
use actix_web::{App, HttpServer, http};
use api::{
    post_trades_start,
    post_trades_stop,
    get_trades_health_check,
    get_trades_chain_by_symbol,
    get_last_trade_by_symbol,
    get_all_symbols_chains,
    get_chain_validity,
    post_trades_order,
    get_trades_balance,
    post_close_all_positions,
    put_leverage,
    get_trades_spy,
    get_trades_monitor,
    post_monitor_assistant,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Server running at http://localhost:8080");

    HttpServer::new(|| {
        App::new()
            .wrap(
                Cors::default()
                    .allowed_origin("http://localhost:5173")
                    .allowed_methods(vec!["GET", "POST", "PUT"])
                    .allowed_headers(vec![http::header::CONTENT_TYPE])
                    .supports_credentials()
                    .max_age(3600),
            )
            .service(post_trades_start)
            .service(post_trades_stop)
            .service(get_trades_health_check)
            .service(get_trades_chain_by_symbol)
            .service(get_last_trade_by_symbol)
            .service(get_all_symbols_chains)
            .service(get_chain_validity)
            .service(post_trades_order)
            .service(get_trades_balance)
            .service(post_close_all_positions)
            .service(put_leverage)
            .service(get_trades_spy)
            .service(get_trades_monitor)
            .service(post_monitor_assistant)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
