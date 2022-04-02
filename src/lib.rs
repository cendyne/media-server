#[macro_use]
extern crate diesel;
extern crate dotenv;

pub mod models;
pub mod schema;

mod content_encoding;
mod content_type;
mod existing_file_handler;
mod file_content;
mod find_object;
mod hash_file;
mod object;
mod parsing;
mod sqlite;
mod virtual_object;

pub use content_encoding::ContentEncodingValue;
pub use content_type::{content_type_or_from_safe_ext, content_type_to_extension};
pub use existing_file_handler::ExistingFileHandler;
pub use file_content::FileContent;
pub use find_object::{
    find_object_by_parameters, parse_existing_file_request, search_existing_file_query,
};
pub use hash_file::hash_file;
pub use object::{
    create_object, find_object_by_file_path, find_object_by_hash, update_object, upsert_object,
    UpsertObjectCommand,
};
pub use parsing::{grab_basename, Basename};
pub use sqlite::{connect_pool, Pool};
pub use virtual_object::{
    find_or_create_virtual_object_by_object_path, find_related_objects_to_virtual_object,
    find_virtual_object_by_object_path, replace_virtual_object_relations,
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

pub fn copy_temp(from_path: &Path, to_path: &Path) -> Result<(), String> {
    use std::io::{Read, Write};
    println!("Copying temp file from {:?} to {:?}", from_path, to_path);

    let mut from_file = std::fs::File::open(from_path).map_err(|err| format!("{:?}", err))?;
    println!("Opening {:?}", to_path);
    let mut to_file = std::fs::File::create(to_path).map_err(|err| format!("{:?}", err))?;
    let mut buffer: [u8; 16384] = [0; 16384];
    let mut read_bytes = from_file
        .read(&mut buffer)
        .map_err(|err| format!("{:?}", err))?;
    to_file
        .write(&buffer[0..read_bytes])
        .map_err(|err| format!("{:?}", err))?;
    println!("Wrote first chunk");

    // let mut total_bytes = 0;
    while read_bytes > 0 {
        // continue
        read_bytes = from_file
            .read(&mut buffer)
            .map_err(|err| format!("{:?}", err))?;
        to_file
            .write(&buffer[0..read_bytes])
            .map_err(|err| format!("{:?}", err))?;
    }
    println!("Done writing");
    Ok(())
}
