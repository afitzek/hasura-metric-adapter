use super::sql::*;
use crate::{Configuration, Telemetry};
use log::{warn, info};

fn create_scheduled_event_request() -> SQLRequest {
    SQLRequest {
            request_type: "bulk".to_string(),
            args: vec![
                RunSQLQuery{
                    request_type: "run_sql".to_string(),
                    args: RunSQLArgs {
                        cascade: false,
                        read_only: true,
                        sql: "SELECT COUNT(*) FROM hdb_catalog.hdb_scheduled_events WHERE status = 'error';".to_string()
                    }
                },
                RunSQLQuery{
                    request_type: "run_sql".to_string(),
                    args: RunSQLArgs {
                        cascade: false,
                        read_only: true,
                        sql: "SELECT COUNT(*) FROM hdb_catalog.hdb_scheduled_events WHERE status = 'delivered';".to_string()
                    }
                },
                RunSQLQuery{
                    request_type: "run_sql".to_string(),
                    args: RunSQLArgs {
                        cascade: false,
                        read_only: true,
                        sql: "SELECT COUNT(*) FROM hdb_catalog.hdb_scheduled_events WHERE status = 'scheduled';".to_string()
                    }
                },
                RunSQLQuery{
                    request_type: "run_sql".to_string(),
                    args: RunSQLArgs {
                        cascade: false,
                        read_only: true,
                        sql: "SELECT COUNT(*) FROM hdb_catalog.hdb_scheduled_events WHERE status = 'error' or status = 'delivered';".to_string()
                    }
                },
            ],
        }
}

pub(crate) async fn check_scheduled_events(cfg: &Configuration,metric_obj: &Telemetry) {
    if cfg.disabled_collectors.contains(&crate::Collectors::ScheduledEvents) {
        info!("Not collecting scheduled event.");
        return;
    }
    let sql_result = make_sql_request(&create_scheduled_event_request(), cfg).await;
    match sql_result {
        Ok(v) => {
            if v.status() == reqwest::StatusCode::OK {
                let response = v.json::<Vec<SQLResult>>().await;
                match response {
                    Ok(v) => {
                        if let Some(Some((count, _))) = v.get(0).map(get_sql_result_value) {
                            metric_obj.SCHEDULED_EVENTS_FAILED.set(count);
                        }
                        if let Some(Some((count, _))) = v.get(1).map(get_sql_result_value) {
                            metric_obj.SCHEDULED_EVENTS_SUCCESSFUL.set(count);
                        }
                        if let Some(Some((count, _))) = v.get(2).map(get_sql_result_value) {
                            metric_obj.SCHEDULED_EVENTS_PENDING.set(count);
                        }
                        if let Some(Some((count, _))) = v.get(3).map(get_sql_result_value) {
                            metric_obj.SCHEDULED_EVENTS_PROCESSED.set(count);
                        }
                    }
                    Err(e) => {
                        warn!(
                            "Failed to collect scheduled event check invalid response format: {}",
                            e
                        );
                        metric_obj.ERRORS_TOTAL.with_label_values(&["scheduled"]).inc();
                    }
                }
            } else {
                warn!(
                    "Failed to collect scheduled event check invalid status code: {}",
                    v.status()
                );
                metric_obj.ERRORS_TOTAL.with_label_values(&["scheduled"]).inc();
            }
        }
        Err(e) => {
            metric_obj.ERRORS_TOTAL.with_label_values(&["scheduled"]).inc();
            warn!("Failed to collect scheduled event check {}", e);
        }
    };
}
