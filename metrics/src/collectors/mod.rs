use std::sync::mpsc;
use std::sync::mpsc::RecvTimeoutError;
use log::debug;
use crate::{Configuration, Telemetry};

mod sql;
mod health;
mod metadata;
mod scheduled_events;
mod cron_triggers;
mod event_triggers;

pub(crate) async fn run_metadata_collector(cfg: &Configuration, metric_obj: &Telemetry, termination_rx: &mpsc::Receiver<()>) -> std::io::Result<()> {
    loop {
        debug!("Running metadata collector");

        tokio::join!(
            health::check_health(cfg,metric_obj),
            scheduled_events::check_scheduled_events(&cfg,metric_obj),
            cron_triggers::check_cron_triggers(&cfg,metric_obj),
            {
                metadata = metadata::check_metadata(cfg,metric_obj).await;
                event_triggers::check_event_triggers(&cfg,metric_obj, &metadata)
            }
        );

        match termination_rx.recv_timeout(std::time::Duration::from_millis(cfg.collect_interval)) {
            Ok(_) | Err(RecvTimeoutError::Disconnected) => return Ok(()),
            Err(RecvTimeoutError::Timeout) => () //continue
        }
    }
}
