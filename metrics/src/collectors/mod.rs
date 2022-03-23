use crate::{Configuration};

mod sql;
mod health;
mod metadata;
mod scheduled_events;
mod cron_triggers;
mod event_triggers;

pub(crate) async fn run_metadata_collector(cfg: &Configuration) -> std::io::Result<()> {
    loop {
        tokio::join!(
            health::check_health(cfg),
            metadata::check_metadata(cfg),
            scheduled_events::check_scheduled_events(&cfg),
            cron_triggers::check_cron_triggers(&cfg),
            event_triggers::check_event_triggers(&cfg),
        );
        tokio::time::sleep(std::time::Duration::from_millis(cfg.collect_interval)).await;
    }
}