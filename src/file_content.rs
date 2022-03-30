use rocket::http::Header;
use rocket::request::Request;
use rocket::response::{self, Responder};

use tokio::fs::File;

use crate::models::Object;
use crate::upload_path;
use crate::ContentEncodingValue;

#[derive(Debug)]
pub struct FileContent {
    object: Object,
    file: File,
}

impl FileContent {
    pub async fn load(object: Object) -> Result<Self, String> {
        let path = upload_path()?.join(object.file_path.clone());
        let file = File::open(path).await.map_err(|e| format!("{}", e))?;

        Ok(Self { file, object })
    }
}

impl<'r> Responder<'r, 'static> for FileContent {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        let mut response = self.file.respond_to(req)?;
        let content_type = self.object.content_type;
        let content_encoding = self.object.content_encoding;

        response.set_header(Header::new("Content-Type", content_type));
        match ContentEncodingValue::from_database(&content_encoding) {
            ContentEncodingValue::Default => {}
            v => {
                response.set_header(Header::new("Content-Encoding", v.to_string()));
            }
        }
        response.set_header(Header::new("ETag", format!("\"{}\"", self.object.content_hash)));
        // TODO last modified with gmt date
        // etc.
        response.set_header(Header::new("Cache-Control", "public, max-age=86400, stale-while-revalidate=3600"));

        Ok(response)
    }
}
