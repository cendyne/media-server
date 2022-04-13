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

use super::schema::{object, virtual_object, virtual_object_relation};
use crate::ContentEncodingValue;
use rocket::serde::{Deserialize, Serialize};

#[derive(Queryable, Debug, Clone)]
pub struct Object {
    pub id: i32,
    pub content_hash: String,
    pub content_type: String,
    pub content_encoding: String,
    pub length: i64,
    pub file_path: String,
    pub created: i64,
    pub modified: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub content_headers: Option<String>,
}

#[derive(Insertable)]
#[table_name = "object"]
pub struct NewObject {
    pub content_hash: String,
    pub content_type: String,
    pub content_encoding: String,
    pub length: i64,
    pub file_path: String,
    pub created: i64,
    pub modified: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub content_headers: Option<String>,
}

#[derive(AsChangeset)]
#[table_name = "object"]
pub struct UpdateObject {
    pub id: i32,
    pub modified: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub content_type: String,
    pub content_encoding: String,
    pub content_headers: Option<String>,
}

#[derive(Queryable, Debug)]
pub struct VirtualObject {
    pub id: i32,
    pub object_path: String,
}

#[derive(Insertable)]
#[table_name = "virtual_object"]
pub struct NewVirtualObject {
    pub object_path: String,
}

#[derive(Insertable)]
#[table_name = "virtual_object_relation"]
pub struct ReplaceVirtualObjectRelation {
    pub virtual_object_id: i32,
    pub object_id: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VirtualObjectInfoResponseObject {
    pub path: String,
    pub content_type: String,
    pub content_encoding: ContentEncodingValue,
    pub content_length: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

#[derive(Serialize)]
pub struct VirtualObjectInfoResponse {
    pub path: String,
    pub objects: Vec<VirtualObjectInfoResponseObject>,
}

#[derive(Deserialize, Debug)]
pub struct UpsertVirtualObjectRequestObjectReference {
    pub path: String,
}

#[derive(Deserialize, Debug)]
pub struct UpsertVirtualObjectRequest {
    pub objects: Vec<UpsertVirtualObjectRequestObjectReference>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpsertObjectResponse {
    pub path: String,
    pub content_type: String,
    pub content_encoding: String,
    pub content_length: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
}
