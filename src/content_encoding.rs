use rocket::form::FromFormField;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, FromFormField, Clone)]
#[allow(dead_code)]
pub enum ContentEncodingValue {
    #[serde(rename = "gzip")]
    #[field(value = "gzip")]
    #[field(value = "gz")]
    Gzip,

    #[serde(rename = "compress")]
    #[field(value = "compress")]
    Compress,

    #[serde(rename = "deflate")]
    #[field(value = "deflate")]
    #[field(value = "zip")]
    Deflate,

    #[serde(rename = "br")]
    #[field(value = "br")]
    Brotli,

    #[serde(rename = "identity")]
    #[field(value = "identity")]
    #[field(value = "id")]
    Identity,

    #[serde(rename = "*")]
    #[field(value = "*")]
    Default,
}

impl ContentEncodingValue {
    pub fn to_string(&self) -> String {
        match self {
            ContentEncodingValue::Gzip => "gzip",
            ContentEncodingValue::Compress => "compress",
            ContentEncodingValue::Deflate => "deflate",
            ContentEncodingValue::Brotli => "br",
            ContentEncodingValue::Identity => "identity",
            ContentEncodingValue::Default => "*",
        }
        .to_string()
    }
    pub fn has_fs_extension(&self) -> bool {
        match self {
            ContentEncodingValue::Identity => false,
            ContentEncodingValue::Default => false,
            _ => true
        }
    }
    pub fn fs_extension(&self) -> &'static str {
        match self {
            ContentEncodingValue::Gzip => ".gz",
            ContentEncodingValue::Compress => ".z",
            ContentEncodingValue::Deflate => ".zl",
            ContentEncodingValue::Brotli => ".br",
            ContentEncodingValue::Identity => "",
            ContentEncodingValue::Default => ""
        }
    }
}
