use common::{ApiError, Body, Credentials, Query, discovery_api};
use hyper::method::Method::{Delete, Get, Post};
use serde_json::{Value, to_string};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct NewCollection {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration_id: Option<String>,
}

pub fn list(creds: &Credentials, env_id: &str) -> Result<Value, ApiError> {
    let path = "/v1/environments/".to_string() + env_id + "/collections";
    Ok(discovery_api(creds, Get, &path, Query::None, &Body::None)?)
}

pub fn detail(creds: &Credentials,
              env_id: &str,
              collection_id: &str)
              -> Result<Value, ApiError> {
    let path = "/v1/environments/".to_string() + env_id + "/collections/" +
               collection_id;
    Ok(discovery_api(creds, Get, &path, Query::None, &Body::None)?)
}

pub fn fields(creds: &Credentials,
              env_id: &str,
              collection_id: &str)
              -> Result<Value, ApiError> {
    let path = "/v1/environments/".to_string() + env_id + "/collections/" +
               collection_id + "/fields";
    Ok(discovery_api(creds, Get, &path, Query::None, &Body::None)?)
}

pub fn create(creds: &Credentials,
              env_id: &str,
              options: &NewCollection)
              -> Result<Value, ApiError> {
    let path = "/v1/environments/".to_string() + env_id + "/collections";
    let request_body = to_string(options)
        .expect("Internal error: failed to convert NewCollection into JSON");
    Ok(discovery_api(creds,
                     Post,
                     &path,
                     Query::None,
                     &Body::Json(&request_body))?)
}

pub fn delete(creds: &Credentials,
              env_id: &str,
              collection_id: &str)
              -> Result<Value, ApiError> {
    let path = "/v1/environments/".to_string() + env_id + "/collections/" +
               collection_id;
    Ok(discovery_api(creds, Delete, &path, Query::None, &Body::None)?)
}
