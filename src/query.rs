use common::{ApiError, Body, Credentials, Query, QueryParams, discovery_api};
use hyper::method::Method::Get;
use serde_json::Value;

pub fn query(creds: &Credentials,
             env_id: &str,
             collection_id: &str,
             query: QueryParams)
             -> Result<Value, ApiError> {
    let path = "/v1/environments/".to_string() + env_id + "/collections/" +
               collection_id + "/query";
    Ok(discovery_api(creds, Get, &path, Query::Query(query), &Body::None)?)
}

pub fn notices(creds: &Credentials,
               env_id: &str,
               collection_id: &str,
               query: QueryParams)
               -> Result<Value, ApiError> {
    let path = "/v1/environments/".to_string() + env_id + "/collections/" +
               collection_id + "/notices";
    Ok(discovery_api(creds, Get, &path, Query::Query(query), &Body::None)?)
}
