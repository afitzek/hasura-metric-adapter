use super::sql::*;
use crate::{Configuration, Telemetry};
use log::{warn, info, debug};
use crate::telemetry::MetricOption;


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
                        v.iter().enumerate().for_each(|(index, query)| {

                            let obj = match index as i32 {
                                // Index values must match create_cron_trigger_request() for coherence
                                0 => Ok((MetricOption::IntGaugeVec(&metric_obj.CRON_TRIGGER_FAILED), "failed cron triggers")),
                                1 => Ok((MetricOption::IntGaugeVec(&metric_obj.CRON_TRIGGER_SUCCESSFUL),"successful cron triggers")),
                                2 => Ok((MetricOption::IntGaugeVec(&metric_obj.CRON_TRIGGER_PENDING),"pending cron triggers")),
                                3 => Ok((MetricOption::IntGaugeVec(&metric_obj.CRON_TRIGGER_PROCESSED),"processed cron triggers")),
                                _ => {
                                    warn!("Unexpected entry {:?}",query);
                                    Err(format!("Unexpected entry {:?}",query))
                                }
                            };

                            process_sql_result(query,obj,None);
                        });
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
