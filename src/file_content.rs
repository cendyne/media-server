use httpdate::fmt_http_date;
use rocket::http::Header;
use rocket::request::Request;
use rocket::response::{self, Responder};
use std::time::{Duration, UNIX_EPOCH};

use tokio::fs::File;

use crate::file_things::hash_base64_url_safe_no_padding;
use crate::models::Object;
use crate::upload_path;
use crate::ContentEncodingValue;

#[derive(Debug)]
pub struct FileContent {
    object: Object,
    file: File,
    etag: Option<String>,
}

impl FileContent {
    pub async fn load(object: Object) -> Result<Self, String> {
        let path = upload_path()?.join(object.file_path.clone());
        let file = File::open(path).await.map_err(|e| format!("{}", e))?;

        // The ETag is simply a re-digested object hash, and will be truncated
        let etag = if let Ok(hash) = hash_base64_url_safe_no_padding(&object.content_hash) {
            Some(format!("\"{}\"", &hash[..10]))
        } else {
            None
        };

        Ok(Self { file, object, etag })
    }
}

impl<'r> Responder<'r, 'static> for FileContent {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        let mut response = self.file.respond_to(req)?;
        let content_type = self.object.content_type;
        let content_encoding = self.object.content_encoding;

        if content_type != "application/octet-stream" {
            response.set_header(Header::new("x-content-type-options", "nosniff"));
        }

        response.set_header(Header::new("Content-Type", content_type));
        response.set_header(Header::new("Age", "0"));
        match ContentEncodingValue::from_database(&content_encoding) {
            ContentEncodingValue::Default => {}
            v => {
                response.set_header(Header::new("Content-Encoding", v.to_string()));
            }
        }

        if let Some(etag) = self.etag {
            response.set_header(Header::new("ETag", etag));
        }

        let unix_duration = Duration::from_secs(self.object.modified as u64);
        if let Some(modified) = UNIX_EPOCH.checked_add(unix_duration) {
            response.set_header(Header::new("Last-Modified", fmt_http_date(modified)));
        }

        response.set_header(Header::new(
            "Cache-Control",
            "public, max-age=86400, stale-while-revalidate=3600",
        ));

        Ok(response)
    }
}
