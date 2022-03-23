use crate::{Configuration, ERRORS_TOTAL};
use lazy_static::lazy_static;
use prometheus::{register_int_gauge, IntGauge};
use log::warn;

lazy_static! {
    static ref HEALTH_CHECK: IntGauge =
        register_int_gauge!("hasura_healthy", "If 1 hasura graphql server is healthy, 0 otherwise").unwrap();
}

pub(crate) async fn check_health(cfg: &Configuration) {
    let health_check = reqwest::get(format!("{}/healthz", cfg.hasura_addr)).await;
    match health_check {
        Ok(v) => {
            if v.status() == reqwest::StatusCode::OK {
                HEALTH_CHECK.set(1);
            } else {
                HEALTH_CHECK.set(0);
            }
        },
        Err(e) => {
            ERRORS_TOTAL.with_label_values(&["health"]).inc();
            warn!("Failed to collect health check {}", e);
        }
    };
}