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

use models::{NewObject, UpdateObject};

pub type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

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