use rocket::form::FromFormField;
use serde::{Deserialize, Serialize};
use std::fmt;

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

impl fmt::Display for ContentEncodingValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ContentEncodingValue::Gzip => write!(f, "gzip"),
            ContentEncodingValue::Compress => write!(f, "compress"),
            ContentEncodingValue::Deflate => write!(f, "deflate"),
            ContentEncodingValue::Brotli => write!(f, "br"),
            ContentEncodingValue::Identity => write!(f, "identity"),
            ContentEncodingValue::Default => write!(f, "*"),
        }
    }
}

impl ContentEncodingValue {
    pub fn has_fs_extension(&self) -> bool {
        !matches!(self, ContentEncodingValue::Identity | ContentEncodingValue::Default)
    }

    pub fn fs_extension(&self) -> &'static str {
        match self {
            ContentEncodingValue::Gzip => ".gz",
            ContentEncodingValue::Compress => ".z",
            ContentEncodingValue::Deflate => ".zl",
            ContentEncodingValue::Brotli => ".br",
            ContentEncodingValue::Identity => "",
            ContentEncodingValue::Default => "",
        }
    }
    pub fn from_extension(ext: &str) -> ContentEncodingValue {
        match ext {
            "" => ContentEncodingValue::Identity,
            "gz" => ContentEncodingValue::Gzip,
            "z" => ContentEncodingValue::Compress,
            "zl" => ContentEncodingValue::Deflate,
            "br" => ContentEncodingValue::Brotli,
            _ => ContentEncodingValue::Default,
        }
    }
    pub fn from_database(ext: &str) -> ContentEncodingValue {
        match ext {
            "id" => ContentEncodingValue::Identity,
            "identity" => ContentEncodingValue::Identity,
            "" => ContentEncodingValue::Identity,
            "gz" => ContentEncodingValue::Gzip,
            "gzip" => ContentEncodingValue::Gzip,
            "z" => ContentEncodingValue::Compress,
            "compress" => ContentEncodingValue::Compress,
            "zl" => ContentEncodingValue::Deflate,
            "deflate" => ContentEncodingValue::Deflate,
            "zip" => ContentEncodingValue::Deflate,
            "br" => ContentEncodingValue::Brotli,
            _ => ContentEncodingValue::Default,
        }
    }
}
