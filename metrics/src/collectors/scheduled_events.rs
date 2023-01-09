use super::sql::*;
use crate::{Configuration, Telemetry};
use log::{warn, info, debug};
use crate::telemetry::MetricOption;

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
                        v.iter().enumerate().for_each(|(index, query)| {
                            let obj = match index as i32 {
                                // Index values must match create_scheduled_event_request() for coherence
                                0 => Ok((MetricOption::IntGauge(&metric_obj.SCHEDULED_EVENTS_FAILED), "failed scheduled triggers")),
                                1 => Ok((MetricOption::IntGauge(&metric_obj.SCHEDULED_EVENTS_SUCCESSFUL), "successful scheduled triggers")),
                                2 => Ok((MetricOption::IntGauge(&metric_obj.SCHEDULED_EVENTS_PENDING), "pending scheduled triggers")),
                                3 => Ok((MetricOption::IntGauge(&metric_obj.SCHEDULED_EVENTS_PROCESSED), "processed scheduled triggers")),
                                _ => {
                                    warn!("Unexpected entry {:?}",query);
                                    Err(format!("Unexpected entry {:?}",query))
                                }
                            };

                            process_sql_result(query, obj, None);
                        });
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
