use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[allow(dead_code)]
pub enum ContentEncodingChoice {
    #[serde(rename = "gzip")]
    Gzip,

    #[serde(rename = "compress")]
    Compress,

    #[serde(rename = "deflate")]
    Deflate,

    #[serde(rename = "br")]
    Brotli,

    #[serde(rename = "identity")]
    Identity,

    #[serde(rename = "*")]
    Default,
}
