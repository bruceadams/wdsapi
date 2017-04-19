use common::{ApiError, Body, Credentials, Query, discovery_api};
use hyper::method::Method::{Get, Post};
use serde_json::Value;

pub fn detail(creds: &Credentials,
              env_id: &str,
              collection_id: &str,
              document_id: &str)
              -> Result<Value, ApiError> {
    let path = "/v1/environments/".to_string() + env_id + "/collections/" +
               collection_id +
               "/documents/" + document_id;
    Ok(discovery_api(creds, Get, &path, Query::None, &Body::None)?)
}

pub fn create(creds: &Credentials,
              env_id: &str,
              collection_id: &str,
              configuration_id: Option<&str>,
              document_id: Option<&str>,
              filename: &str)
              -> Result<Value, ApiError> {
    let path = match document_id {
        Some(id) => {
            "/v1/environments/".to_string() + env_id + "/collections/" +
            collection_id + "/documents/" + id
        }
        None => {
            "/v1/environments/".to_string() + env_id + "/collections/" +
            collection_id + "/documents"
        }
    };
    let q = match configuration_id {
        Some(id) => Query::Config(id.to_string()),
        None => Query::None,
    };
    Ok(discovery_api(creds, Post, &path, q, &Body::Filename(filename))?)
}
