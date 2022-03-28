#[macro_use]
extern crate diesel;
extern crate dotenv;

pub mod models;
pub mod schema;

mod content_encoding;
mod content_type;
mod find_object;
mod hash_file;
mod object;
mod sqlite;
mod virtual_object;

pub use content_encoding::ContentEncodingValue;
pub use content_type::{content_type_or_from_safe_ext, content_type_to_extension};
pub use find_object::{
    find_object_by_parameters, parse_existing_file_request, search_existing_file_query,
};
pub use hash_file::hash_file;
pub use object::{
    create_object, find_object_by_hash, find_object_by_object_path, update_object, upsert_object,
    UpsertObjectCommand,
};
pub use sqlite::{connect_pool, Pool};
pub use virtual_object::{
    find_or_create_virtual_object_by_object_path, replace_virtual_object_relations,
};

use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

pub fn upload_path() -> Result<PathBuf, String> {
    // TODO cache
    let path = std::env::var("UPLOAD_PATH").unwrap_or_else(|_| "./files".to_string());
    create_dir_all(&path).map_err(|err| format!("{}", err))?;
    let absolute_path = Path::new(&path)
        .canonicalize()
        .map_err(|err| format!("{}", err))?;
    Ok(absolute_path)
}
