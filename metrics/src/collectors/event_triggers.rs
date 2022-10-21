use super::sql::*;
use crate::{Configuration,  Telemetry};
use log::{warn, info};

fn create_event_trigger_request() -> SQLRequest {
    SQLRequest {
            request_type: "bulk".to_string(),
            args: vec![
                RunSQLQuery{
                    request_type: "run_sql".to_string(),
                    args: RunSQLArgs {
                        cascade: false,
                        read_only: true,
                        sql: "SELECT COUNT(*), trigger_name FROM hdb_catalog.event_log WHERE delivered = true OR error = true GROUP BY trigger_name;".to_string()
                    }
                },
                RunSQLQuery{
                    request_type: "run_sql".to_string(),
                    args: RunSQLArgs {
                        cascade: false,
                        read_only: true,
                        sql: "SELECT COUNT(*), trigger_name FROM hdb_catalog.event_log WHERE delivered = false AND error = false AND archived = false GROUP BY trigger_name;".to_string()
                    }
                },
                RunSQLQuery{
                    request_type: "run_sql".to_string(),
                    args: RunSQLArgs {
                        cascade: false,
                        read_only: true,
                        sql: "SELECT COUNT(*), trigger_name FROM hdb_catalog.event_log WHERE error = true GROUP BY trigger_name;".to_string()
                    }
                },
                RunSQLQuery{
                    request_type: "run_sql".to_string(),
                    args: RunSQLArgs {
                        cascade: false,
                        read_only: true,
                        sql: "SELECT COUNT(*), trigger_name FROM hdb_catalog.event_log WHERE error = false AND delivered = true GROUP BY trigger_name;".to_string()
                    }
                },
            ],
        }
}

pub(crate) async fn check_event_triggers(cfg: &Configuration, metric_obj: &Telemetry) {
    if cfg.disabled_collectors.contains(&crate::Collectors::EventTriggers) {
        info!("Not collecting event triggers.");
        return;
    }
    let sql_result = make_sql_request(&create_event_trigger_request(), cfg).await;
    match sql_result {
        Ok(v) => {
            if v.status() == reqwest::StatusCode::OK {
                let response = v.json::<std::vec::Vec<SQLResult>>().await;
                match response {
                    Ok(v) => {
                        if let Some(failed) = v.get(0) {
                            failed.result.iter().skip(1).for_each(|entry| {
                                if let Some((value, Some(label))) = get_sql_entry_value(entry) {
                                    metric_obj.EVENT_TRIGGER_FAILED.with_label_values(&[label.as_str()]).set(value);
                                }
                            });
                        }
                        if let Some(success) = v.get(1) {
                            success.result.iter().skip(1).for_each(|entry| {
                                if let Some((value, Some(label))) = get_sql_entry_value(entry) {
                                    metric_obj.EVENT_TRIGGER_SUCCESSFUL.with_label_values(&[label.as_str()]).set(value);
                                }
                            });
                        }
                        if let Some(pending) = v.get(2) {
                            pending.result.iter().skip(1).for_each(|entry| {
                                if let Some((value, Some(label))) = get_sql_entry_value(entry) {
                                    metric_obj.EVENT_TRIGGER_PENDING.with_label_values(&[label.as_str()]).set(value);
                                }
                            });
                        }
                        if let Some(processed) = v.get(3) {
                            processed.result.iter().skip(1).for_each(|entry| {
                                if let Some((value, Some(label))) = get_sql_entry_value(entry) {
                                    metric_obj.EVENT_TRIGGER_PROCESSED.with_label_values(&[label.as_str()]).set(value);
                                }
                            });
                        }
                    }
                    Err(e) => {
                        warn!(
                            "Failed to collect event triggers check invalid response format: {}",
                            e
                        );
                        metric_obj.ERRORS_TOTAL.with_label_values(&["event"]).inc();
                    }
                }
            } else {
                warn!(
                    "Failed to collect event triggers check invalid status code: {}",
                    v.status()
                );
                metric_obj.ERRORS_TOTAL.with_label_values(&["event"]).inc();
            }
        }
        Err(e) => {
            metric_obj.ERRORS_TOTAL.with_label_values(&["event"]).inc();
            warn!("Failed to collect event triggers check {}", e);
        }
    };
}
