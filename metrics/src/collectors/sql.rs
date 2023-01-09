use log::{info};
use reqwest::Response;
use serde::{Deserialize, Serialize};
use snafu::{prelude::*, Whatever};
use crate::telemetry::MetricOption;

#[derive(Serialize, Debug)]
pub struct SQLRequest {
    #[serde(rename = "type")]
    pub request_type: String,
    #[serde(rename = "args")]
    pub args: Vec<RunSQLQuery>,
}

#[derive(Serialize, Debug)]
pub struct RunSQLQuery {
    #[serde(rename = "type")]
    pub request_type: String,
    #[serde(rename = "args")]
    pub args: RunSQLArgs,
}

#[derive(Serialize, Debug)]
pub struct RunSQLArgs {
    #[serde(rename = "cascade")]
    pub cascade: bool,
    #[serde(rename = "read_only")]
    pub read_only: bool,
    #[serde(rename = "sql")]
    pub sql: String,
    #[serde(rename = "source")]
    pub source: String,
}

#[derive(Deserialize,Debug)]
pub struct SQLResult {
    // #[serde(rename = "result_type")]
    pub result_type: String,
    // #[serde(rename = "result")]
    pub result: Option<Vec<SQLResultItem>>,
}

#[derive(Deserialize,Debug)]
#[serde(untagged)]
pub enum SQLResultItem {
    IntStr(i64,String),
    StrStr(String,String),
    Str(Vec<String>),
    Int(Vec<i64>)
}

pub(crate) async fn make_sql_request(request: &SQLRequest, cfg: &crate::Configuration) -> Result<Response, Whatever> {
    let admin_secret = match &cfg.hasura_admin {
        Some(v) => Ok(v),
        None => {
            whatever!("Metadata should be collected, but admin secret missing!")
        }
    }?;
    let client = reqwest::Client::new();
    match client
        .post(format!("{}/v2/query", cfg.hasura_addr))
        .json(request)
        .header("x-hasura-admin-secret", admin_secret)
        .send()
        .await {
            Ok(v) => Ok(v),
            Err(e) => whatever!("Failed to run SQL request against hasura: {}", e)
    }
}

pub(crate) fn get_sql_entry_value(entry: &SQLResultItem ) -> SQLResultItem {
    match entry {
        SQLResultItem::IntStr(value,trigger_name) => {
            SQLResultItem::IntStr(*value, trigger_name.to_string())
        }
        SQLResultItem::StrStr(count,trigger_name) => {
            let value = match count.trim().parse::<i64>() {
                Ok(value) => value,
                _ => 0,
            };
            SQLResultItem::IntStr(value,trigger_name.to_string())
        }
        SQLResultItem::Str(vect) => {
            let parsed_count;
            if vect.len() == 1 {
                parsed_count = match vect[0].trim().parse::<i64>() {
                    Ok(value) => value,
                    Err(_) => 0,
                };
            } else {
                print!("Expected one value in array '{:?}'",vect);
                parsed_count = 0;
            }

            SQLResultItem::IntStr(parsed_count,"".to_string())
        }
        SQLResultItem::Int(vect) => {
            let count;
            if vect.len() == 1 {
                count = vect[0];
            } else {
                count = 0;
                print!("Expected one value in array '{:?}'",vect);
            }
            SQLResultItem::IntStr(count,"".to_string())
        }
        // default => {
        //     warn!("Failed to process entry '{:?}', expected either two values [ count, trigger_name ] or one value [ count ]",default);
        //     SQLResultItem::IntStr(0,"".to_string())
        // }
    }
}

pub(crate) fn process_sql_result<T>(query: &SQLResult, obj: Result<(MetricOption,&str),T>, db_name_opt: Option<&str>) {
    if let Ok((metric, metric_name)) = obj {
        if query.result_type == String::from("TuplesOk") {
            query.result.as_ref().unwrap().iter().skip(1).for_each(|entry| {
                let (value, trigger_name) = if let SQLResultItem::IntStr(value, trigger_name) = get_sql_entry_value(entry) {
                    (value, trigger_name)
                } else {
                    (0,"".to_string())
                };

                match metric {
                    MetricOption::IntGaugeVec(metric) => {
                        if let Some(db_name) = db_name_opt {
                            metric.with_label_values( & [trigger_name.as_str(), db_name]).set(value);
                        } else {
                            metric.with_label_values( & [trigger_name.as_str()]).set(value);
                        }
                    }
                    MetricOption::IntGauge(metric) => {
                        metric.set(value)
                    }
                }
            });
        } else {
            if let Some(db_name) = db_name_opt {
                info!("Result of SQL query for '{}' on database {} has failed or is empty: {:?}",metric_name,db_name.to_string(),query);
            } else {
                info!("Result of SQL query for '{}' has failed or is empty: {:?}",metric_name,query);
            }
        }
    }
}
// pub(crate) fn get_sql_entry_value(entry: &Vec<String>) -> Option<(i64, Option<String>)> {
//     if entry.len() >= 1 && entry.len() <= 2 {
//         let str_value = entry.get(0).unwrap();
//         if let Some(value) = str_value.parse::<i64>().ok() {
//             return Some((value, entry.get(1).map(|v| v.to_owned())));
//         } else {
//             warn!(
//                 "Failed to collect scheduled event count result not a number: {}",
//                 str_value
//             );
//         }
//     }
//     None
// }
//
//
// pub(crate) fn get_sql_result_value(result: &SQLResult) -> Option<(i64, Option<String>)> {
//     if let Some(entry) = result.result.get(1) {
//         return get_sql_entry_value(entry);
//     }
//     None
// }