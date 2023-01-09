use super::sql::*;
use crate::{Configuration,  Telemetry};
use log::{warn, info, debug};
use serde_json::{Map, Value};
use futures::stream::{self, StreamExt};
use crate::telemetry::MetricOption;

fn create_event_trigger_request(request_type: &String, source: &String) -> SQLRequest {
    SQLRequest {
            request_type: "bulk".to_string(),
            args: vec![
                RunSQLQuery{
                    request_type: request_type.to_string(),
                    args: RunSQLArgs {
                        source: source.to_string(),
                        cascade: false,
                        read_only: true,
                        sql: "SELECT COUNT(*), trigger_name FROM hdb_catalog.event_log WHERE delivered = 'true' OR error = 'true' GROUP BY trigger_name;".to_string()
                    }
                },
                RunSQLQuery{
                    request_type: request_type.to_string(),
                    args: RunSQLArgs {
                        source: source.to_string(),
                        cascade: false,
                        read_only: true,
                        sql: "SELECT COUNT(*), trigger_name FROM hdb_catalog.event_log WHERE delivered = 'false' AND error = 'false' AND archived = 'false' GROUP BY trigger_name;".to_string()
                    }
                },
                RunSQLQuery{
                    request_type: request_type.to_string(),
                    args: RunSQLArgs {
                        source: source.to_string(),
                        cascade: false,
                        read_only: true,
                        sql: "SELECT COUNT(*), trigger_name FROM hdb_catalog.event_log WHERE error = 'true' GROUP BY trigger_name;".to_string()
                    }
                },
                RunSQLQuery{
                    request_type: request_type.to_string(),
                    args: RunSQLArgs {
                        source: source.to_string(),
                        cascade: false,
                        read_only: true,
                        sql: "SELECT COUNT(*), trigger_name FROM hdb_catalog.event_log WHERE error = 'false' AND delivered = 'true' GROUP BY trigger_name;".to_string()
                    }
                },
            ],
        }
}


async fn process_database (data_source: &Map<String, Value>,  cfg: &Configuration, metric_obj: &Telemetry) {
    let sql_type;
    if let Some(kind) = data_source["kind"].as_str() {
        match kind {
            "mssql" => sql_type = "mssql_run_sql",
            "postgres" => sql_type = "run_sql",
            _ => sql_type = ""
        }
    } else {
        sql_type = ""
    }

    if sql_type != "" {
        debug!("Querying data from database {}",data_source["name"]);
        if let Some(db_name) = data_source["name"].as_str() {
            debug!("Request made: {:#?}",serde_json::to_string(&create_event_trigger_request(&sql_type.to_string(), &db_name.to_string())).unwrap());
            let sql_result = make_sql_request(&create_event_trigger_request(&sql_type.to_string(), &db_name.to_string()), cfg).await;
            match sql_result {
                Ok(v) => {
                    if v.status() == reqwest::StatusCode::OK {
                        let response = v.json::<Vec<SQLResult>>().await;
                        debug!("Response: {:?}", response);
                        match response {
                            Ok(v) => {
                                v.iter().enumerate().for_each(|(index, query)| {

                                    let obj = match index as i32 {
                                        // Index values must match create_event_trigger_request() for coherence
                                        0 => Ok((MetricOption::IntGaugeVec(&metric_obj.EVENT_TRIGGER_FAILED), "failed event triggers")),
                                        1 => Ok((MetricOption::IntGaugeVec(&metric_obj.EVENT_TRIGGER_SUCCESSFUL),"successful event triggers")),
                                        2 => Ok((MetricOption::IntGaugeVec(&metric_obj.EVENT_TRIGGER_PENDING),"pending event triggers")),
                                        3 => Ok((MetricOption::IntGaugeVec(&metric_obj.EVENT_TRIGGER_PROCESSED),"processed event triggers")),
                                        _ => {
                                            warn!("Unexpected entry {:?}",query);
                                            Err(format!("Unexpected entry {:?}",query))
                                        }
                                    };

                                    process_sql_result(query, obj,Some(db_name));

                                });
                            }
                            Err(e) => {
                                warn!( "Failed to collect event triggers check invalid response format: {}", e );
                                metric_obj.ERRORS_TOTAL.with_label_values(&["event"]).inc();
                            }
                        }
                    } else {
                        warn!( "Failed to collect event triggers from database {}. Check invalid status code: {}", data_source["name"], v.status() );
                        metric_obj.ERRORS_TOTAL.with_label_values(&["event"]).inc();
                    }
                }
                Err(e) => {
                    metric_obj.ERRORS_TOTAL.with_label_values(&["event"]).inc();
                    warn!("Failed to collect event triggers check {}", e);
                }
            };
        }
    }
}

pub(crate) async fn check_event_triggers(cfg: &Configuration, metric_obj: &Telemetry, metadata: &Map<String, Value>) {
    if cfg.disabled_collectors.contains(&crate::Collectors::EventTriggers) {
        info!("Not collecting event triggers.");
        return;
    }

    debug!("Processing all the databases to look for event triggers");

    let list_tmp = metadata["metadata"]["sources"].as_array();

    match list_tmp {
        Some(list) => {

            let stream = stream::iter(list);
            stream.for_each_concurrent(cfg.concurrency_limit, |data_source| async move {

                debug!("Processing database {} of kind {}",data_source["name"],data_source["kind"]);
                process_database(data_source.as_object().unwrap(), cfg, metric_obj).await;
                debug!("Processed database {} of kind {}",data_source["name"],data_source["kind"]);

            }).await;
        }
        None => {
            metric_obj.ERRORS_TOTAL.with_label_values(&["event"]).inc();
            warn!("Failed to read metadata from responte. It may be inconsistent.");
        }
    }

}
