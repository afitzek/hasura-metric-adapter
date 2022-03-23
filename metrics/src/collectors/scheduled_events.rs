use super::sql::*;
use crate::{Configuration, ERRORS_TOTAL};
use lazy_static::lazy_static;
use log::warn;
use prometheus::{register_int_gauge, IntGauge};

lazy_static! {
    static ref SCHEDULED_EVENTS_PENDING: IntGauge = register_int_gauge!(
        "hasura_pending_one_off_events",
        "number of pending hasura one off scheduled events"
    )
    .unwrap();
    static ref SCHEDULED_EVENTS_PROCESSED: IntGauge = register_int_gauge!(
        "hasura_processed_one_off_events",
        "number of processed hasura one off scheduled events"
    )
    .unwrap();
    static ref SCHEDULED_EVENTS_SUCCESSFUL: IntGauge = register_int_gauge!(
        "hasura_successful_one_off_events",
        "number of successful hasura one off scheduled events"
    )
    .unwrap();
    static ref SCHEDULED_EVENTS_FAILED: IntGauge = register_int_gauge!(
        "hasura_failed_one_off_events",
        "number of failed hasura one off scheduled events"
    )
    .unwrap();
}

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
                        sql: "SELECT COUNT(*) FROM hdb_catalog.hdb_cron_events WHERE status = 'error' or status = 'delivered';".to_string()
                    }
                },
            ],
        }
}

pub(crate) async fn check_scheduled_events(cfg: &Configuration) {
    let sql_result = make_sql_request(&create_scheduled_event_request(), cfg).await;
    match sql_result {
        Ok(v) => {
            if v.status() == reqwest::StatusCode::OK {
                let response = v.json::<std::vec::Vec<SQLResult>>().await;
                match response {
                    Ok(v) => {
                        if let Some(Some((count, _))) = v.get(0).map(get_sql_result_value) {
                            SCHEDULED_EVENTS_FAILED.set(count);
                        }
                        if let Some(Some((count, _))) = v.get(1).map(get_sql_result_value) {
                            SCHEDULED_EVENTS_SUCCESSFUL.set(count);
                        }
                        if let Some(Some((count, _))) = v.get(2).map(get_sql_result_value) {
                            SCHEDULED_EVENTS_PENDING.set(count);
                        }
                        if let Some(Some((count, _))) = v.get(3).map(get_sql_result_value) {
                            SCHEDULED_EVENTS_PROCESSED.set(count);
                        }
                    }
                    Err(e) => {
                        warn!(
                            "Failed to collect scheduled event check invalid response format: {}",
                            e
                        );
                        ERRORS_TOTAL.with_label_values(&["scheduled"]).inc();
                    }
                }
            } else {
                warn!(
                    "Failed to collect scheduled event check invalid status code: {}",
                    v.status()
                );
                ERRORS_TOTAL.with_label_values(&["scheduled"]).inc();
            }
        }
        Err(e) => {
            ERRORS_TOTAL.with_label_values(&["scheduled"]).inc();
            warn!("Failed to collect scheduled event check {}", e);
        }
    };
}
