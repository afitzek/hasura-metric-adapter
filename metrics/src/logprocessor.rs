use lazy_static::lazy_static;
use prometheus::{register_histogram_vec, HistogramVec};
use prometheus::{register_int_counter, IntCounter};
use prometheus::{register_int_counter_vec, IntCounterVec};
use prometheus::{register_int_gauge, IntGauge};

use log::warn;

use serde::Deserialize;
use serde_json::{from_str, from_value};

lazy_static! {
    static ref ACTIVE_WEBSOCKETS: IntGauge =
        register_int_gauge!("hasura_websockets_active", "Active web socket connections").unwrap();
    static ref ACTIVE_WEBSOCKET_OPERATIONS: IntGauge = register_int_gauge!(
        "hasura_websockets_operations_active",
        "Active web socket operations like subscriptions"
    )
    .unwrap();
    static ref WEBSOCKET_OPERATIONS: IntCounterVec = register_int_counter_vec!(
        "hasura_websockets_operations",
        "Counts websocket operation by operation name and error code (on success error is '' other its the error code) (unnnamed operations are '')",
        &["operation", "error"]
    )
    .unwrap();
    static ref LOG_LINES_COUNTER_TOTAL: IntCounter = register_int_counter!(
        "hasura_log_lines_counter_total",
        "Total Number of log lines processed"
    )
    .unwrap();
    static ref LOG_LINES_COUNTER: IntCounterVec = register_int_counter_vec!(
        "hasura_log_lines_counter",
        "Number of log lines processed",
        &["logtype"]
    )
    .unwrap();
    static ref REQUEST_COUNTER: IntCounterVec = register_int_counter_vec!(
        "hasura_request_counter",
        "Number requests",
        &["url", "status"]
    )
    .unwrap();
    static ref REQUEST_QUERY_COUNTER: IntCounterVec = register_int_counter_vec!(
        "hasura_request_query_counter",
        "Number query requests (on success error is '' other its the error code) (unnnamed operations are '')",
        &["operation", "error"]
    )
    .unwrap();
    static ref QUERY_EXECUTION_TIMES: HistogramVec = register_histogram_vec!(
        "hasura_query_execution_seconds",
        "Query execution Times (on success error is '' other its the error code) (unnnamed operations are '')",
        &["operation", "error"]
    )
    .expect("Failed to create ");
}

#[derive(Deserialize)]
pub struct BaseLog {
    #[serde(rename = "timestamp")]
    pub timestamp: String,
    #[serde(rename = "level")]
    pub level: String,
    #[serde(rename = "type")]
    pub logtype: String,
    #[serde(rename = "detail")]
    pub detail: serde_json::Value,
}

#[derive(Deserialize)]
pub struct HttpLogDetailHttpInfo {
    #[serde(rename = "status")]
    pub status: i32,
    #[serde(rename = "http_version")]
    pub http_version: String,
    #[serde(rename = "url")]
    pub url: String,
    #[serde(rename = "method")]
    pub method: String,
    #[serde(rename = "ip")]
    pub ip: String,
}

#[derive(Deserialize)]
pub struct HttpLogDetailOperationError {
    #[serde(rename = "path")]
    pub path: String,
    #[serde(rename = "error")]
    pub error: String,
    #[serde(rename = "code")]
    pub code: String,
}

#[derive(Deserialize)]
pub struct HttpLogDetailOperationQuery {
    #[serde(rename = "operationName")]
    pub operation_name: Option<String>,
    #[serde(rename = "query")]
    pub query: Option<String>,
}

#[derive(Deserialize)]
pub struct HttpLogDetailOperation {
    #[serde(rename = "query_execution_time")]
    pub query_execution_time: Option<f64>,
    #[serde(rename = "request_id")]
    pub request_id: String,
    #[serde(rename = "parameterized_query_hash")]
    pub parameterized_query_hash: Option<String>,
    #[serde(rename = "response_size")]
    pub response_size: i32,
    #[serde(rename = "error")]
    pub error: Option<HttpLogDetailOperationError>,
    #[serde(rename = "query")]
    pub query: Option<HttpLogDetailOperationQuery>,
}

#[derive(Deserialize)]
pub struct HttpLogDetails {
    #[serde(rename = "request_id")]
    pub request_id: String,
    #[serde(rename = "operation")]
    pub operation: HttpLogDetailOperation,
    pub http_info: HttpLogDetailHttpInfo,
}

