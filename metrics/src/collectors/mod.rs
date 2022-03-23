use crate::{Configuration};

mod health;
mod metadata;

pub(crate) async fn run_metadata_collector(cfg: &Configuration) -> std::io::Result<()> {
    loop {
        tokio::join!(
            health::check_health(cfg),
            metadata::check_metadata(cfg),
        );
        tokio::time::sleep(std::time::Duration::from_millis(cfg.sleep_time)).await;
    }
}