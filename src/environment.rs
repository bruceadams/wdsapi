use chrono::{DateTime, UTC};
use common::{ApiError, Body, Credentials, Deleted, Query, Status,
             discovery_api};
use hyper::method::Method::{Delete, Get, Post};
use serde_json::Value;
use serde_json::de::from_str;
use serde_json::ser::to_string;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Capacity {
    pub used_bytes: u64,
    pub total_bytes: u64,
    pub used: String,
    pub total: String,
    pub percent_used: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct IndexCapacity {
    pub disk_usage: Capacity,
    pub memory_usage: Capacity,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct DeletedEnvironment {
    pub environment_id: String,
    pub status: Deleted,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct NewEnvironment {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub size: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Environment {
    pub environment_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub created: DateTime<UTC>,
    pub updated: DateTime<UTC>,
    pub status: Status,
    pub read_only: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index_capacity: Option<IndexCapacity>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Environments {
    pub environments: Vec<Environment>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TestDocument {
    pub configuration_id: String,
    pub status: String,
    pub enriched_field_units: u64,
    pub original_media_type: String,
    pub snapshots: Vec<Value>,
    pub notices: Vec<Value>,
}

pub fn list(creds: &Credentials) -> Result<Environments, ApiError> {
    let res =
        discovery_api(creds, Get, "/v1/environments", Query::None, Body::None)?;
    Ok(from_str(&res)?)
}

pub fn detail(creds: &Credentials,
              env_id: &str)
              -> Result<Environment, ApiError> {
    let path = "/v1/environments/".to_string() + env_id;
    let res = discovery_api(creds, Get, &path, Query::None, Body::None)?;
    Ok(from_str(&res)?)
}

pub fn preview(creds: &Credentials,
               env_id: &str,
               configuration_id: &str,
               filename: &str)
               -> Result<TestDocument, ApiError> {
    let path = "/v1/environments/".to_string() + env_id;
    let res = discovery_api(creds,
                            Post,
                            &path,
                            Query::None,
                            Body::Filename(filename))?;
    Ok(from_str(&res)?)
}

pub fn create(creds: &Credentials,
              options: &NewEnvironment)
              -> Result<Environment, ApiError> {
    let request_body = to_string(options)
        .expect("Internal error: failed to convert NewEnvironment into JSON");
    let res = discovery_api(creds,
                            Post,
                            "/v1/environments",
                            Query::None,
                            Body::Json(&request_body))?;
    Ok(from_str(&res)?)
}

pub fn delete(creds: &Credentials,
              env_id: &str)
              -> Result<DeletedEnvironment, ApiError> {
    let path = "/v1/environments/".to_string() + env_id;
    let res = discovery_api(creds, Delete, &path, Query::None, Body::None)?;
    Ok(from_str(&res)?)
}
