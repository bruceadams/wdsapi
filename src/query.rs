use common::{ApiError, Body, Credentials, Query, QueryParams, discovery_api};
use hyper::method::Method::Get;
use serde_json::Value;
use serde_json::from_str;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct QueryResponse {
    pub matching_results: u64,
    pub response: Value,
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
    let response =
        discovery_api(creds, Get, &path, Query::Query(query), &Body::None)?;
    Ok(QueryResponse {
        matching_results: match response["matching_results"].as_u64() {
            Some(n) => n,
            None => 0,
        },
        response: response,
    })
}
