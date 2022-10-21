use std::collections::HashMap;
use prometheus::{HistogramOpts, HistogramVec, IntCounter, IntCounterVec, IntGauge, IntGaugeVec, Opts};
use prometheus::{register_int_counter_vec, register_int_counter, register_int_gauge, register_int_gauge_vec, register_histogram_vec};

#[allow(non_snake_case)]
#[derive(Clone, Debug)]
pub struct Telemetry {
    pub ERRORS_TOTAL: IntCounterVec,

    pub CRON_TRIGGER_PENDING: IntGaugeVec,
    pub CRON_TRIGGER_PROCESSED: IntGaugeVec,
    pub CRON_TRIGGER_SUCCESSFUL: IntGaugeVec,
    pub CRON_TRIGGER_FAILED: IntGaugeVec,

    pub EVENT_TRIGGER_PENDING: IntGaugeVec,
    pub EVENT_TRIGGER_PROCESSED: IntGaugeVec,
    pub EVENT_TRIGGER_SUCCESSFUL: IntGaugeVec,
    pub EVENT_TRIGGER_FAILED: IntGaugeVec,

    pub HEALTH_CHECK: IntGauge,

    pub METADATA_CONSISTENCY: IntGauge,
    pub METADATA_VERSION: IntGaugeVec,

    pub SCHEDULED_EVENTS_PENDING: IntGauge,
    pub SCHEDULED_EVENTS_PROCESSED: IntGauge,
    pub SCHEDULED_EVENTS_SUCCESSFUL: IntGauge,
    pub SCHEDULED_EVENTS_FAILED: IntGauge,

    pub ACTIVE_WEBSOCKET: IntGauge,
    pub ACTIVE_WEBSOCKET_OPERATIONS: IntGauge,
    pub WEBSOCKET_OPERATIONS: IntCounterVec,

    pub LOG_LINES_COUNTER_TOTAL: IntCounter,
    pub LOG_LINES_COUNTER: IntCounterVec,

    pub REQUEST_COUNTER: IntCounterVec,
    pub REQUEST_QUERY_COUNTER: IntCounterVec,
    pub QUERY_EXECUTION_TIMES: HistogramVec,

}

