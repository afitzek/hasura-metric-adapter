use crate::{Configuration, Telemetry};
use log::{debug, warn};

pub(crate) async fn check_health(cfg: &Configuration, metric_obj: &Telemetry) {
    let health_check = reqwest::get(format!("{}/healthz", cfg.hasura_addr)).await;
    match health_check {
        Ok(v) => {
            if v.status() == reqwest::StatusCode::OK {
                debug!("Healthcheck OK");
                metric_obj.HEALTH_CHECK.set(1);
            } else {
                debug!("Healthcheck NOK");
                metric_obj.HEALTH_CHECK.set(0);
            }
        },
        Err(e) => {
            metric_obj.HEALTH_CHECK.set(0);
            metric_obj.ERRORS_TOTAL.with_label_values(&["health"]).inc();
            warn!("Failed to collect health check {}", e);
        }
    };
}
