use chrono::{DateTime, UTC};
use common::{ApiError, Body, Credentials, Deleted, Status, discovery_api};
use hyper::method::Method::{Delete, Get, Post};
use serde_json::de::from_str;
use serde_json::ser::to_string;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct DeletedCollection {
    pub collection_id: String,
    pub status: Deleted,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct DocumentCounts {
    pub available: u64,
    pub processing: u64,
    pub failed: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct NewCollection {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Collection {
    pub collection_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub created: DateTime<UTC>,
    pub updated: DateTime<UTC>,
    pub status: Status,
    pub configuration_id: String,
    pub language: String,
    pub document_counts: Option<DocumentCounts>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Collections {
    pub collections: Vec<Collection>,
}


pub fn list(creds: &Credentials,
            env_id: &str)
            -> Result<Collections, ApiError> {
    let path = "/v1/environments/".to_string() + env_id + "/collections";
    let res = discovery_api(creds, Get, &path, Body::None)?;
    Ok(from_str(&res)?)
}

pub fn detail(creds: &Credentials,
              env_id: &str,
              collection_id: &str)
              -> Result<Collection, ApiError> {
    let path = "/v1/environments/".to_string() + env_id + "/collections/" +
               collection_id;
    let res = discovery_api(creds, Get, &path, Body::None)?;
    Ok(from_str(&res)?)
}

pub fn create(creds: &Credentials,
              env_id: &str,
              options: &NewCollection)
              -> Result<Collection, ApiError> {
    let path = "/v1/environments/".to_string() + env_id + "/collections";
    let request_body = to_string(options)
        .expect("Internal error: failed to convert NewCollection into JSON");
    let res = discovery_api(creds, Post, &path, Body::Json(&request_body))?;
    Ok(from_str(&res)?)
}

pub fn delete(creds: &Credentials,
              env_id: &str,
              collection_id: &str)
              -> Result<DeletedCollection, ApiError> {
    let path = "/v1/environments/".to_string() + env_id + "/collections/" +
               collection_id;
    let res = discovery_api(creds, Delete, &path, Body::None)?;
    Ok(from_str(&res)?)
}