async fn handle_http_log(log: &BaseLog) {
    let detail_result = from_value::<HttpLogDetails>(log.detail.clone());
    match detail_result {
        Ok(http) => {
            REQUEST_COUNTER
                .with_label_values(&[
                    http.http_info.url.as_str(),
                    format!("{}", http.http_info.status).as_str(),
                ])
                .inc();

            if let Some(query) = http.operation.query {
                let error = http.operation.error.map_or("".to_string(), |v| v.code);

                let operation = query.operation_name.unwrap_or("".to_string());
                REQUEST_QUERY_COUNTER
                    .with_label_values(&[operation.as_str(), error.as_str()])
                    .inc();

                if let Some(exec_time) = http.operation.query_execution_time {
                    QUERY_EXECUTION_TIMES
                        .with_label_values(&[operation.as_str(), error.as_str()])
                        .observe(exec_time);
                }
            }
        }
        Err(e) => {
            eprintln!("Invalid HTTP log detail: {}", e);
        }
    };
}

#[derive(Deserialize)]
pub struct WebSocketDetailEvent {
    #[serde(rename = "type")]
    pub event_type: String,

    #[serde(rename = "detail")]
    pub detail: Option<WebSocketDetailEventDetail>,
}

#[derive(Deserialize)]
pub struct WebSocketDetailEventDetailOperationType {
    #[serde(rename = "type")]
    pub operation_type: String,
    #[serde(rename = "detail")]
    pub detail: Option<HttpLogDetailOperationError>,
}

#[derive(Deserialize)]
pub struct WebSocketDetailEventDetail {
    #[serde(rename = "operation_name")]
    pub operation_name: Option<String>,
    #[serde(rename = "request_id")]
    pub request_id: Option<String>,
    #[serde(rename = "operation_type")]
    pub operation_type: WebSocketDetailEventDetailOperationType,
}

#[derive(Deserialize)]
pub struct WebSocketDetailConnInfo {}

#[derive(Deserialize)]
pub struct WebSocketDetail {
    #[serde(rename = "event")]
    pub event: WebSocketDetailEvent,
    #[serde(rename = "connection_info")]
    pub connection_info: WebSocketDetailConnInfo,
}

async fn handle_websocket_log(log: &BaseLog) {
    let detail_result = from_value::<WebSocketDetail>(log.detail.clone());
    match detail_result {
        Ok(http) => {
            match &http.event.event_type as &str {
                "accepted" => ACTIVE_WEBSOCKETS.inc(),
                "closed" => ACTIVE_WEBSOCKETS.dec(),
                "operation" => {
                    if let Some(detail) = http.event.detail {
                        let op_name = detail.operation_name.unwrap_or("".to_string());
                        match &detail.operation_type.operation_type as &str {
                            "started" => ACTIVE_WEBSOCKET_OPERATIONS.inc(),
                            "stopped" => {
                                WEBSOCKET_OPERATIONS
                                    .with_label_values(&[op_name.as_str(), ""])
                                    .inc();
                                ACTIVE_WEBSOCKET_OPERATIONS.dec()
                            }
                            "query_err" => {
                                let err = detail
                                    .operation_type
                                    .detail
                                    .map_or("".to_string(), |v| v.code);
                                WEBSOCKET_OPERATIONS
                                    .with_label_values(&[op_name.as_str(), err.as_str()])
                                    .inc();
                            }
                            _ => (),
                        };
                    }
                }
                _ => (),
            };
        }
        Err(e) => {
            warn!("Invalid Websocket log detail: {}", e);
        }
    };
}

pub async fn log_processor(logline: &String) {
    //println!("{}", logline);
    LOG_LINES_COUNTER_TOTAL.inc();
    let log_result = from_str::<BaseLog>(logline);
    match log_result {
        Ok(log) => {
            LOG_LINES_COUNTER
                .with_label_values(&[log.logtype.as_str()])
                .inc();
            match &log.logtype as &str {
                "http-log" => {
                    handle_http_log(&log).await;
                }
                "websocket-log" => {
                    handle_websocket_log(&log).await;
                }
                _ => {}
            };
        }
        Err(e) => {
            warn!("Failed to parse log line: {}", e);
        }
    };
}
