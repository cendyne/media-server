#[macro_use]
extern crate diesel;
extern crate dotenv;

pub mod models;
pub mod schema;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel::sqlite::SqliteConnection;
use dotenv::dotenv;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use models::{NewObject, UpdateObject, NewVirtualObject, VirtualObjectRelation, VirtualObject, Object};

pub type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

no_arg_sql_function!(last_insert_rowid, diesel::types::Integer);

pub fn connect_pool() -> Pool {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL should be set in the environment to a value like database.sqlite");
    Pool::builder()
        .build(ConnectionManager::new(database_url))
        .unwrap()
}

pub fn upload_path() -> Result<PathBuf, String> {
    // TODO cache
    let path = std::env::var("UPLOAD_PATH").unwrap_or_else(|_| "./files".to_string());
    create_dir_all(&path).map_err(|err| format!("{}", err))?;
    let absolute_path = Path::new(&path)
        .canonicalize()
        .map_err(|err| format!("{}", err))?;
    Ok(absolute_path)
}

pub fn create_object(conn: &SqliteConnection, new_object: &NewObject) -> Result<(), String> {
    use schema::object;

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
    use schema::object;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|err| format!("{}", err))?
        .as_secs();
    let _ = diesel::update(object::table)
        .set(&UpdateObject {
            id,
            modified: now as i64,
            width,
            height,
            content_headers: headers,
        })
        .execute(conn)
        .map_err(|err| format!("{}", err))?;
    Ok(())
}

pub fn find_object_by_hash(
    conn: &SqliteConnection,
    hash: &str,
) -> Result<Option<models::Object>, String> {
    use schema::object::dsl::*;
    let result = object
        .filter(content_hash.eq(hash))
        .first(conn)
        .optional()
        .map_err(|err| format!("{}", err))?;
    Ok(result)
}

pub fn find_object_by_object_path(
    conn: &SqliteConnection,
    path: &str,
) -> Result<Option<models::Object>, String> {
    use schema::object::dsl::*;
    let result = object
        .filter(object_path.eq(path))
        .first(conn)
        .optional()
        .map_err(|err| format!("{}", err))?;
    Ok(result)
}

pub fn find_virtual_object_by_object_path(
    conn: &SqliteConnection,
    path: &str,
) -> Result<Option<VirtualObject>, String> {
    use schema::virtual_object::dsl::*;
    let result = virtual_object
        .filter(object_path.eq(path))
        .first(conn)
        .optional()
        .map_err(|err| format!("{}", err))?;
    Ok(result)
}

pub fn find_or_create_virtual_object_by_object_path(conn: &SqliteConnection, path: &str) -> Result<VirtualObject, String> {
    match find_virtual_object_by_object_path(conn, path)? {
        Some(virtual_object) => Ok(virtual_object),
        None => {
            use schema::virtual_object;
            // cannot use get_result on Sqlite
            // Hint.. newer sqlite has returning..
            // feature returning_clauses_for_sqlite_3_35 has not been released yet
            let result = diesel::insert_into(virtual_object::table)
                .values(NewVirtualObject {
                    object_path: path.to_string()
                })
                .execute(conn)
                .map_err(|err| format!("{}", err))?;
            if result > 0 {
                use schema::virtual_object::dsl::*;
                // TODO improve by running in a transaction
                let last_id = diesel::select(last_insert_rowid)
                    .get_result::<i32>(conn)
                    .map_err(|err| format!("{}", err))?;
                let record = virtual_object
                    .filter(id.eq(last_id))
                    .first(conn)
                    .map_err(|err| format!("{}", err))?;
                Ok(record)
            } else {
                Err("Could not insert".to_string())
            }
        }
    }
}

pub fn find_object_by_parameters(
    conn: &SqliteConnection,
    path: &str,
    width: Option<i32>,
    height: Option<i32>,
    extension: Option<&str>
) -> Result<Option<Object>, String> {
    println!("Looking for virtual object by path {}", path);
    let virtual_object = match find_virtual_object_by_object_path(conn, path) {
        Ok(Some(virtual_object)) => virtual_object,
        Ok(None) => {
            return Ok(None);
        }
        Err(_) => {
            return Ok(None);
        }
    };
    println!("Found virtual object {:?}", virtual_object);
    // TODO
    // TODO implement virtual object lookup and subsequent object
    Err("TODO".to_string())
}