use std::collections::HashMap;

use crate::{Configuration, Telemetry};
use log::warn;
use serde::{Serialize, Deserialize};

#[derive(Serialize)]
pub struct MetadataCheckRequest {
    #[serde(rename = "type")]
    pub request_type: String,
    #[serde(rename = "args")]
    pub args: HashMap<String, serde_json::Value>,
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

async fn fetch_metadata(cfg: &Configuration, metric_obj: &Telemetry) {
    if cfg.disabled_collectors.contains(&crate::Collectors::MetadataInconsistency) {
        return
    }
    let admin_secret = match &cfg.hasura_admin {
        Some(v) => v,
        None => {
            warn!("Metadata should be collected, but admin secret missing!");
            return;
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
}

pub(crate) async fn check_metadata(cfg: &Configuration, metric_obj: &Telemetry) {
    tokio::join!(
        fetch_version(cfg,metric_obj),
        fetch_metadata(cfg,metric_obj)
    );
}
