use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use either::Either;
use std::time::SystemTime;

use crate::content_encoding::ContentEncodingValue;
use crate::models::{NewObject, Object, UpdateObject};

pub fn create_object(conn: &SqliteConnection, new_object: &NewObject) -> Result<(), String> {
    use crate::schema::object;

    let result = diesel::insert_into(object::table)
        .values(new_object)
        .execute(conn)
        .map_err(|err| format!("{}", err))?;
    if result > 0 {
        Ok(())
    } else {
        Err("Could not insert".to_string())
    }
}

pub fn update_object(
    conn: &SqliteConnection,
    id: i32,
    width: Option<i32>,
    height: Option<i32>,
    headers: Option<String>,
) -> Result<(), String> {
    use crate::schema::object;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|err| format!("{}", err))?
        .as_secs();
    let count = diesel::update(object::table)
        .set(&UpdateObject {
            id,
            modified: now as i64,
            width,
            height,
            content_headers: headers,
        })
        .filter(object::id.eq(&id))
        .execute(conn)
        .map_err(|err| format!("{}", err))?;
    println!("Updated {}", count);
    Ok(())
}

pub fn find_object_by_hash(conn: &SqliteConnection, hash: &str) -> Result<Option<Object>, String> {
    use crate::schema::object::dsl::*;
    let result = object
        .filter(content_hash.eq(hash))
        .first(conn)
        .optional()
        .map_err(|err| format!("{}", err))?;
    Ok(result)
}

pub fn find_object_by_file_path(
    conn: &SqliteConnection,
    path: &str,
) -> Result<Option<Object>, String> {
    use crate::schema::object::dsl::*;
    let result = object
        .filter(file_path.eq(path))
        .first(conn)
        .optional()
        .map_err(|err| format!("{}", err))?;
    Ok(result)
}

pub struct UpsertObjectCommand<'a> {
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub content_type: &'a str,
    pub length: i64,
    pub file_path: &'a str,
    pub content_hash: &'a str,
    pub content_encoding: ContentEncodingValue,
}

pub fn upsert_object(
    conn: &SqliteConnection,
    command: UpsertObjectCommand<'_>,
) -> Result<Either<Object, Object>, String> {
    let existing_object = find_object_by_hash(conn, command.content_hash)?;
    let mut insert = false;
    match existing_object {
        Some(obj) => {
            // TODO headers
            // TODO use content encoding (may change)
            // TODO use content type (may change)
            update_object(conn, obj.id, command.width, command.height, None)?;
        }
        None => {
            // only need to save it a first time
            insert = true;
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map_err(|err| format!("{}", err))?
                .as_secs() as i64;
            let new_object = NewObject {
                content_hash: command.content_hash.to_string(),
                content_type: command.content_type.to_string(),
                content_encoding: command.content_encoding.to_string(),
                length: command.length,
                file_path: command.file_path.to_string(),
                created: now,
                modified: now,
                width: command.width,
                height: command.height,
                // TODO headers
                content_headers: None,
            };
            create_object(conn, &new_object)?;
        }
    }
    let object = find_object_by_hash(conn, command.content_hash)?
        .ok_or_else(|| "Could not find object after upserting".to_string())?;
    if insert {
        Ok(Either::Left(object))
    } else {
        Ok(Either::Right(object))
    }
}
