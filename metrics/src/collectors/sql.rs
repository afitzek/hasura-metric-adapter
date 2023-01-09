
use reqwest::Response;
use serde::{Deserialize, Serialize};
use snafu::{prelude::*, Whatever};

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