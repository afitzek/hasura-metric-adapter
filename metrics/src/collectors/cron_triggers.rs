use super::sql::*;
use crate::{Configuration, Telemetry};
use log::{warn, info};


fn create_cron_trigger_request() -> SQLRequest {
    SQLRequest {
            request_type: "bulk".to_string(),
            args: vec![
                RunSQLQuery{
                    request_type: "run_sql".to_string(),
                    args: RunSQLArgs {
                        cascade: false,
                        read_only: true,
                        sql: "SELECT COUNT(*), trigger_name FROM hdb_catalog.hdb_cron_events WHERE status = 'error' GROUP BY trigger_name;".to_string()
                    }
                },
                RunSQLQuery{
                    request_type: "run_sql".to_string(),
                    args: RunSQLArgs {
                        cascade: false,
                        read_only: true,
                        sql: "SELECT COUNT(*), trigger_name FROM hdb_catalog.hdb_cron_events WHERE status = 'delivered' GROUP BY trigger_name;".to_string()
                    }
                },
                RunSQLQuery{
                    request_type: "run_sql".to_string(),
                    args: RunSQLArgs {
                        cascade: false,
                        read_only: true,
                        sql: "SELECT COUNT(*), trigger_name FROM hdb_catalog.hdb_cron_events WHERE status = 'scheduled' GROUP BY trigger_name;".to_string()
                    }
                },
                RunSQLQuery{
                    request_type: "run_sql".to_string(),
                    args: RunSQLArgs {
                        cascade: false,
                        read_only: true,
                        sql: "SELECT COUNT(*), trigger_name FROM hdb_catalog.hdb_cron_events WHERE status = 'error' or status = 'delivered' GROUP BY trigger_name;".to_string()
                    }
                },
            ],
        }
}

pub(crate) async fn check_cron_triggers(cfg: &Configuration, metric_obj: &Telemetry) {
    if cfg.disabled_collectors.contains(&crate::Collectors::CronTriggers) {
        info!("Not collecting cron triggers.");
        return;
    }
    let sql_result = make_sql_request(&create_cron_trigger_request(), cfg).await;
    match sql_result {
        Ok(v) => {
            if v.status() == reqwest::StatusCode::OK {
                let response = v.json::<Vec<SQLResult>>().await;
                match response {
                    Ok(v) => {
                        if let Some(failed) = v.get(0) {
                            failed.result.iter().skip(1).for_each(|entry| {
                                if let Some((value, Some(label))) = get_sql_entry_value(entry) {
                                    metric_obj.CRON_TRIGGER_FAILED.with_label_values(&[label.as_str()]).set(value);
                                }
                            });
                        }
                        if let Some(success) = v.get(1) {
                            success.result.iter().skip(1).for_each(|entry| {
                                if let Some((value, Some(label))) = get_sql_entry_value(entry) {
                                    metric_obj.CRON_TRIGGER_SUCCESSFUL.with_label_values(&[label.as_str()]).set(value);
                                }
                            });
                        }
                        if let Some(pending) = v.get(2) {
                            pending.result.iter().skip(1).for_each(|entry| {
                                if let Some((value, Some(label))) = get_sql_entry_value(entry) {
                                    metric_obj.CRON_TRIGGER_PENDING.with_label_values(&[label.as_str()]).set(value);
                                }
                            });
                        }
                        if let Some(processed) = v.get(3) {
                            processed.result.iter().skip(1).for_each(|entry| {
                                if let Some((value, Some(label))) = get_sql_entry_value(entry) {
                                    metric_obj.CRON_TRIGGER_PROCESSED.with_label_values(&[label.as_str()]).set(value);
                                }
                            });
                        }
                    }
                    Err(e) => {
                        warn!(
                            "Failed to collect cron triggers check invalid response format: {}",
                            e
                        );
                        metric_obj.ERRORS_TOTAL.with_label_values(&["cron"]).inc();
                    }
                }
            } else {
                warn!(
                    "Failed to collect cron triggers check invalid status code: {}",
                    v.status()
                );
                metric_obj.ERRORS_TOTAL.with_label_values(&["cron"]).inc();
            }
        }
        Err(e) => {
            metric_obj.ERRORS_TOTAL.with_label_values(&["cron"]).inc();
            warn!("Failed to collect cron triggers check {}", e);
        }
    };
}
