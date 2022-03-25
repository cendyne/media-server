#[macro_use]
extern crate diesel;
extern crate dotenv;

pub mod models;
pub mod schema;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel::sql_types;
use diesel::sqlite::SqliteConnection;
use dotenv::dotenv;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use models::{
    NewObject, NewVirtualObject, Object, ReplaceVirtualObjectRelation, UpdateObject, VirtualObject,
};

pub type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

no_arg_sql_function!(last_insert_rowid, sql_types::Integer);

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
        .or_filter(file_path.eq(path))
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

pub fn find_or_create_virtual_object_by_object_path(
    conn: &SqliteConnection,
    path: &str,
) -> Result<VirtualObject, String> {
    match find_virtual_object_by_object_path(conn, path)? {
        Some(virtual_object) => Ok(virtual_object),
        None => {
            use schema::virtual_object;
            // cannot use get_result on Sqlite
            // Hint.. newer sqlite has returning..
            // feature returning_clauses_for_sqlite_3_35 has not been released yet
            let result = diesel::insert_into(virtual_object::table)
                .values(NewVirtualObject {
                    object_path: path.to_string(),
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

pub fn find_related_objects_to_virtual_object(
    conn: &SqliteConnection,
    virtual_object: &VirtualObject,
) -> Result<Vec<Object>, String> {
    use schema::virtual_object_relation::dsl::*;
    let result = virtual_object_relation
        .inner_join(schema::virtual_object::table)
        .inner_join(schema::object::table)
        .filter(virtual_object_id.eq(&virtual_object.id))
        .select(schema::object::all_columns)
        .load(conn)
        .map_err(|err| format!("{}", err))?;
    Ok(result)
}

/* TODO use around insertion
conn.transaction::<_, diesel::result::Error, _>(|| {
    delete(opts, &conn);
    Ok(())
})
.unwrap()
*/

pub fn remove_virtual_object_relations(
    conn: &SqliteConnection,
    objects: &[&Object],
    virtual_object: &VirtualObject,
) -> Result<(), String> {
    use schema::virtual_object_relation::dsl::*;
    if objects.len() == 0 {
        return Ok(());
    }
    let ids = objects.iter().map(|o| o.id);
    println!("Removing objects {:?}", ids);
    let targets = virtual_object_relation
        .filter(object_id.eq_any(ids))
        .filter(virtual_object_id.eq(&virtual_object.id));
    diesel::delete(targets)
        .execute(conn)
        .map_err(|err| format!("{}", err))?;
    Ok(())
}

pub fn add_virtual_object_relations(
    conn: &SqliteConnection,
    objects: &[Object],
    virtual_object_ref: &VirtualObject,
) -> Result<(), String> {
    if objects.len() == 0 {
        return Ok(());
    }
    // Have to annotate it so that the DSL doesn't create some
    // crazy recursion type checking exception
    // Ibzan recommends explicit type annotation on collect() use
    let relations: Vec<ReplaceVirtualObjectRelation> = objects
        .iter()
        .map(|o| ReplaceVirtualObjectRelation {
            virtual_object_id: virtual_object_ref.id,
            object_id: o.id,
        })
        .collect();
    diesel::replace_into(schema::virtual_object_relation::table)
        .values(relations)
        .execute(conn)
        .map_err(|err| format!("{}", err))?;
    Ok(())
}

pub fn replace_virtual_object_relations(
    conn: &SqliteConnection,
    objects: &[Object],
    virtual_object: &VirtualObject,
) -> Result<(), String> {
    // This method could be a lot more optimal,
    // but due to how infrequent it is used, this remains to be optimized
    let mut to_have = HashSet::new();
    for object in objects {
        to_have.insert(object.id);
    }
    let has = find_related_objects_to_virtual_object(conn, virtual_object)?;
    let has_ids: HashSet<i32> = has.iter().map(|o| o.id).collect();
    // println!("Has: {:?}", has);
    let to_keep_ids: HashSet<i32> = has_ids.intersection(&to_have).map(|i| i.clone()).collect();
    // println!("To Keep Ids: {:?}", to_keep_ids);
    let to_remove: Vec<&Object> = has
        .iter()
        .filter(|o| !to_keep_ids.contains(&o.id))
        .collect();
    // println!("To Remove: {:?}", to_remove);
    remove_virtual_object_relations(conn, &to_remove, virtual_object)?;
    // Add does a replace into, no need to do another difference
    add_virtual_object_relations(conn, objects, virtual_object)?;
    Ok(())
}

pub fn find_object_by_parameters(
    conn: &SqliteConnection,
    path: &str,
    width: Option<i32>,
    height: Option<i32>,
    extension: Option<&str>,
) -> Result<Option<Object>, String> {
    println!("Looking for virtual object by path {}", path);
    let virtual_object = match find_virtual_object_by_object_path(conn, path) {
        Ok(Some(virtual_object)) => virtual_object,
        Ok(None) => {
            println!("Could not find virtual object");
            return Ok(None);
        }
        Err(_) => {
            return Ok(None);
        }
    };
    println!("Found virtual object {:?}", virtual_object);
    let objects = find_related_objects_to_virtual_object(conn, &virtual_object)?;
    println!("Found objects {:?}", objects);
    if objects.is_empty() {
        println!("Bailing out early, objects is empty");
        return Ok(None);
    }
    // TODO switch to content type matching instead of ext
    let same_extension: Vec<Object> = match extension {
        // This is kind of dumb
        None => objects.iter().map(|o| o.clone()).collect(),
        Some(ext) => objects
            .iter()
            .filter(|o| o.file_path.ends_with(ext))
            .map(|o| o.clone())
            .collect(),
    };
    // Bail out early
    if same_extension.is_empty() {
        println!("No matching extension");
        return Ok(None);
    }
    // TODO
    println!("Looking for closest {:?}, {:?}", width, height);
    let closest = same_extension.iter().reduce(|left, right| {
        println!("Folding left:{:?}, right:{:?}", (left.id, left.width, left.height), (right.id, right.width, right.height));
        match (
            left.width,
            left.height,
            right.width,
            right.height,
            width,
            height,
        ) {
            // ---------EXACT-MATCHES--------------------
            // Keep left if exact match
            (Some(wl), Some(hl), _, _, Some(w), Some(h)) if wl == w && hl == h => left,
            // Keep right if exact match
            (_, _, Some(wr), Some(hr), Some(w), Some(h)) if wr == w && hr == h => right,
            // Keep left if width matches exactly and height is smaller than width
            (Some(wl), _, _, _, Some(w), Some(h)) if wl == w && h <= w => left,
            // Keep left if height matches exactly and width is smaller than height
            (_, Some(hl), _, _, Some(w), Some(h)) if hl == h && w <= h => left,
            // Keep right if width matches exactly and height is smaller than width
            (_, _, Some(wr), _, Some(w), Some(h)) if wr == w && h <= w => right,
            // Keep right if height matches exactly and width is smaller than height
            (_, _, _, Some(hr), Some(w), Some(h)) if hr == h && w <= h => right,
            // Keep right if width matches exactly
            (_, _, Some(wr), _, Some(w), None) if wr == w => right,
            // Keep right if height matches exactly
            (_, _, _, Some(hr), None, Some(h)) if hr == h => right,

            // ------------------------------------------
            // Bias right if smaller than left but greater than desired width
            (Some(wl), _, Some(wr), _, Some(w), Some(h)) if wr >= w && (wr < wl || wl < w) && h <= w => right,
            (Some(wl), _, Some(wr), _, Some(w), None) if wr >= w && (wr < wl || wl < w) => right,
            // Bias right if smaller than left but greater than desired height
            (_, Some(hl), _, Some(hr), Some(w), Some(h)) if hr >= h && (hr < hl || hl < h) && w <= h => right,
            (_, Some(hl), _, Some(hr), None, Some(h)) if hr >= h && (hr < hl || hl < h) => right,
            // Bias right if width is a greater size
            (None, _, Some(wr), _, Some(w), Some(h)) if wr >= w && h <= w => right,
            (None, _, Some(wr), _, Some(w), None) if wr >= w => right,
            // Bias right if height is a greater size
            (_, None, _, Some(hr), Some(w), Some(h)) if hr >= h && w <= h => right,
            (_, None, _, Some(hr), None, Some(h)) if hr >= h => right,
            // Keep left if right is not greater or equal to
            _ => left,
        }
    });
    println!("Found closest {:?}", closest);
    Ok(closest.map(|o| o.clone()))
}