impl Telemetry {
    pub fn new(common_labels: HashMap<String, String>, histogram_buckets: Vec<f64>) -> Telemetry {

        let errors_total_opts = Opts {
            namespace: String::from(""),
            subsystem: String::from(""),
            name : String::from("hasura_errors_total"),
            help : String::from("The total number of errors per collector"),
            const_labels : common_labels.clone(),
            variable_labels : vec![]
        };


        let cron_trigger_pending_opts = Opts {
            namespace: String::from(""),
            subsystem: String::from(""),
            name : String::from("hasura_pending_cron_triggers"),
            help : String::from("Number of pending hasura cron triggers"),
            const_labels : common_labels.clone(),
            variable_labels : vec![]
        };
        let cron_trigger_processed_opts = Opts {
            namespace: String::from(""),
            subsystem: String::from(""),
            name : String::from("hasura_processed_cron_triggers"),
            help : String::from("Number of processed hasura cron triggers"),
            const_labels : common_labels.clone(),
            variable_labels : vec![]
        };
        let cron_trigger_successful_opts = Opts {
            namespace: String::from(""),
            subsystem: String::from(""),
            name : String::from("hasura_successful_cron_triggers"),
            help : String::from("Number of successfully processed hasura cron triggers"),
            const_labels : common_labels.clone(),
            variable_labels : vec![]
        };
        let cron_trigger_failed_opts = Opts {
            namespace: String::from(""),
            subsystem: String::from(""),
            name : String::from("hasura_failed_cron_triggers"),
            help : String::from("Number of failed hasura cron triggers"),
            const_labels : common_labels.clone(),
            variable_labels : vec![]
        };


        let event_trigger_pending_opts = Opts {
            namespace: String::from(""),
            subsystem: String::from(""),
            name : String::from("hasura_pending_event_triggers"),
            help : String::from("Number of pending hasura event triggers"),
            const_labels : common_labels.clone(),
            variable_labels : vec![]
        };
        let event_trigger_processed_opts = Opts {
            namespace: String::from(""),
            subsystem: String::from(""),
            name : String::from("hasura_processed_event_triggers"),
            help : String::from("Number of processed hasura event triggers"),
            const_labels : common_labels.clone(),
            variable_labels : vec![]
        };
        let event_trigger_successful_opts = Opts {
            namespace: String::from(""),
            subsystem: String::from(""),
            name : String::from("hasura_successful_event_triggers"),
            help : String::from("Number of successfully processed hasura event triggers"),
            const_labels : common_labels.clone(),
            variable_labels : vec![]
        };
        let event_trigger_failed_opts = Opts {
            namespace: String::from(""),
            subsystem: String::from(""),
            name : String::from("hasura_failed_event_triggers"),
            help : String::from("Number of failed hasura event triggers"),
            const_labels : common_labels.clone(),
            variable_labels : vec![]
        };


        let health_check_opts = Opts {
            namespace: String::from(""),
            subsystem: String::from(""),
            name : String::from("hasura_healthy"),
            help : String::from("If 1, Hasura GraphQl server is healthy, 0 otherwise"),
            const_labels : common_labels.clone(),
            variable_labels : vec![]
        };


        let metadata_consistency_opts = Opts {
            namespace: String::from(""),
            subsystem: String::from(""),
            name : String::from("hasura_metadata_consistency_status"),
            help : String::from("If 1, metadata is consistent, 0 otherwise"),
            const_labels : common_labels.clone(),
            variable_labels : vec![]
        };
        let metadata_version_opts = Opts {
            namespace: String::from(""),
            subsystem: String::from(""),
            name : String::from("hasura_metadata_version"),
            help : String::from("If 1, version is active, 0 otherwise"),
            const_labels : common_labels.clone(),
            variable_labels : vec![]
        };


        let scheduled_events_pending_opts = Opts {
            namespace: String::from(""),
            subsystem: String::from(""),
            name : String::from("hasura_pending_one_off_events"),
            help : String::from("Number of pending Hasura one off scheduled events"),
            const_labels : common_labels.clone(),
            variable_labels : vec![]
        };
        let scheduled_events_processed_opts = Opts {
            namespace: String::from(""),
            subsystem: String::from(""),
            name : String::from("hasura_processed_one_off_events"),
            help : String::from("Number of processed Hasura one off scheduled events"),
            const_labels : common_labels.clone(),
            variable_labels : vec![]
        };
        let scheduled_events_successful_opts = Opts {
            namespace: String::from(""),
            subsystem: String::from(""),
            name : String::from("hasura_successful_one_off_events"),
            help : String::from("Number of successful Hasura one off scheduled events"),
            const_labels : common_labels.clone(),
            variable_labels : vec![]
        };
        let scheduled_events_failed_opts = Opts {
            namespace: String::from(""),
            subsystem: String::from(""),
            name : String::from("hasura_failed_one_off_events"),
            help : String::from("Number of failed Hasura one off scheduled events"),
            const_labels : common_labels.clone(),
            variable_labels : vec![]
        };


        let active_websockets_opts = Opts {
            namespace: String::from(""),
            subsystem: String::from(""),
            name : String::from("hasura_websockets_active"),
            help : String::from("Number of Hasura web socket connectios"),
            const_labels : common_labels.clone(),
            variable_labels : vec![]
        };
        let active_websockets_operations_opts = Opts {
            namespace: String::from(""),
            subsystem: String::from(""),
            name : String::from("hasura_websockets_operations_active"),
            help : String::from("Number of Hasura web socket operations like subscriptions"),
            const_labels : common_labels.clone(),
            variable_labels : vec![]
        };
        let websockets_operations_opts = Opts {
            namespace: String::from(""),
            subsystem: String::from(""),
            name : String::from("hasura_websockets_operations"),
            help : String::from("Counts websocket operation by operation name and error code. On success, error is '', otherwise it's the error code. Unnnamed operations are ''"),
            const_labels : common_labels.clone(),
            variable_labels : vec![]
        };


        let log_lines_counter_total_opts = Opts {
            namespace: String::from(""),
            subsystem: String::from(""),
            name : String::from("hasura_log_lines_counter_total"),
            help : String::from("Total number of log lines processed"),
            const_labels : common_labels.clone(),
            variable_labels : vec![]
        };
        let log_lines_counter_opts = Opts {
            namespace: String::from(""),
            subsystem: String::from(""),
            name : String::from("hasura_log_lines_counter"),
            help : String::from("Number of log lines processed"),
            const_labels : common_labels.clone(),
            variable_labels : vec![]
        };


        let request_counter_opts = Opts {
            namespace: String::from(""),
            subsystem: String::from(""),
            name : String::from("hasura_request_counter"),
            help : String::from("Number of http requests. It provides status the http status code and url the path that was called."),
            const_labels : common_labels.clone(),
            variable_labels : vec![]
        };
        let request_query_counter_opts = Opts {
            namespace: String::from(""),
            subsystem: String::from(""),
            name : String::from("hasura_request_query_counter"),
            help : String::from("Number of query requests. On success, error is '', otherwise it's the error code. Unnnamed operations are ''"),
            const_labels : common_labels.clone(),
            variable_labels : vec![]
        };
        let query_execution_seconds_opts = Opts {
            namespace: String::from(""),
            subsystem: String::from(""),
            name : String::from("hasura_query_execution_seconds"),
            help : String::from("Query execution time. On success, error is '', otherwise it's the error code. Unnnamed operations are ''"),
            const_labels : common_labels.clone(),
            variable_labels : vec![]
        };
        let query_execution_seconds_histogram_opts = HistogramOpts {
            common_opts: query_execution_seconds_opts,
            buckets: histogram_buckets
        };


        Telemetry {
            ERRORS_TOTAL : register_int_counter_vec!(errors_total_opts,&["collector"]).unwrap(),

            CRON_TRIGGER_PENDING: register_int_gauge_vec!(cron_trigger_pending_opts,&["trigger_name"]).unwrap(),
            CRON_TRIGGER_PROCESSED: register_int_gauge_vec!(cron_trigger_processed_opts,&["trigger_name"]).unwrap(),
            CRON_TRIGGER_SUCCESSFUL: register_int_gauge_vec!(cron_trigger_successful_opts,&["trigger_name"]).unwrap(),
            CRON_TRIGGER_FAILED: register_int_gauge_vec!(cron_trigger_failed_opts,&["trigger_name"]).unwrap(),

            EVENT_TRIGGER_PENDING: register_int_gauge_vec!(event_trigger_pending_opts,&["trigger_name"]).unwrap(),
            EVENT_TRIGGER_PROCESSED: register_int_gauge_vec!(event_trigger_processed_opts,&["trigger_name"]).unwrap(),
            EVENT_TRIGGER_SUCCESSFUL: register_int_gauge_vec!(event_trigger_successful_opts,&["trigger_name"]).unwrap(),
            EVENT_TRIGGER_FAILED: register_int_gauge_vec!(event_trigger_failed_opts,&["trigger_name"]).unwrap(),

            HEALTH_CHECK: register_int_gauge!(health_check_opts).unwrap(),

            METADATA_CONSISTENCY: register_int_gauge!(metadata_consistency_opts).unwrap(),
            METADATA_VERSION: register_int_gauge_vec!(metadata_version_opts,&["hasura_version"]).unwrap(),

            SCHEDULED_EVENTS_PENDING: register_int_gauge!(scheduled_events_pending_opts).unwrap(),
            SCHEDULED_EVENTS_PROCESSED: register_int_gauge!(scheduled_events_processed_opts).unwrap(),
            SCHEDULED_EVENTS_SUCCESSFUL: register_int_gauge!(scheduled_events_successful_opts).unwrap(),
            SCHEDULED_EVENTS_FAILED: register_int_gauge!(scheduled_events_failed_opts).unwrap(),

            ACTIVE_WEBSOCKET: register_int_gauge!(active_websockets_opts).unwrap(),
            ACTIVE_WEBSOCKET_OPERATIONS: register_int_gauge!(active_websockets_operations_opts).unwrap(),
            WEBSOCKET_OPERATIONS: register_int_counter_vec!(websockets_operations_opts,&["operation", "error"]).unwrap(),

            LOG_LINES_COUNTER_TOTAL: register_int_counter!(log_lines_counter_total_opts).unwrap(),
            LOG_LINES_COUNTER: register_int_counter_vec!(log_lines_counter_opts,&["logtype"]).unwrap(),

            REQUEST_COUNTER: register_int_counter_vec!(request_counter_opts,&["url", "status"]).unwrap(),
            REQUEST_QUERY_COUNTER: register_int_counter_vec!(request_query_counter_opts,&["operation", "error"]).unwrap(),
            QUERY_EXECUTION_TIMES: register_histogram_vec!(query_execution_seconds_histogram_opts,&["operation", "error"]).unwrap()
        }

    }
}