use actix_web::{get, post, put, web, HttpResponse, Responder};
use crate::balance::get_futures_balance;
use crate::config::Settings;
use crate::dto::{OpenOrderRequest, SymbolRequest};
use crate::leverage::set_leverage;
use crate::order::{close_all_positions, execute_future_order};
use crate::schedule::get_scheduler;
use crate::blockchain::{get_blockchain_for, get_last_trade_for, get_all_symbols, BLOCKCHAIN};
use crate::spy::spy_cryptos;
use crate::monitor::monitor_cryptos;
use crate::open_ai::send_to_assistant;

use std::fmt::Write;

#[post("/trades/start")]
pub async fn post_trades_start() -> impl Responder {
    let scheduler = get_scheduler();
    let mut scheduler = scheduler.lock().unwrap();
    scheduler.start();
    HttpResponse::Ok().body("Timer started")
}

#[post("/trades/stop")]
pub async fn post_trades_stop() -> impl Responder {
    let scheduler = get_scheduler();
    let mut scheduler = scheduler.lock().unwrap();
    scheduler.stop();
    HttpResponse::Ok().body("Timer stopped")
}

#[get("/trades/health-check")]
pub async fn get_trades_health_check() -> impl Responder {
    let scheduler = get_scheduler();
    let scheduler = scheduler.lock().unwrap();
    let status = if scheduler.is_active() { "UP" } else { "DOWN" };
    HttpResponse::Ok().body(format!("status: {}", status))
}

#[get("/trades/chains/{symbol}")]
pub async fn get_trades_chain_by_symbol(path: web::Path<String>) -> impl Responder {
    let symbol = path.into_inner();
    match get_blockchain_for(&symbol) {
        Some(chain) => HttpResponse::Ok().json(chain),
        None => HttpResponse::NotFound().body(format!("Nenhuma blockchain encontrada para {}", symbol)),
    }
}

#[get("/trades/chains/{symbol}/last")]
pub async fn get_last_trade_by_symbol(path: web::Path<String>) -> impl Responder {
    let symbol = path.into_inner();
    match get_last_trade_for(&symbol) {
        Some(trade) => HttpResponse::Ok().json(trade),
        None => HttpResponse::NotFound().body(format!("Nenhum trade encontrado para {}", symbol)),
    }
}

#[get("/trades/balance")]
pub async fn get_trades_balance() -> impl Responder {
    let settings = Settings::load();

    match get_futures_balance(&settings.binance).await {
        Ok(balances) => {
            let usdt_balance: Vec<_> = balances
                .into_iter()
                .filter(|b| b.asset == "USDT")
                .collect();
            HttpResponse::Ok().json(usdt_balance)
        }
        Err(e) => {
            eprintln!("Erro ao consultar saldo de futuros: {}", e);
            HttpResponse::InternalServerError().body(format!("Erro: {}", e))
        }
    }
}

#[post("/trades/order/open")]
pub async fn post_trades_order(req: web::Json<OpenOrderRequest>) -> impl Responder {
    let settings = Settings::load();
    let binance_settings = &settings.binance;

    let side = req.side.to_uppercase();
    let symbol = req.symbol.to_uppercase();

    if side != "BUY" && side != "SELL" {
        return HttpResponse::BadRequest().body("O parâmetro 'side' deve ser 'BUY' ou 'SELL'");
    }

    match execute_future_order(binance_settings, &side, &symbol).await {
        Ok(order) => HttpResponse::Ok().json(order),
        Err(e) => {
            eprintln!("Erro ao enviar ordem para Binance: {}", e);
            HttpResponse::InternalServerError().body(e)
        }
    }
}

#[post("/trades/order/close")]
pub async fn post_close_all_positions(req: web::Json<SymbolRequest>) -> impl Responder {
    let settings = Settings::load();
    let binance_settings = &settings.binance;

    match close_all_positions(binance_settings, &req.symbol).await {
        Ok(orders) => HttpResponse::Ok().json(orders),
        Err(e) => {
            eprintln!("Erro ao fechar posições: {}", e);
            HttpResponse::InternalServerError().body(e)
        }
    }
}

#[put("/trades/leverage")]
pub async fn put_leverage(req: web::Json<SymbolRequest>) -> impl Responder {
    let settings = Settings::load();
    let symbol = &req.symbol;

    match set_leverage(&settings.binance, symbol).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            eprintln!("Erro ao aplicar alavancagem: {}", e);
            HttpResponse::InternalServerError().body(format!("Erro: {}", e))
        }
    }
}

#[get("/trades/spy")]
pub async fn get_trades_spy() -> impl Responder {
    let settings = Settings::load();

    if !settings.spy {
        return HttpResponse::Forbidden().body("Serviço /trades/spy está desativado na configuração");
    }

    let binance_settings = settings.binance.clone();
    let cryptos = settings.cryptos.clone();

    let trades = spy_cryptos(
        &binance_settings.base_url,
        &binance_settings.interval,
        binance_settings.limit,
        cryptos,
    )
    .await;

    HttpResponse::Ok().json(trades)
}

#[get("/trades/chains")]
pub async fn get_all_symbols_chains() -> impl Responder {
    let symbols = get_all_symbols();
    HttpResponse::Ok().json(symbols)
}

#[get("/trades/chains/{symbol}/valid")]
pub async fn get_chain_validity(path: web::Path<String>) -> impl Responder {
    let symbol = path.into_inner();
    let map = BLOCKCHAIN.lock().unwrap();

    match map.get(&symbol) {
        Some(chain) => {
            if chain.is_valid() {
                HttpResponse::Ok().body("Blockchain válida")
            } else {
                HttpResponse::Conflict().body("Blockchain corrompida")
            }
        }
        None => HttpResponse::NotFound().body("Blockchain não encontrada"),
    }
}

#[get("/trades/monitor")]
pub async fn get_trades_monitor(query: web::Query<std::collections::HashMap<String, String>>) -> impl Responder {
    let settings = Settings::load();

    let trades = spy_cryptos(
        &settings.binance.base_url,
        &settings.binance.interval,
        settings.binance.limit,
        settings.cryptos.clone(),
    )
    .await;

    let response = monitor_cryptos(&trades, &settings);

    match query.get("format").map(|f| f.as_str()) {
        Some("text") => {
            let mut buffer = String::new();
            writeln!(&mut buffer, "[{}] - Criptos monitoradas:", response.timestamp).unwrap();

            buffer.push_str("\n(tabela disponível apenas no terminal)\n");
            writeln!(
                &mut buffer,
                "\nDistribuicao por zona: {}",
                response.zone_distribution
                    .iter()
                    .map(|z| format!("{}: {}", z.zone, z.count))
                    .collect::<Vec<_>>()
                    .join(" | ")
            )
            .unwrap();

            HttpResponse::Ok()
                .content_type("text/plain; charset=utf-8")
                .body(buffer)
        }
        _ => HttpResponse::Ok().json(response),
    }
}

#[post("/monitors/assistant")]
pub async fn post_monitor_assistant() -> impl Responder {
    match send_to_assistant().await {
        Ok(resposta) => {
            println!("Resposta do ChatGPT:\n{}", resposta.content);
            HttpResponse::Ok().json(serde_json::json!({
                "response": resposta.content
            }))
        }
        Err(err) => {
            eprintln!("Erro ao chamar ChatGPT: {}", err);
            HttpResponse::InternalServerError().body("Erro ao chamar ChatGPT.")
        }
    }
}

