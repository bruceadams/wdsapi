use chrono::{DateTime, UTC};
use common::{ApiError, Body, Credentials, Query, discovery_api};
use hyper::method::Method::{Get, Post};
use serde_json::de::from_str;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub enum DocumentProcessingStatus {
    #[serde(rename="available")]
    Available,
    #[serde(rename="available with notices")]
    AvailableWithNotices,
    #[serde(rename="failed")]
    Failed,
    #[serde(rename="processing")]
    Processing,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct DocumentProcessing {
    pub document_id: String,
    pub status: DocumentProcessingStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub enum Severity {
    #[serde(rename="warning")]
    Warning,
    #[serde(rename="error")]
    Error,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Notice {
    pub notice_id: String,
    pub created: DateTime<UTC>,
    pub document_id: String,
    pub severity: Severity,
    pub step: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct DocumentStatus {
    pub document_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<DateTime<UTC>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<DateTime<UTC>>,
    pub status: DocumentProcessingStatus,
    pub status_description: String,
    pub notices: Vec<Notice>,
}

pub fn detail(creds: &Credentials,
              env_id: &str,
              collection_id: &str,
              document_id: &str)
              -> Result<DocumentStatus, ApiError> {
    let path = "/v1/environments/".to_string() + env_id + "/collections/" +
               collection_id +
               "/documents/" + document_id;
    let res = discovery_api(creds, Get, &path, Query::None, &Body::None)?;
    Ok(from_str(&res)?)
}

pub fn create(creds: &Credentials,
              env_id: &str,
              collection_id: &str,
              filename: &str)
              -> Result<DocumentProcessing, ApiError> {
    let path = "/v1/environments/".to_string() + env_id + "/collections/" +
               collection_id + "/documents";
    let res = discovery_api(creds,
                            Post,
                            &path,
                            Query::None,
                            &Body::Filename(filename))?;
    Ok(from_str(&res)?)
}
