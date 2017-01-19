use chrono::{DateTime, UTC};
use common::{ApiError, Body, Credentials, Deleted, discovery_api};
use hyper::method::Method::{Delete, Get, Post};
use serde_json::de::from_str;
use serde_json::ser::to_string;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct DeletedConfiguration {
    pub configuration_id: String,
    pub status: Deleted,
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

pub fn list(creds: &Credentials,
            env_id: &str)
            -> Result<Configurations, ApiError> {
    let path = "/v1/environments/".to_string() + env_id + "/configurations";
    let res = discovery_api(creds, Get, &path, Body::None)?;
    Ok(from_str(&res)?)
}

pub fn detail(creds: &Credentials,
              env_id: &str,
              configuration_id: &str)
              -> Result<Configuration, ApiError> {
    let path = "/v1/environments/".to_string() + env_id + "/configurations/" +
               configuration_id;
    let res = discovery_api(creds, Get, &path, Body::None)?;
    Ok(from_str(&res)?)
}

pub fn create(creds: &Credentials,
              env_id: &str,
              options: &Configuration)
              -> Result<Configuration, ApiError> {
    let path = "/v1/environments/".to_string() + env_id + "/configurations";
    let request_body = to_string(options)
        .expect("Internal error: failed to convert NewConfiguration into JSON");
    let res = discovery_api(creds, Post, &path, Body::Json(&request_body))?;
    Ok(from_str(&res)?)
}

pub fn delete(creds: &Credentials,
              env_id: &str,
              configuration_id: &str)
              -> Result<DeletedConfiguration, ApiError> {
    let path = "/v1/environments/".to_string() + env_id + "/configurations/" +
               configuration_id;
    let res = discovery_api(creds, Delete, &path, Body::None)?;
    Ok(from_str(&res)?)
}
