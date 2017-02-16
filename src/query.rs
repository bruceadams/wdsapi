use common::{ApiError, Body, Credentials, Query, QueryParams, discovery_api};
use hyper::method::Method::Get;
use serde_json::Value;
use serde_json::de::from_str;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct QueryResponse {
    pub matching_results: u64,
    // I do not see any way to apply partial type information into results nor
    // aggregations.
    #[serde(default = "value_null")]
    #[serde(skip_serializing_if = "Value::is_null")]
    pub results: Value,
    #[serde(default = "value_null")]
    #[serde(skip_serializing_if = "Value::is_null")]
    pub aggregations: Value,
}

fn value_null() -> Value {
    Value::Null
}

pub fn query(creds: &Credentials,
             env_id: &str,
             collection_id: &str,
             query: QueryParams)
             -> Result<QueryResponse, ApiError> {
    let path = "/v1/environments/".to_string() + env_id + "/collections/" +
               collection_id + "/query";
    let res =
        discovery_api(creds, Get, &path, Query::Query(query), Body::None)?;
    Ok(from_str(&res)?)
}
