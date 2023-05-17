use log::debug;
use tokio::{sync::watch, time};
use crate::{Configuration, Telemetry};

mod sql;
mod health;
mod metadata;
mod scheduled_events;
mod cron_triggers;
mod event_triggers;

pub(crate) async fn run_metadata_collector(cfg: &Configuration, metric_obj: &Telemetry, mut termination_rx: watch::Receiver<()>) -> std::io::Result<()> {
    let mut interval = time::interval(time::Duration::from_millis(cfg.collect_interval));

    loop {
        tokio::select! {
            biased;
            _ = termination_rx.changed() => return Ok(()),

            _ = interval.tick() => {
                debug!("Running metadata collector");

                tokio::join!(
                    health::check_health(cfg,metric_obj),
                    scheduled_events::check_scheduled_events(&cfg,metric_obj),
                    cron_triggers::check_cron_triggers(&cfg,metric_obj),
                    async {
                        let metadata = metadata::check_metadata(cfg,metric_obj).await;
                        event_triggers::check_event_triggers(&cfg,metric_obj, &metadata).await;
                    }
                );
            },
        }
    }
}
