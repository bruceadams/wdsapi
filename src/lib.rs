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
pub enum Deleted {
    #[serde(rename="deleted")]
    Deleted,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub enum Status {
    #[serde(rename="active")]
    Active,
    #[serde(rename="pending")]
    Pending,
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
    pub index_capacity: Option<IndexCapacity>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Environments {
    pub environments: Vec<Environment>,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct FontSetting {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bold: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub italic: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct PdfHeading {
    pub fonts: Option<Vec<FontSetting>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct PdfSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heading: Option<PdfHeading>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct StyleSetting {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub names: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct WordHeading {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fonts: Option<Vec<FontSetting>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub styles: Option<Vec<StyleSetting>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct WordSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heading: Option<WordHeading>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct XPathPatterns {
    #[serde(skip_serializing_if = "Option::is_none")]
    xpaths: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct HtmlSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_tags_completely: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_tags_keep_content: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_content: Option<XPathPatterns>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_content: Option<XPathPatterns>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_tag_attributes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_tag_attributes: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub enum NormalizationOperation {
    #[serde(rename="copy")]
    Copy,
    #[serde(rename="move")]
    Move,
    #[serde(rename="merge")]
    Merge,
    #[serde(rename="remove")]
    Remove,
    #[serde(rename="remove_nulls")]
    RemoveNulls,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct JsonNormalization {
    // The documentation describes each of these fields as optional.
    // That doesn't sound right to me, so I'm requiring them here.
    pub operation: NormalizationOperation,
    pub source_field: String,
    pub destination_field: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Conversions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pdf: Option<PdfSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    word: Option<WordSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    html: Option<HtmlSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    json_normalizations: Option<Vec<JsonNormalization>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub enum Language {
    #[serde(rename="english")]
    English,
    #[serde(rename="german")]
    German,
    #[serde(rename="french")]
    French,
    #[serde(rename="italian")]
    Italian,
    #[serde(rename="portuguese")]
    Portuguese,
    #[serde(rename="russian")]
    Russian,
    #[serde(rename="spanish")]
    Spanish,
    #[serde(rename="swedish")]
    Swedish,
    #[serde(rename="en")]
    En,
    #[serde(rename="eng")]
    Eng,
    #[serde(rename="de")]
    De,
    #[serde(rename="ger")]
    Ger,
    #[serde(rename="deu")]
    Deu,
    #[serde(rename="fr")]
    Fr,
    #[serde(rename="fre")]
    Fre,
    #[serde(rename="fra")]
    Fra,
    #[serde(rename="it")]
    It,
    #[serde(rename="ita")]
    Ita,
    #[serde(rename="pt")]
    Pt,
    #[serde(rename="por")]
    Por,
    #[serde(rename="ru")]
    Ru,
    #[serde(rename="rus")]
    Rus,
    #[serde(rename="es")]
    Es,
    #[serde(rename="spa")]
    Spa,
    #[serde(rename="sv")]
    Sv,
    #[serde(rename="swe")]
    Swe,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct EnrichmentOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    extract: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sentiment: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    quotations: Option<bool>,
    #[serde(rename="showSourceText", skip_serializing_if = "Option::is_none")]
    show_source_text: Option<bool>,
    #[serde(rename="hierarchicalTypedRelations", skip_serializing_if = "Option::is_none")]
    hierarchical_typed_relations: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    language: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Enrichment {
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    destination_field: String,
    source_field: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    overwrite: Option<bool>,
    enrichment: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    ignore_downstream_errors: Option<bool>,
    options: Option<EnrichmentOptions>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Normalization {
    #[serde(skip_serializing_if = "Option::is_none")]
    operation: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration_id: Option<String>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<DateTime<UTC>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<DateTime<UTC>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversions: Option<Conversions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enrichments: Option<Vec<Enrichment>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normalizations: Option<Vec<Normalization>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Configurations {
    pub configurations: Vec<Configuration>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServiceError {
    pub code: u64,
    #[serde(default)]
    pub error: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

// When the response from the service does not match expectations,
// generate a ServiceError that wraps the body of the response.
fn unknown_service_error(response_body: &str) -> ServiceError {
    ServiceError {
        code: 0,
        error: String::new(),
        message: "Unknown service error format".to_string(),
        description: response_body.to_string(),
    }
}

fn service_error(response_body: &str) -> ServiceError {
    // The body of service errors usually conforms to:
    // { "code": 456, "error": "Human readable" }
    // or sometimes to:
    // { "code": 456, "message": "Summary", "description": "Detail" }
    let service_error = match from_str(response_body) {
        Ok(e) => e,
        Err(_) => unknown_service_error(response_body),
    };
    // We need some text in either "error" or "message".
    // It seems like I should be able to encode this restriction into the
    // type, but I don't know what I'm doing well enough with types and
    // Serde.
    if service_error.error.is_empty() && service_error.message.is_empty() {
        unknown_service_error(response_body)
    } else {
        service_error
    }
}

// Feels like this should be refactored into smaller parts
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
        Err(ApiError::Service(ApiErrorDetail {
            status_code: response.status,
            service_error: service_error(&response_body),
        }))
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
                          -> Result<DeletedEnvironment, ApiError> {
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

pub fn create_collection(creds: &Credentials,
                         env_id: &str,
                         options: &NewCollection)
                         -> Result<Collection, ApiError> {
    let path = "/v1/environments/".to_string() + env_id + "/collections";
    let request_body = to_string(options)
        .expect("Internal error: failed to convert NewCollection into JSON");
    let res = try!(discovery_api(&creds, Post, &path, Some(&request_body)));
    Ok(try!(from_str(&res)))
}

pub fn get_configurations(creds: &Credentials,
                          env_id: &str)
                          -> Result<Configurations, ApiError> {
    let path = "/v1/environments/".to_string() + env_id + "/configurations";
    let res = try!(discovery_api(&creds, Get, &path, None));
    Ok(try!(from_str(&res)))
}

pub fn get_configuration_detail(creds: &Credentials,
                                env_id: &str,
                                configuration_id: &str)
                                -> Result<Configuration, ApiError> {
    let path = "/v1/environments/".to_string() + env_id + "/configurations/" +
               configuration_id;
    let res = try!(discovery_api(&creds, Get, &path, None));
    Ok(try!(from_str(&res)))
}

pub fn create_configuration(creds: &Credentials,
                            env_id: &str,
                            options: &Configuration)
                            -> Result<Configuration, ApiError> {
    let path = "/v1/environments/".to_string() + env_id + "/configurations";
    let request_body = to_string(options)
        .expect("Internal error: failed to convert NewConfiguration into JSON");
    let res = try!(discovery_api(&creds, Post, &path, Some(&request_body)));
    Ok(try!(from_str(&res)))
}
