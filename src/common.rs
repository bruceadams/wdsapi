use hyper;
use hyper::client::request::Request;
use hyper::header::{Authorization, Basic, ContentType};
use hyper::method::Method;
use hyper::mime::Attr::Charset;
use hyper::mime::Mime;
use hyper::mime::SubLevel::Json;
use hyper::mime::TopLevel::Application;
use hyper::mime::Value::Utf8;
use hyper::net::HttpsConnector;
use hyper::status::StatusCode;

use hyper_rustls::TlsClient;
use multipart::client::Multipart;

use serde_json;
use serde_json::de::{from_reader, from_str};

use std;
use std::io::Read;
use std::io::Write;

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
pub enum Body<'a> {
    Json(&'a str),
    Filename(&'a str),
    None,
}

#[derive(Debug)]
pub struct QueryParams {
    pub filter: Option<String>,
    pub query: Option<String>,
    pub aggregation: Option<String>,
    pub count: u64,
    pub return_hierarchy: Option<String>,
    pub offset: Option<u64>,
}

#[derive(Debug)]
pub enum Query {
    Query(QueryParams),
    None,
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
    HyperParse(hyper::error::ParseError),
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

impl From<hyper::error::ParseError> for ApiError {
    fn from(err: hyper::error::ParseError) -> ApiError {
        ApiError::HyperParse(err)
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
            ApiError::HyperParse(ref e) => std::fmt::Display::fmt(e, f),
        }
    }
}


pub fn credentials_from_file(creds_file: &str)
                             -> Result<Credentials, ApiError> {
    // I may wish to make the error messages more user friendly here.
    Ok(from_reader(std::fs::File::open(creds_file)?)?)
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

fn deal_with_query(url: &mut hyper::Url, query: Query) {
    match query {
        Query::Query(q) => {
            url.query_pairs_mut().append_pair("count", &format!("{}", q.count));
            if let Some(offset) = q.offset {
                url.query_pairs_mut()
                   .append_pair("offset", &format!("{}", offset));
            };
            if let Some(filter) = q.filter {
                url.query_pairs_mut().append_pair("filter", &filter);
            };
            if let Some(query) = q.query {
                url.query_pairs_mut().append_pair("query", &query);
            };
            if let Some(aggregation) = q.aggregation {
                url.query_pairs_mut().append_pair("aggregation", &aggregation);
            };
            if let Some(return_hierarchy) = q.return_hierarchy {
                url.query_pairs_mut().append_pair("return", &return_hierarchy);
            };
        }
        Query::None => {}
    }
}

// Feels like this should be refactored into smaller parts
pub fn discovery_api(creds: &Credentials,
                     method: Method,
                     path: &str,
                     query: Query,
                     request_body: Body)
                     -> Result<String, ApiError> {
    let mut url = hyper::Url::parse(&(creds.url.clone() + path))?;
    url.query_pairs_mut().append_pair("version", "2017-02-02");
    deal_with_query(&mut url, query);
    let auth = Authorization(Basic {
        username: creds.username.clone(),
        password: Some(creds.password.clone()),
    });
    let mut request =
        Request::with_connector(method,
                                url,
                                &HttpsConnector::new(TlsClient::new()))?;
    request.headers_mut().set(auth);
    let mut response = match request_body {
        Body::Json(body) => {
            let json =
                ContentType(Mime(Application, Json, vec![(Charset, Utf8)]));
            request.headers_mut().set(json);
            let mut started = request.start()?;
            started.write_all(body.as_bytes())?;
            started.send()?
        }
        Body::Filename(filename) => {
            let mut one = Multipart::from_request(request)?;
            one.write_file("file", filename)?;
            one.send()?
        }
        Body::None => request.start()?.send()?,
    };
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
