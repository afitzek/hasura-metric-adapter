use std::sync::mpsc;
use crate::{Configuration, util};

mod sql;
mod health;
mod metadata;
mod scheduled_events;
mod cron_triggers;
mod event_triggers;

pub(crate) async fn run_metadata_collector(cfg: &Configuration, rx: &mpsc::Receiver<()>) -> std::io::Result<()> {
    loop {
        tokio::join!(
            health::check_health(cfg),
            metadata::check_metadata(cfg),
            scheduled_events::check_scheduled_events(&cfg),
            cron_triggers::check_cron_triggers(&cfg),
            event_triggers::check_event_triggers(&cfg),
        );
        for _i in 1..100 { // split up the interval so it stops in a useful amount of time
            if util::should_stop(rx) {
                return Ok(());
            }

            tokio::time::sleep(std::time::Duration::from_millis(cfg.collect_interval / 100)).await;
        }
    }
}