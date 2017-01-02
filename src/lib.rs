#![feature(proc_macro)]

extern crate chrono;
extern crate hyper;
extern crate hyper_rustls;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use chrono::{DateTime, UTC};
use hyper::Client;
use hyper::header::{Authorization, Basic, ContentType};
use hyper::method::Method;
use hyper::method::Method::*;
use hyper::mime::Attr::Charset;
use hyper::mime::Mime;
use hyper::mime::SubLevel::Json;
use hyper::mime::TopLevel::Application;
use hyper::mime::Value::Utf8;

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
    #[serde(default)]
    pub description: String,
    pub size: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Environment {
    pub environment_id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
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
    #[serde(default)]
    pub description: String,
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
    #[serde(default)]
    pub description: String,
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
    #[serde(default)]
    pub error: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub description: String,
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
        let error = &self.service_error.error;
        let message = &self.service_error.message;
        let error_message = if error.is_empty() { message } else { error };
        if self.service_error.description.is_empty() {
            f.write_str(&format!("{}: {}", self.status_code, error_message))
        } else {
            f.write_str(&format!("{}: {}: {}",
                                 self.status_code,
                                 error_message,
                                 self.service_error.description))
        }
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
                 method: Method,
                 path: &str,
                 request_body: Option<&str>)
                 -> Result<String, ApiError> {
    let full_url = creds.url.clone() + path + "?version=2016-11-07";
    let auth = Authorization(Basic {
        username: creds.username.clone(),
        password: Some(creds.password.clone()),
    });
    let client = Client::with_connector(HttpsConnector::new(TlsClient::new()));
    let mut response = try!(match request_body {
        Some(body) => {
            let json =
                ContentType(Mime(Application, Json, vec![(Charset, Utf8)]));
            client.request(method, &full_url)
                  .header(auth)
                  .header(json)
                  .body(body)
                  .send()
        }
        None => client.request(method, &full_url).header(auth).send(),
    });
    let mut response_body = String::new();

    // We are more interested in the body of the response than any IO
    // error. Often the service closes the connection fairly abruptly when
    // it is returning an error response. We get more information from the
    // error text sent from the server than we do from an IO error such as
    // CloseNotify.
    if let Err(err) = response.read_to_string(&mut response_body) {
        if response_body.is_empty() {
            return Err(ApiError::Io(err));
        }
    }

    if response.status.is_success() {
        // 2xx HTTP response codes
        Ok(response_body)
    } else {
        // The body of service errors usually conforms to:
        // { "code": 456, "error": "Human readable" }
        // And sometimes to:
        // { "code": 456, "message": "Summary", "description": "Detail" }
        let se = from_str(&response_body).unwrap_or(ServiceError {
            code: 0,
            error: String::new(),
            message: String::new(),
            description: String::new(),
        });
        let service_error = if se.error.is_empty() && se.message.is_empty() {
            // When the response from the service does not match expectations,
            // generate a ServiceError that wraps the body of the response.
            ServiceError {
                code: 0,
                error: String::new(),
                message: "Unknown service error format".to_string(),
                description: response_body.clone(),
            }
        } else {
            se
        };

        let api_error = ApiErrorDetail {
            status_code: response.status,
            service_error: service_error,
        };
        Err(ApiError::Service(api_error))
    }
}

pub fn get_environments(creds: &Credentials) -> Result<Environments, ApiError> {
    let res = try!(discovery_api(&creds, Get, "/v1/environments", None));
    Ok(try!(from_str(&res)))
}

pub fn get_environment_detail(creds: &Credentials,
                              env_id: &str)
                              -> Result<Environment, ApiError> {
    let path = "/v1/environments/".to_string() + env_id;
    let res = try!(discovery_api(&creds, Get, &path, None));
    Ok(try!(from_str(&res)))
}

pub fn create_environment(creds: &Credentials,
                          options: &NewEnvironment)
                          -> Result<Environment, ApiError> {
    let request_body = to_string(options)
        .expect("Internal error: failed to convert NewEnvironment into JSON");
    let res = try!(discovery_api(&creds,
                                 Post,
                                 "/v1/environments",
                                 Some(&request_body)));
    Ok(try!(from_str(&res)))
}

pub fn delete_environment(creds: &Credentials,
                          env_id: &str)
                          -> Result<Environment, ApiError> {
    let path = "/v1/environments/".to_string() + env_id;
    let res = try!(discovery_api(&creds, Delete, &path, None));
    Ok(try!(from_str(&res)))
}

pub fn get_collections(creds: &Credentials,
                       env_id: &str)
                       -> Result<Collections, ApiError> {
    let path = "/v1/environments/".to_string() + env_id + "/collections";
    let res = try!(discovery_api(&creds, Get, &path, None));
    Ok(try!(from_str(&res)))
}

pub fn get_collection_detail(creds: &Credentials,
                             env_id: &str,
                             collection_id: &str)
                             -> Result<Collection, ApiError> {
    let path = "/v1/environments/".to_string() + env_id + "/collections/" +
               collection_id;
    let res = try!(discovery_api(&creds, Get, &path, None));
    Ok(try!(from_str(&res)))
}

pub fn get_configurations(creds: &Credentials,
                          env_id: &str)
                          -> Result<Configurations, ApiError> {
    let path = "/v1/environments/".to_string() + env_id + "/configurations";
    let res = try!(discovery_api(&creds, Get, &path, None));
    Ok(try!(from_str(&res)))
}
