use chrono::{DateTime, TimeZone, UTC};

use hyper;
use hyper::header::{Authorization, Basic, ContentType};
use hyper::method::Method;
use hyper::mime::Attr::Charset;
use hyper::mime::Mime;
use hyper::mime::SubLevel::Json;
use hyper::mime::TopLevel::Application;
use hyper::mime::Value::Utf8;
use hyper::status::StatusCode;

use hyper_rustls::TlsClient;
use multipart::client::lazy::Multipart;

use serde_json;
use serde_json::{Value, from_reader, to_string};

use std;
use std::env;
use std::fs::File;
use std::time::UNIX_EPOCH;

header! { (XGlobalTransactionID, "X-Global-Transaction-ID") => [String] }

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Credentials {
    pub url: String,
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
struct Metadata {
    filename: String,
    last_modified: DateTime<UTC>,
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
    pub natural_language_query: Option<String>,
    pub passages: Option<bool>,
    pub aggregation: Option<String>,
    pub count: u64,
    pub return_hierarchy: Option<String>,
    pub offset: Option<u64>,
    pub sort: Option<String>,
}

#[derive(Debug)]
pub enum Query {
    Query(QueryParams),
    Config(String),
    None,
}

#[derive(Debug)]
pub struct ApiErrorDetail {
    pub status_code: StatusCode,
    pub service_error: Value,
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
        f.write_str(&to_string(&self.service_error)
            .expect("failed to format error"))
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
    Ok(from_reader(File::open(creds_file)?)?)
}

fn deal_with_query(url: &mut hyper::Url, query: Query) {
    match query {
        Query::Query(q) => {
            if let Some(filter) = q.filter {
                url.query_pairs_mut().append_pair("filter", &filter);
            };
            if let Some(query) = q.query {
                url.query_pairs_mut().append_pair("query", &query);
            };
            if let Some(natural_language_query) = q.natural_language_query {
                url.query_pairs_mut().append_pair("natural_language_query",
                                                  &natural_language_query);
            };
            if let Some(passages) = q.passages {
                url.query_pairs_mut()
                   .append_pair("passages", &format!("{}", passages));
            }
            if let Some(aggregation) = q.aggregation {
                url.query_pairs_mut().append_pair("aggregation", &aggregation);
            };
            url.query_pairs_mut().append_pair("count", &format!("{}", q.count));
            if let Some(return_hierarchy) = q.return_hierarchy {
                url.query_pairs_mut().append_pair("return", &return_hierarchy);
            };
            if let Some(offset) = q.offset {
                url.query_pairs_mut()
                   .append_pair("offset", &format!("{}", offset));
            };
            if let Some(sort) = q.sort {
                url.query_pairs_mut().append_pair("sort", &sort);
            };
        }
        Query::Config(c) => {
            url.query_pairs_mut()
               .append_pair("configuration_id", &c);
        }
        Query::None => {}
    }
}

lazy_static! {
    static ref TODAY: String = format!("{}", UTC::now().format("%F"));

    static ref CLIENT: hyper::client::Client =
        hyper::client::Client::with_connector(
            hyper::client::Pool::with_connector(
                Default::default(),
                hyper::net::HttpsConnector::new(TlsClient::new())));
}

// Feels like this should be refactored into smaller parts
pub fn discovery_api(creds: &Credentials,
                     method: Method,
                     path: &str,
                     query: Query,
                     request_body: &Body)
                     -> Result<Value, ApiError> {
    let mut url = hyper::Url::parse(&(creds.url.clone() + path))?;
    url.query_pairs_mut().append_pair("version", &TODAY);
    deal_with_query(&mut url, query);
    let txid = XGlobalTransactionID(env::var("X_GLOBAL_TRANSACTION_ID")
        .unwrap_or_else(|_| "wdscli".to_string()));
    let auth = Authorization(Basic {
        username: creds.username.clone(),
        password: Some(creds.password.clone()),
    });
    let response = match *request_body {
        Body::Json(body) => {
            let json =
                ContentType(Mime(Application, Json, vec![(Charset, Utf8)]));
            CLIENT.request(method, url)
                  .header(txid)
                  .header(auth)
                  .header(json)
                  .body(body)
                  .send()?
        }
        Body::Filename(filename) => {
            let file = File::open(filename)?;
            let modified = file.metadata()?
                               .modified()?
                               .duration_since(UNIX_EPOCH)
                               .expect("Failed to convert time?!");
            let metadata = Metadata {
                filename: filename.to_string(),
                last_modified: UTC.timestamp(modified.as_secs() as i64,
                                             modified.subsec_nanos()),
            };
            Multipart::new().add_stream("file", file, Some(filename), None)
                .add_text("metadata", to_string(&metadata)?)
                .client_request_mut(&CLIENT,
                                    url,
                                    |rb| rb.header(txid).header(auth))?
        }
        Body::None => {
            CLIENT.request(method, url).header(txid).header(auth).send()?
        }
    };

    let status = response.status;
    match from_reader(response) {
        Ok(json_body) => {
            if status.is_success() {
                Ok(json_body)
            } else {
                Err(ApiError::Service(ApiErrorDetail {
                    status_code: status,
                    service_error: json_body,
                }))
            }
        }
        Err(e) => Err(ApiError::SerdeJson(e)),
    }
}
