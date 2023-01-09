use super::sql::*;
use crate::{Configuration, Telemetry};
use log::{warn, info, debug};


fn create_cron_trigger_request() -> SQLRequest {
    SQLRequest {
            request_type: "bulk".to_string(),
            args: vec![
                RunSQLQuery{
                    request_type: "run_sql".to_string(),
                    args: RunSQLArgs {
                        source: "default".to_string(),
                        cascade: false,
                        read_only: true,
                        sql: "SELECT COUNT(*), trigger_name FROM hdb_catalog.hdb_cron_events WHERE status = 'error' GROUP BY trigger_name;".to_string()
                    }
                },
                RunSQLQuery{
                    request_type: "run_sql".to_string(),
                    args: RunSQLArgs {
                        source: "default".to_string(),
                        cascade: false,
                        read_only: true,
                        sql: "SELECT COUNT(*), trigger_name FROM hdb_catalog.hdb_cron_events WHERE status = 'delivered' GROUP BY trigger_name;".to_string()
                    }
                },
                RunSQLQuery{
                    request_type: "run_sql".to_string(),
                    args: RunSQLArgs {
                        source: "default".to_string(),
                        cascade: false,
                        read_only: true,
                        sql: "SELECT COUNT(*), trigger_name FROM hdb_catalog.hdb_cron_events WHERE status = 'scheduled' GROUP BY trigger_name;".to_string()
                    }
                },
                RunSQLQuery{
                    request_type: "run_sql".to_string(),
                    args: RunSQLArgs {
                        source: "default".to_string(),
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
    debug!("Running SQL query for cron triggers");
    let sql_result = make_sql_request(&create_cron_trigger_request(), cfg).await;
    match sql_result {
        Ok(v) => {
            if v.status() == reqwest::StatusCode::OK {
                let response = v.json::<Vec<SQLResult>>().await;
                match response {
                    Ok(v) => {
                        if let Some(failed) = v.get(0) {
                            if failed.result_type == "TuplesOK" {
                                failed.result.as_ref().unwrap().iter().skip(1).for_each(|entry| {
                                    match entry {
                                        SQLResultItem::IntStr(value,trigger_name) => {
                                            metric_obj.CRON_TRIGGER_FAILED.with_label_values(&[trigger_name]).set(*value);
                                        }
                                        SQLResultItem::StrStr(count,trigger_name) => {
                                            let value = match count.trim().parse::<i64>() {
                                                Ok(value) => value,
                                                _ => 0,
                                            };
                                            metric_obj.CRON_TRIGGER_FAILED.with_label_values(&[trigger_name]).set(value);
                                        }
                                        default => {
                                            warn!("Failed to process entry '{:?}', expected two values [ count, trigger_name ]",default);
                                        }
                                    }
                                });
                            } else {
                                info!("Result of SQL query for 'failed cron trigger' has failed or is empty: {:?}", failed);
                            }
                        }
                        if let Some(success) = v.get(1) {
                            if success.result_type == "TuplesOK" {
                                success.result.as_ref().unwrap().iter().skip(1).for_each(|entry| {
                                    match entry {
                                        SQLResultItem::IntStr(value,trigger_name) => {
                                            metric_obj.CRON_TRIGGER_SUCCESSFUL.with_label_values(&[trigger_name]).set(*value);
                                        }
                                        SQLResultItem::StrStr(count,trigger_name) => {
                                            let value = match count.trim().parse::<i64>() {
                                                Ok(value) => value,
                                                _ => 0,
                                            };
                                            metric_obj.CRON_TRIGGER_SUCCESSFUL.with_label_values(&[trigger_name]).set(value);
                                        }
                                        default => {
                                            warn!("Failed to process entry '{:?}', expected two values [ count, trigger_name ]",default);
                                        }
                                    }
                                });
                            } else {
                                info!("Result of SQL query for 'successful cron trigger' has failed or is empty: {:?}", success);
                            }
                        }
                        if let Some(pending) = v.get(2) {
                            if pending.result_type == "TuplesOK" {
                                pending.result.as_ref().unwrap().iter().skip(1).for_each(|entry| {
                                    match entry {
                                        SQLResultItem::IntStr(value,trigger_name) => {
                                            metric_obj.CRON_TRIGGER_PENDING.with_label_values(&[trigger_name]).set(*value);
                                        }
                                        SQLResultItem::StrStr(count,trigger_name) => {
                                            let value = match count.trim().parse::<i64>() {
                                                Ok(value) => value,
                                                _ => 0,
                                            };
                                            metric_obj.CRON_TRIGGER_PENDING.with_label_values(&[trigger_name]).set(value);
                                        }
                                        default => {
                                            warn!("Failed to process entry '{:?}', expected two values [ count, trigger_name ]",default);
                                        }
                                    }
                                });
                            } else {
                                info!("Result of SQL query for 'pending cron trigger' has failed or is empty: {:?}", pending);
                            }
                        }
                        if let Some(processed) = v.get(3) {
                            if processed.result_type == "TuplesOK" {
                                processed.result.as_ref().unwrap().iter().skip(1).for_each(|entry| {
                                    match entry {
                                        SQLResultItem::IntStr(value,trigger_name) => {
                                            metric_obj.CRON_TRIGGER_PROCESSED.with_label_values(&[trigger_name]).set(*value);
                                        }
                                        SQLResultItem::StrStr(count,trigger_name) => {
                                            let value = match count.trim().parse::<i64>() {
                                                Ok(value) => value,
                                                _ => 0,
                                            };
                                            metric_obj.CRON_TRIGGER_PROCESSED.with_label_values(&[trigger_name]).set(value);
                                        }
                                        default => {
                                            warn!("Failed to process entry '{:?}', expected two values [ count, trigger_name ]",default);
                                        }
                                    }
                                });
                            } else {
                                info!("Result of SQL query for 'processed cron trigger' has failed or is empty: {:?}", processed);
                            }
                        }
                    }
                    Err(e) => {
                        warn!( "Failed to collect cron triggers check invalid response format: {}", e );
                        metric_obj.ERRORS_TOTAL.with_label_values(&["cron"]).inc();
                    }
                }
            } else {
                warn!( "Failed to collect cron triggers check invalid status code: {}", v.status() );
                metric_obj.ERRORS_TOTAL.with_label_values(&["cron"]).inc();
            }
        }
        Err(e) => {
            metric_obj.ERRORS_TOTAL.with_label_values(&["cron"]).inc();
            warn!("Failed to collect cron triggers check {}", e);
        }
    };
}
