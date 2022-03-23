use std::collections::HashMap;

use crate::{Configuration, ERRORS_TOTAL};
use lazy_static::lazy_static;
use log::warn;
use prometheus::{register_int_gauge, IntGauge};
use serde::{Serialize, Deserialize};

lazy_static! {
    static ref METADATA_CONSISTENCY: IntGauge = register_int_gauge!(
        "hasura_metadata_consistency_status",
        "If 1 metadata is consistent, 0 otherwise"
    )
    .unwrap();
}

#[derive(Serialize)]
pub struct MetadataCheckRequest {
    #[serde(rename = "type")]
    pub request_type: String,
    #[serde(rename = "args")]
    pub args: HashMap<String, serde_json::Value>,
}

impl MetadataCheckRequest {
    fn GetInconsistentMetadata() -> Self {
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

pub(crate) async fn check_metadata(cfg: &Configuration) {
    let client = reqwest::Client::new();
    let metadata_check = client
        .post(format!("{}/v1/metadata", cfg.hasura_addr))
        .json(&MetadataCheckRequest::GetInconsistentMetadata())
        .header("x-hasura-admin-secret", cfg.hasura_admin.to_owned())
        .send()
        .await;
    match metadata_check {
        Ok(v) => {
            if v.status() == reqwest::StatusCode::OK {
                let response = v.json::<MetadataCheckResponse>().await;
                match response {
                    Ok(v) => {
                        if v.is_consistent {
                            METADATA_CONSISTENCY.set(1);
                        } else {
                            METADATA_CONSISTENCY.set(0);
                        }
                    },
                    Err(e) => {
                        warn!("Failed to collect metadata check invalid response format: {}", e);
                        ERRORS_TOTAL.with_label_values(&["metadata"]).inc();
                    }
                }
            } else {
                warn!("Failed to collect metadata check invalid status code: {}", v.status());
                ERRORS_TOTAL.with_label_values(&["metadata"]).inc();
            }
        }
        Err(e) => {
            ERRORS_TOTAL.with_label_values(&["metadata"]).inc();
            warn!("Failed to collect metadata check {}", e);
        }
    };
}
