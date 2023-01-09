use super::sql::*;
use crate::{Configuration, Telemetry};
use log::{warn, info, debug};

fn create_scheduled_event_request() -> SQLRequest {
    SQLRequest {
            request_type: "bulk".to_string(),
            args: vec![
                RunSQLQuery{
                    request_type: "run_sql".to_string(),
                    args: RunSQLArgs {
                        source: "default".to_string(),
                        cascade: false,
                        read_only: true,
                        sql: "SELECT COUNT(*) FROM hdb_catalog.hdb_scheduled_events WHERE status = 'error';".to_string()
                    }
                },
                RunSQLQuery{
                    request_type: "run_sql".to_string(),
                    args: RunSQLArgs {
                        source: "default".to_string(),
                        cascade: false,
                        read_only: true,
                        sql: "SELECT COUNT(*) FROM hdb_catalog.hdb_scheduled_events WHERE status = 'delivered';".to_string()
                    }
                },
                RunSQLQuery{
                    request_type: "run_sql".to_string(),
                    args: RunSQLArgs {
                        source: "default".to_string(),
                        cascade: false,
                        read_only: true,
                        sql: "SELECT COUNT(*) FROM hdb_catalog.hdb_scheduled_events WHERE status = 'scheduled';".to_string()
                    }
                },
                RunSQLQuery{
                    request_type: "run_sql".to_string(),
                    args: RunSQLArgs {
                        source: "default".to_string(),
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
    debug!("Running SQL query for scheduled events");
    let sql_result = make_sql_request(&create_scheduled_event_request(), cfg).await;
    match sql_result {
        Ok(v) => {
            if v.status() == reqwest::StatusCode::OK {
                let response = v.json::<Vec<SQLResult>>().await;
                match response {
                    Ok(v) => {
                        if let Some(failed) = v.get(0) {
                            if failed.result_type == "TuplesOK" {
                                if failed.result.as_ref().unwrap().len() == 1 {
                                    let entry = &failed.result.as_ref().unwrap()[0];
                                    match entry {
                                        SQLResultItem::Str(vect) => {
                                            let parsed_count;
                                            if vect.len() == 1 {
                                                parsed_count = match vect[0].trim().parse::<i64>() {
                                                    Ok(value) => value,
                                                    Err(_) => 0,
                                                };
                                            } else {
                                                warn!("Failed to process entry '{:?}', expected one value [ count ]",vect);
                                                parsed_count = 0;
                                            }
                                            metric_obj.SCHEDULED_EVENTS_FAILED.set(parsed_count);
                                        }
                                        SQLResultItem::Int(vect) => {
                                            let count;
                                            if vect.len() == 1 {
                                                count = vect[0];
                                            } else {
                                                count = 0;
                                                warn!("Failed to process entry '{:?}', expected one value [ count ]",vect);
                                            }
                                            metric_obj.SCHEDULED_EVENTS_FAILED.set(count);
                                        }
                                        default => {
                                            metric_obj.SCHEDULED_EVENTS_FAILED.set(0);
                                            warn!("Failed to process entry '{:?}', expected one value [ count ]",default);
                                        }
                                    }
                                } else {
                                    warn!("Failed to process 'failed scheduled triggers' because the sql query response has incorrect format: '{:?}'",failed);
                                }
                            } else {
                                info!("Result of SQL query for 'failed scheduled trigger' has failed or is empty: {:?}", failed);
                            }
                        }
                        if let Some(success) = v.get(1) {
                            if success.result_type == "TuplesOK" {
                                if success.result.as_ref().unwrap().len() == 1 {
                                    let entry = &success.result.as_ref().unwrap()[0];
                                    match entry {
                                        SQLResultItem::Str(vect) => {
                                            let parsed_count;
                                            if vect.len() == 1 {
                                                parsed_count = match vect[0].trim().parse::<i64>() {
                                                    Ok(value) => value,
                                                    Err(_) => 0,
                                                };
                                            } else {
                                                warn!("Failed to process entry '{:?}', expected one value [ count ]",vect);
                                                parsed_count = 0;
                                            }
                                            metric_obj.SCHEDULED_EVENTS_SUCCESSFUL.set(parsed_count);
                                        }
                                        SQLResultItem::Int(vect) => {
                                            let count;
                                            if vect.len() == 1 {
                                                count = vect[0];
                                            } else {
                                                count = 0;
                                                warn!("Failed to process entry '{:?}', expected one value [ count ]",vect);
                                            }
                                            metric_obj.SCHEDULED_EVENTS_SUCCESSFUL.set(count);
                                        }
                                        default => {
                                            metric_obj.SCHEDULED_EVENTS_SUCCESSFUL.set(0);
                                            warn!("Failed to process entry '{:?}', expected one value [ count ]",default);
                                        }
                                    }
                                } else {
                                    warn!("Failed to process 'successful scheduled triggers' because the sql query response has incorrect format: '{:?}'",success);
                                }
                            } else {
                                info!("Result of SQL query for 'successful scheduled trigger' has failed or is empty: {:?}", success);
                            }
                        }
                        if let Some(pending) = v.get(2) {
                            if pending.result_type == "TuplesOK" {
                                if pending.result.as_ref().unwrap().len() == 1 {
                                    let entry = &pending.result.as_ref().unwrap()[0];
                                    match entry {
                                        SQLResultItem::Str(vect) => {
                                            let parsed_count;
                                            if vect.len() == 1 {
                                                parsed_count = match vect[0].trim().parse::<i64>() {
                                                    Ok(value) => value,
                                                    Err(_) => 0,
                                                };
                                            } else {
                                                warn!("Failed to process entry '{:?}', expected one value [ count ]",vect);
                                                parsed_count = 0;
                                            }
                                            metric_obj.SCHEDULED_EVENTS_PENDING.set(parsed_count);
                                        }
                                        SQLResultItem::Int(vect) => {
                                            let count;
                                            if vect.len() == 1 {
                                                count = vect[0];
                                            } else {
                                                count = 0;
                                                warn!("Failed to process entry '{:?}', expected one value [ count ]",vect);
                                            }
                                            metric_obj.SCHEDULED_EVENTS_PENDING.set(count);
                                        }
                                        default => {
                                            metric_obj.SCHEDULED_EVENTS_PENDING.set(0);
                                            warn!("Failed to process entry '{:?}', expected one value [ count ]",default);
                                        }
                                    }
                                } else {
                                    warn!("Failed to process 'pending scheduled triggers' because the sql query response has incorrect format: '{:?}'",pending);
                                }
                            } else {
                                info!("Result of SQL query for 'pending scheduled trigger' has failed or is empty: {:?}", pending);
                            }
                        }
                        if let Some(processed) = v.get(3) {
                            if processed.result_type == "TuplesOK" {
                                if processed.result.as_ref().unwrap().len() == 1 {
                                    let entry = &processed.result.as_ref().unwrap()[0];
                                    match entry {
                                        SQLResultItem::Str(vect) => {
                                            let parsed_count;
                                            if vect.len() == 1 {
                                                parsed_count = match vect[0].trim().parse::<i64>() {
                                                    Ok(value) => value,
                                                    Err(_) => 0,
                                                };
                                            } else {
                                                warn!("Failed to process entry '{:?}', expected one value [ count ]",vect);
                                                parsed_count = 0;
                                            }
                                            metric_obj.SCHEDULED_EVENTS_PROCESSED.set(parsed_count);
                                        }
                                        SQLResultItem::Int(vect) => {
                                            let count;
                                            if vect.len() == 1 {
                                                count = vect[0];
                                            } else {
                                                count = 0;
                                                warn!("Failed to process entry '{:?}', expected one value [ count ]",vect);
                                            }
                                            metric_obj.SCHEDULED_EVENTS_PROCESSED.set(count);
                                        }
                                        default => {
                                            metric_obj.SCHEDULED_EVENTS_PROCESSED.set(0);
                                            warn!("Failed to process entry '{:?}', expected one value [ count ]",default);
                                        }
                                    }
                                } else {
                                    warn!("Failed to process 'successful scheduled triggers' because the sql query response has incorrect format: '{:?}'",processed);
                                }
                            } else {
                                info!("Result of SQL query for 'processed scheduled trigger' has failed or is empty: {:?}",processed);
                            }
                        }

                    }
                    Err(e) => {
                        warn!( "Failed to collect scheduled event check invalid response format: {}", e );
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
