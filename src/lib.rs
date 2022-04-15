// Copyright (C) 2022 Cendyne.
// This file is part of Cendyne Media-Server.

// Cendyne Media-Server is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.

// Cendyne Media-Server is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

#[macro_use]
extern crate diesel;
extern crate dotenv;

pub mod models;
pub mod schema;

mod byte_content;
mod content_encoding;
mod content_type;
mod existing_file_handler;
mod file_content;
mod file_things;
mod find_object;
mod image_operations;
mod object;
mod parsing;
mod server_name;
mod sqlite;
mod transformations;
mod virtual_object;

pub use byte_content::ByteContent;
pub use content_encoding::ContentEncodingValue;
pub use content_type::{content_type_or_from_safe_ext, content_type_to_extension};
pub use existing_file_handler::ExistingFileHandler;
pub use file_content::FileContent;
pub use file_things::{copy_temp, hash_file, upload_path};
pub use find_object::{
    find_object_by_parameters, parse_existing_file_request, search_existing_file_query,
};
pub use image_operations::open_image_dimensions_only;
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
