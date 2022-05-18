use std::sync::mpsc;
use std::sync::mpsc::RecvTimeoutError;
use crate::{Configuration};

mod sql;
mod health;
mod metadata;
mod scheduled_events;
mod cron_triggers;
mod event_triggers;

pub(crate) async fn run_metadata_collector(cfg: &Configuration, termination_rx: &mpsc::Receiver<()>) -> std::io::Result<()> {
    loop {
        tokio::join!(
            health::check_health(cfg),
            metadata::check_metadata(cfg),
            scheduled_events::check_scheduled_events(&cfg),
            cron_triggers::check_cron_triggers(&cfg),
            event_triggers::check_event_triggers(&cfg),
        );

        match termination_rx.recv_timeout(std::time::Duration::from_millis(cfg.collect_interval)) {
            Ok(_) | Err(RecvTimeoutError::Disconnected) => return Ok(()),
            Err(RecvTimeoutError::Timeout) => () //continue
        }
    }
}
