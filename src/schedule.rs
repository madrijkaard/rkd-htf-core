use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::time::interval;
use once_cell::sync::Lazy;

use crate::config::Settings;
use crate::spy::spy_cryptos;
use crate::monitor::monitor_cryptos;
use crate::crypto_candidate::{process_existing_cryptos, choose_candidate_cryptos};

static SCHEDULER: Lazy<Arc<Mutex<Scheduler>>> = Lazy::new(|| Arc::new(Mutex::new(Scheduler::new())));

pub struct Scheduler {
    active: bool,
    handle: Option<JoinHandle<()>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            active: false,
            handle: None,
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn start(&mut self) {
        if self.active {
            return;
        }

        self.active = true;
        let settings = Settings::load();

        self.handle = Some(tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(50));

            loop {
                interval.tick().await;
                execute_trade(&settings).await;
            }
        }));
    }

    pub fn stop(&mut self) {
        self.active = false;
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }
}

pub fn get_scheduler() -> Arc<Mutex<Scheduler>> {
    SCHEDULER.clone()
}

async fn execute_trade(settings: &Settings) {
    
    let trades = spy_cryptos(
        &settings.binance.base_url,
        &settings.binance.interval,
        settings.binance.limit,
        settings.cryptos.clone(),
    )
    .await;

    monitor_cryptos(&trades, settings);
    process_existing_cryptos(&trades, settings).await;
    choose_candidate_cryptos(trades, settings).await;
}
