use common::{ApiError, Body, Credentials, Query, discovery_api};
use hyper::method::Method::{Delete, Get, Post};
use serde_json::{Value, to_string};

pub fn list(creds: &Credentials, env_id: &str) -> Result<Value, ApiError> {
    let path = "/v1/environments/".to_string() + env_id + "/configurations";
    Ok(discovery_api(creds, Get, &path, Query::None, &Body::None)?)
}

pub fn detail(creds: &Credentials,
              env_id: &str,
              configuration_id: &str)
              -> Result<Value, ApiError> {
    let path = "/v1/environments/".to_string() + env_id + "/configurations/" +
               configuration_id;
    Ok(discovery_api(creds, Get, &path, Query::None, &Body::None)?)
}

pub fn create(creds: &Credentials,
              env_id: &str,
              configuration: &Value)
              -> Result<Value, ApiError> {
    let path = "/v1/environments/".to_string() + env_id + "/configurations";
    let request_body = to_string(configuration)
        .expect("Internal error: failed to convert configuration JSON into \
                 string");
    Ok(discovery_api(creds,
                     Post,
                     &path,
                     Query::None,
                     &Body::Json(&request_body))?)
}

pub fn delete(creds: &Credentials,
              env_id: &str,
              configuration_id: &str)
              -> Result<Value, ApiError> {
    let path = "/v1/environments/".to_string() + env_id + "/configurations/" +
               configuration_id;
    Ok(discovery_api(creds, Delete, &path, Query::None, &Body::None)?)
}
