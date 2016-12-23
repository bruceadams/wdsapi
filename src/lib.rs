#![feature(proc_macro)]

extern crate chrono;
extern crate hyper;
extern crate hyper_rustls;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use chrono::{DateTime, UTC};
use hyper::{Client, Error};
use hyper::client::response::Response;
use hyper::header::{Authorization, Basic, Headers};
use hyper::net::HttpsConnector;
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
pub struct Credentials {
    pub url: String,
    pub username: String,
    pub password: String,
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

pub fn credentials_from_file(creds_file: &str)
                             -> Result<Credentials, Box<std::error::Error>> {
    // I may wish to make the error messages more user friendly here.
    Ok(try!(from_reader(try!(std::fs::File::open(creds_file)))))
}

fn discovery_api(creds: &Credentials,
                 env_id: Option<&str>,
                 path: &str,
                 body: Option<&str>)
                 -> Result<Response, Error> {
    let path_tail = match env_id {
        Some(env_id) => "/".to_string() + env_id + path,
        None => "".to_string(),
    };
    let full_url = creds.url.clone() + "/v1/environments" + &path_tail +
                   "?version=2016-11-07";
    let mut headers = Headers::new();
    headers.set(Authorization(Basic {
        username: creds.username.clone(),
        password: Some(creds.password.clone()),
    }));
    let client = Client::with_connector(HttpsConnector::new(TlsClient::new()));
    match body {
        Some(body) => {
            client.post(&full_url)
                  .headers(headers)
                  .body(body)
                  .send()
        }
        None => client.get(&full_url).headers(headers).send(),
    }
}

pub fn get_envs(creds: &Credentials)
                -> Result<Environments, Box<std::error::Error>> {
    let res = try!(discovery_api(&creds, None, "", None));
    Ok(try!(from_reader(res)))
}

pub fn create_env(creds: &Credentials,
                  options: &NewEnvironment)
                  -> Result<Environment, Box<std::error::Error>> {
    let mut res = try!(discovery_api(&creds,
                                     None,
                                     "",
                                     Some(&to_string(options).unwrap())));
    let mut body = String::new();
    try!(res.read_to_string(&mut body));
    let env = from_str(&body);

    match env {
        Ok(_) => {}
        Err(_) => {
            println!("POST environments failed, returning: {}", body);
        }
    }
    Ok(try!(env))
}

pub fn get_collections(creds: &Credentials,
                       env_id: &str)
                       -> Result<Collections, Box<std::error::Error>> {
    let res = try!(discovery_api(&creds, Some(env_id), "/collections", None));
    Ok(try!(from_reader(res)))
}

pub fn get_collection_detail(creds: &Credentials,
                             env_id: &str,
                             collection_id: &str)
                             -> Result<Collection, Box<std::error::Error>> {
    let path = "/collections/".to_string() + collection_id;
    let res = try!(discovery_api(&creds, Some(env_id), &path, None));
    Ok(try!(from_reader(res)))
}

pub fn get_configurations(creds: &Credentials,
                          env_id: &str)
                          -> Result<Configurations, Box<std::error::Error>> {
    let res =
        try!(discovery_api(&creds, Some(env_id), "/configurations", None));
    Ok(try!(from_reader(res)))
}
