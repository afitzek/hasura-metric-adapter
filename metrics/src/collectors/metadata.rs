use std::collections::HashMap;

use crate::{Configuration, Telemetry};
use log::{warn,debug};
use serde::{Serialize, Deserialize};
use serde_json::{json, Map, Value};

#[derive(Serialize)]
pub struct MetadataCheckRequest {
    #[serde(rename = "type")]
    pub request_type: String,
    #[serde(rename = "args")]
    pub args: HashMap<String, Value>,
}

impl MetadataCheckRequest {
    fn get_inconsistent_metadata() -> Self {
        MetadataCheckRequest {
            request_type: "get_inconsistent_metadata".to_string(),
            args: HashMap::new(),
        }
    }
}

#[derive(Deserialize)]
pub struct MetadataCheckResponse {
    #[serde(rename = "is_consistent")]
    pub is_consistent: bool
}

#[derive(Serialize)]
pub struct MetadataExportRequest {
    #[serde(rename = "type")]
    pub request_type: String,
    #[serde(rename = "version")]
    pub version: i32,
    #[serde(rename = "args")]
    pub args: HashMap<String, Value>,
}

impl MetadataExportRequest {
    fn export_metadata() -> Self {
        MetadataExportRequest {
            request_type: "export_metadata".to_string(),
            version: 2,
            args: HashMap::new(),
        }
    }
}

#[derive(Deserialize)]
pub struct VersionResponse {
    #[serde(rename = "version")]
    pub version: String
}

async fn fetch_version(cfg: &Configuration, metric_obj: &Telemetry) {
    let client = reqwest::Client::new();
    let version_check = client
        .get(format!("{}/v1/version", cfg.hasura_addr))
        .send()
        .await;
    match version_check {
        Ok(v) => {
            if v.status() == reqwest::StatusCode::OK {
                let response = v.json::<VersionResponse>().await;
                match response {
                    Ok(v) => {
                        metric_obj.METADATA_VERSION.reset();
                        metric_obj.METADATA_VERSION.with_label_values(&[v.version.as_str()]).set(1);
                    },
                    Err(e) => {
                        warn!("Failed to collect version information invalid response format: {}", e);
                        metric_obj.ERRORS_TOTAL.with_label_values(&["version"]).inc();
                    }
                }
            } else {
                warn!("Failed to collect version information invalid status code: {}", v.status());
                metric_obj.ERRORS_TOTAL.with_label_values(&["version"]).inc();
            }
        }
        Err(e) => {
            metric_obj.ERRORS_TOTAL.with_label_values(&["version"]).inc();
            warn!("Failed to collect version information {}", e);
        }
    };
}

async fn fetch_metadata_consistency(cfg: &Configuration, metric_obj: &Telemetry) -> bool {
    let mut consistency = false;
    if cfg.disabled_collectors.contains(&crate::Collectors::MetadataInconsistency) {
        return consistency;
    }
    let admin_secret = match &cfg.hasura_admin {
        Some(v) => v,
        None => {
            warn!("Metadata should be collected, but admin secret missing!");
            return consistency;
        }
    };

    let client = reqwest::Client::new();
    let metadata_check = client
        .post(format!("{}/v1/metadata", cfg.hasura_addr))
        .json(&MetadataCheckRequest::get_inconsistent_metadata())
        .header("x-hasura-admin-secret", admin_secret)
        .send()
        .await;
    
    match metadata_check {
        Ok(v) => {
            if v.status() == reqwest::StatusCode::OK {
                let response = v.json::<MetadataCheckResponse>().await;
                match response {
                    Ok(v) => {
                        if v.is_consistent {
                            metric_obj.METADATA_CONSISTENCY.set(1);
                            consistency = true;
                        } else {
                            metric_obj.METADATA_CONSISTENCY.set(0);
                        }
                    },
                    Err(e) => {
                        warn!("Failed to collect metadata check invalid response format: {}", e);
                        metric_obj.ERRORS_TOTAL.with_label_values(&["metadata"]).inc();
                    }
                }
            } else {
                warn!("Failed to collect metadata check invalid status code: {}", v.status());
                metric_obj.ERRORS_TOTAL.with_label_values(&["metadata"]).inc();
            }
        }
        Err(e) => {
            metric_obj.ERRORS_TOTAL.with_label_values(&["metadata"]).inc();
            warn!("Failed to collect metadata check {}", e);
        }
    };

    return consistency;
}


async fn fetch_metadata(cfg: &Configuration, metric_obj: &Telemetry) -> Map<String, Value> {
    
    let mut metadata = json!({}).as_object().unwrap().clone();
    
    if cfg.disabled_collectors.contains(&crate::Collectors::EventTriggers) {
        return metadata;
    }
    
    let admin_secret = match &cfg.hasura_admin {
        Some(v) => v,
        None => {
            warn!("Metadata should be collected, but admin secret missing!");
            return metadata;
        }
    };
    let client = reqwest::Client::new();
    let metadata_export = client
        .post(format!("{}/v1/metadata", cfg.hasura_addr))
        .json(&MetadataExportRequest::export_metadata())
        .header("x-hasura-admin-secret", admin_secret)
        .send()
        .await;

    match metadata_export {
        Ok(v) => {
            if v.status() == reqwest::StatusCode::OK {
                let response = v.json::<Map<String, Value>>().await;
                match response {
                    Ok(v) => {
                        metadata = v.clone();
                    },
                    Err(e) => {
                        warn!("Failed to fetch metadata. Invalid response format: {}", e);
                        metric_obj.ERRORS_TOTAL.with_label_values(&["metadata"]).inc();
                    }
                }
            } else {
                warn!("Failed to collect metadata check invalid status code: {}", v.status());
                metric_obj.ERRORS_TOTAL.with_label_values(&["metadata"]).inc();
            }
        }
        Err(e) => {
            metric_obj.ERRORS_TOTAL.with_label_values(&["metadata"]).inc();
            warn!("Failed to collect metadata check {}", e);
        }
    };
    return metadata;
}

async fn dummy() {
    return;
}
pub(crate) async fn check_metadata(cfg: &Configuration, metric_obj: &Telemetry) -> Map<String, Value> {
    let consistent: bool;
    let metadata;

    tokio::join!(
        fetch_version(cfg, metric_obj),
        {
            consistent = fetch_metadata_consistency(cfg, metric_obj).await;

            if consistent {
                debug!("Metadata is consistent");
                metadata = fetch_metadata(cfg, metric_obj).await
            } else {
                warn!("Failed to collect metadata because it is inconsistent");
                metric_obj.ERRORS_TOTAL.with_label_values(&["metadata"]).inc();
                metadata = json!({}).as_object().unwrap().clone()
            }
            dummy()
        }
    );

    return metadata;
}
