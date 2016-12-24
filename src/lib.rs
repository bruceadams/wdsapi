#![feature(proc_macro)]

extern crate chrono;
extern crate hyper;
extern crate hyper_rustls;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use chrono::{DateTime, UTC};
use hyper::Client;
use hyper::header::{Authorization, Basic, Headers};
use hyper::net::HttpsConnector;
use hyper::status::StatusCode;
use hyper_rustls::TlsClient;
use serde_json::de::{from_reader, from_str};
use serde_json::ser::to_string;
use std::io::Read;

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Capacity {
    pub used_bytes: u64,
    pub total_bytes: u64,
    pub used: String,
    pub total: String,
    pub percent_used: f64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct IndexCapacity {
    pub disk_usage: Capacity,
    pub memory_usage: Capacity,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub enum Deleted {
    #[serde(rename="deleted")]
    Deleted,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub enum Status {
    #[serde(rename="active")]
    Active,
    #[serde(rename="pending")]
    Pending,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct NewEnvironment {
    pub name: String,
    pub description: Option<String>,
    pub size: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Environment {
    pub environment_id: String,
    pub name: String,
    pub description: Option<String>,
    pub created: DateTime<UTC>,
    pub updated: DateTime<UTC>,
    pub status: Status,
    pub read_only: bool,
    pub index_capacity: Option<IndexCapacity>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Environments {
    pub environments: Vec<Environment>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct DocumentCounts {
    pub available: u64,
    pub processing: u64,
    pub failed: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Collection {
    pub collection_id: String,
    pub name: String,
    pub description: Option<String>,
    pub created: DateTime<UTC>,
    pub updated: DateTime<UTC>,
    pub status: Status,
    pub configuration_id: String,
    pub language: String,
    pub document_counts: Option<DocumentCounts>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Collections {
    pub collections: Vec<Collection>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    pub configuration_id: String,
    pub name: String,
    pub description: Option<String>,
    pub created: DateTime<UTC>,
    pub updated: DateTime<UTC>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Configurations {
    pub configurations: Vec<Configuration>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServiceError {
    pub code: u64,
    pub error: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Credentials {
    pub url: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug)]
pub struct ApiErrorDetail {
    pub status_code: StatusCode,
    pub service_error: ServiceError,
}

#[derive(Debug)]
pub enum ApiError {
    Service(ApiErrorDetail),
    SerdeJson(serde_json::error::Error),
    Io(std::io::Error),
    Hyper(hyper::error::Error),
}

impl From<std::io::Error> for ApiError {
    fn from(err: std::io::Error) -> ApiError {
        ApiError::Io(err)
    }
}

impl From<serde_json::error::Error> for ApiError {
    fn from(err: serde_json::error::Error) -> ApiError {
        ApiError::SerdeJson(err)
    }
}

impl From<hyper::error::Error> for ApiError {
    fn from(err: hyper::error::Error) -> ApiError {
        ApiError::Hyper(err)
    }
}

impl std::fmt::Display for ApiErrorDetail {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(&format!("{}: {}",
                             self.status_code,
                             self.service_error.error))
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            ApiError::Service(ref e) => std::fmt::Display::fmt(e, f),
            ApiError::SerdeJson(ref e) => std::fmt::Display::fmt(e, f),
            ApiError::Io(ref e) => std::fmt::Display::fmt(e, f),
            ApiError::Hyper(ref e) => std::fmt::Display::fmt(e, f),
        }
    }
}


pub fn credentials_from_file(creds_file: &str)
                             -> Result<Credentials, ApiError> {
    // I may wish to make the error messages more user friendly here.
    Ok(try!(from_reader(try!(std::fs::File::open(creds_file)))))
}

fn discovery_api(creds: &Credentials,
                 path: &str,
                 request_body: Option<&str>)
                 -> Result<String, ApiError> {
    let full_url = creds.url.clone() + path + "?version=2016-11-07";
    let mut headers = Headers::new();
    headers.set(Authorization(Basic {
        username: creds.username.clone(),
        password: Some(creds.password.clone()),
    }));
    let client = Client::with_connector(HttpsConnector::new(TlsClient::new()));
    let mut response = try!(match request_body {
        Some(body) => {
            client.post(&full_url)
                  .headers(headers)
                  .body(body)
                  .send()
        }
        None => client.get(&full_url).headers(headers).send(),
    });
    let mut response_body = String::new();

    if let Err(err) = response.read_to_string(&mut response_body) {
        if response_body.is_empty() {
            return Err(ApiError::Io(err));
        }
    }

    if response.status.is_success() {
        Ok(response_body)
    } else {
        // The body of service errors usually conforms to:
        // { "code": 456, "error": "Human readable" }
        let service_error = match from_str(&response_body) {
            Ok(se) => se,
            // If parsing the response body failed, build one.
            Err(_) => {
                ServiceError {
                    code: 0,
                    error: response_body,
                }
            }
        };
        let api_error = ApiErrorDetail {
            status_code: response.status,
            service_error: service_error,
        };
        Err(ApiError::Service(api_error))
    }
}

pub fn get_envs(creds: &Credentials) -> Result<Environments, ApiError> {
    let res = try!(discovery_api(&creds, "/v1/environments", None));
    Ok(try!(from_str(&res)))
}

pub fn create_env(creds: &Credentials,
                  options: &NewEnvironment)
                  -> Result<Environment, ApiError> {
    let request_body = to_string(options)
        .expect("Internal error: failed to convert NewEnvironment into JSON");
    let res =
        try!(discovery_api(&creds, "/v1/environments", Some(&request_body)));
    let env = from_str(&res);

    match env {
        Ok(_) => {}
        Err(_) => {
            println!("POST environments {} failed, returning: {}",
                     request_body,
                     res);
        }
    }
    Ok(try!(env))
}

pub fn get_collections(creds: &Credentials,
                       env_id: &str)
                       -> Result<Collections, ApiError> {
    let path = "/v1/environments/".to_string() + env_id + "/collections";
    let res = try!(discovery_api(&creds, &path, None));
    Ok(try!(from_str(&res)))
}

pub fn get_collection_detail(creds: &Credentials,
                             env_id: &str,
                             collection_id: &str)
                             -> Result<Collection, ApiError> {
    let path = "/v1/environments/".to_string() + env_id + "/collections/" +
               collection_id;
    let res = try!(discovery_api(&creds, &path, None));
    Ok(try!(from_str(&res)))
}

pub fn get_configurations(creds: &Credentials,
                          env_id: &str)
                          -> Result<Configurations, ApiError> {
    let path = "/v1/environments/".to_string() + env_id + "/configurations";
    let res = try!(discovery_api(&creds, &path, None));
    Ok(try!(from_str(&res)))
}
