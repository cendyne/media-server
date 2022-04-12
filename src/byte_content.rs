use rocket::request::Request;
use rocket::response::{self, Responder, Response};
use std::io::Cursor;

use crate::content_encoding::ContentEncodingValue;

#[derive(Debug)]
pub enum ByteContentSource {
    Static(&'static [u8]),
    Dynamic(Vec<u8>),
}

#[derive(Debug)]
pub struct ByteContent {
    bytes: ByteContentSource,
    content_type: (&'static str, &'static str),
    content_encoding: ContentEncodingValue,
    cache_max_age: Option<u32>,
}

impl ByteContent {
    pub fn from_bytes(
        bytes: Vec<u8>,
        content_type: (&'static str, &'static str),
        content_encoding: ContentEncodingValue,
        cache_max_age: Option<u32>,
    ) -> Result<Self, String> {
        Ok(Self {
            bytes: ByteContentSource::Dynamic(bytes),
            content_type,
            content_encoding,
            cache_max_age,
        })
    }
    pub fn from_static_bytes(
        bytes: &'static [u8],
        content_type: (&'static str, &'static str),
        content_encoding: ContentEncodingValue,
        cache_max_age: Option<u32>,
    ) -> Result<Self, String> {
        Ok(Self {
            bytes: ByteContentSource::Static(bytes),
            content_type,
            content_encoding,
            cache_max_age,
        })
    }
}

impl<'r> Responder<'r, 'static> for ByteContent {
    fn respond_to(self, _req: &'r Request<'_>) -> response::Result<'static> {
        let mut response_builder = Response::build();
        use ByteContentSource::*;

        match self.bytes {
            Dynamic(b) => response_builder.sized_body(b.len(), Cursor::new(b)),
            Static(b) => response_builder.sized_body(b.len(), Cursor::new(b)),
        };

        let (top, sub) = self.content_type;

        // != application/octet-stream
        if top != "application" || sub != "octet-stream" {
            response_builder.raw_header("x-content-type-options", "nosniff");
        }

        response_builder.raw_header("Content-Type", format!("{}/{}", top, sub));
        match &self.content_encoding {
            ContentEncodingValue::Default => {}
            v => {
                response_builder.raw_header("Content-Encoding", v.to_string());
            }
        }
        response_builder.raw_header("Age", "0");
        // No last modified, this is on demand
        if let Some(max_age) = self.cache_max_age {
            response_builder.raw_header("Cache-Control", format!("public, max-age={}", max_age));
        }

        response_builder.ok()
    }
}
