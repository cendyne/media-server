#[macro_use]
extern crate diesel;
extern crate dotenv;

pub mod models;
pub mod schema;

mod content_encoding;
mod content_type;
mod existing_file_handler;
mod file_content;
mod file_things;
mod find_object;
mod object;
mod parsing;
mod server_name;
mod sqlite;
mod virtual_object;

pub use content_encoding::ContentEncodingValue;
pub use content_type::{content_type_or_from_safe_ext, content_type_to_extension};
pub use existing_file_handler::ExistingFileHandler;
pub use file_content::FileContent;
pub use file_things::{copy_temp, hash_file, upload_path};
pub use find_object::{
    find_object_by_parameters, parse_existing_file_request, search_existing_file_query,
};
pub use object::{
    create_object, find_object_by_file_path, find_object_by_hash, update_object, upsert_object,
    UpsertObjectCommand,
};
pub use parsing::{grab_basename, Basename};
pub use server_name::ServerName;
pub use sqlite::{connect_pool, Pool};
pub use virtual_object::{
    add_virtual_object_relations, find_or_create_virtual_object_by_object_path,
    find_related_objects_to_virtual_object, find_virtual_object_by_object_path,
    replace_virtual_object_relations,
};
