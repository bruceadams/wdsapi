use hyper;
use hyper::Client;
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

use serde_json;
use serde_json::de::{from_reader, from_str};

use std;
use std::io::Read;

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
pub fn discovery_api(creds: &Credentials,
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
