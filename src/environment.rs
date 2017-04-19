use common::{ApiError, Body, Credentials, Query, discovery_api};
use hyper::method::Method::{Delete, Get, Post};
use serde_json::{Value, to_string};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct NewEnvironment {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub size: u64,
}

pub fn list(creds: &Credentials) -> Result<Value, ApiError> {
    Ok(discovery_api(creds, Get, "/v1/environments", Query::None, &Body::None)?)
}

pub fn detail(creds: &Credentials, env_id: &str) -> Result<Value, ApiError> {
    let path = "/v1/environments/".to_string() + env_id;
    Ok(discovery_api(creds, Get, &path, Query::None, &Body::None)?)
}

pub fn preview(creds: &Credentials,
               env_id: &str,
               configuration_id: Option<&str>,
               filename: &str)
               -> Result<Value, ApiError> {
    let path = "/v1/environments/".to_string() + env_id + "/preview";
    let q = match configuration_id {
        Some(id) => Query::Config(id.to_string()),
        None => Query::None,
    };
    Ok(discovery_api(creds, Post, &path, q, &Body::Filename(filename))?)
}

pub fn create(creds: &Credentials,
              options: &NewEnvironment)
              -> Result<Value, ApiError> {
    let request_body = to_string(options)
        .expect("Internal error: failed to convert NewEnvironment into JSON");
    Ok(discovery_api(creds,
                     Post,
                     "/v1/environments",
                     Query::None,
                     &Body::Json(&request_body))?)
}

pub fn delete(creds: &Credentials, env_id: &str) -> Result<Value, ApiError> {
    let path = "/v1/environments/".to_string() + env_id;
    Ok(discovery_api(creds, Delete, &path, Query::None, &Body::None)?)
}
