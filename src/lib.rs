#[macro_use]
extern crate diesel;
extern crate dotenv;

pub mod models;
pub mod schema;

use ct_codecs::{Base64UrlSafeNoPadding, Encoder};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel::sql_types;
use diesel::sqlite::SqliteConnection;
use dotenv::dotenv;
use either::Either;
use phf::{phf_map, phf_set};
use rocket::http::ContentType;
use rocket::request::Request;
use std::collections::HashSet;
use std::fs::create_dir_all;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use models::{
    NewObject, NewVirtualObject, Object, ReplaceVirtualObjectRelation, UpdateObject, VirtualObject,
};

pub type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

// pub const JXL: ContentType = ContentType::from(MediaType::const_new("image", "jxl", &[]));
// pub const MP3: ContentType = ContentType::from(MediaType::const_new("audio", "mpeg", &[]));
// pub const YAML: ContentType = ContentType::from(MediaType::const_new("application", "yaml", &[]));
// pub const TOML: ContentType = ContentType::from(MediaType::const_new("application", "toml", &[]));

pub const SAFE_EXTS: phf::Set<&'static str> = phf_set! {
    "7z",
    "aac",
    "avif",
    "bin",
    "bz",
    "bz2",
    "css",
    "csv",
    "gif",
    "gz",
    "html",
    "ico",
    "jar",
    "jpg",
    "js",
    "json",
    "jxl",
    "mid",
    "mp3",
    "mp4",
    "ogg",
    "ogv",
    "opus",
    "otf",
    "pdf",
    "png",
    "svg",
    "tar",
    "ttf",
    "tif",
    "toml",
    "txt",
    "weba",
    "webm",
    "webp",
    "woff",
    "woff2",
    "yaml",
    "zip",
};

pub const EXTENSION_CONTENT_TYPES: phf::Map<&'static str, (&'static str, &'static str)> = phf_map! {
    "7z" => ("application", "x-7z-compressed"),
    "aac" => ("audio", "aac"),
    "avif" => ("image", "avif"),
    "bin" => ("application", "octet-stream"),
    "bz" => ("application", "x-bzip"),
    "bz2" => ("application", "x-bzip2"),
    "css" => ("text", "css"),
    "csv" => ("text", "csv"),
    "gif" => ("image", "gif"),
    "gz" => ("application", "gzip"),
    "html" => ("text", "html"),
    "ico" => ("image", "vnd.microsoft.icon"),
    "jar" => ("application", "java-archive"),
    "jpg" => ("image", "jpeg"),
    "jpeg" => ("image", "jpeg"),
    "js" => ("text", "javascript"),
    "json" => ("application", "json"),
    "jxl" => ("image", "jxl"),
    "mid" => ("audio", "midi"),
    "mp3" => ("audio", "mpeg"),
    "mp4" => ("video", "mp4"),
    "oga" => ("audio", "ogg"),
    "ogg" => ("audio", "ogg"),
    "ogv" => ("video", "ogg"),
    "opus" => ("audio", "opus"),
    "otf" => ("font", "otf"),
    "pdf" => ("application", "pdf"),
    "png" => ("image", "png"),
    "svg" => ("image", "svg+xml"),
    "tar" => ("application", "x-tar"),
    "ttf" => ("font", "ttf"),
    "tif" => ("image", "tiff"),
    "tiff" => ("image", "tiff"),
    "toml" => ("application", "toml"),
    "txt" => ("text", "plain"),
    "weba" => ("audio", "webm"),
    "webm" => ("video", "webm"),
    "webp" => ("image", "webp"),
    "woff" => ("font", "woff"),
    "woff2" => ("font", "woff2"),
    "yaml" => ("application", "yaml"),
    "zip" => ("application", "zip"),
};

pub const ALTERNATE_EXTS: phf::Map<&'static str, &'static str> = phf_map! {
    "jpeg" => "jpg",
    "htm" => "html",
    "weba" => "webm",
    "yml" => "yaml",
    "tml" => "toml",
    "midi" => "mid",
    "tiff" => "tif",
};

pub const AUDIO_TYPE_EXTENSIONS: phf::Map<&'static str, &'static str> = phf_map! {
    "mpeg" => "mp3",
    "webm" => "weba",
    "aac" => "aac",
    "ogg" => "ogg",
    "opus" => "opus",
    "midi" => "mid",
    "wav" => "wav",
};

pub const IMAGE_TYPE_EXTENSIONS: phf::Map<&'static str, &'static str> = phf_map! {
    "jxl" => "jxl",
    "tiff" => "tif",
    "jpeg" => "jpg",
    "gif" => "gif",
    "avif" => "avif",
    "png" => "png",
    "svg" => "svg",
    "svg+xml" => "svg",
    "webp" => "webp",
    "bmp" => "bmp",
};

pub const VIDEO_TYPE_EXTENSIONS: phf::Map<&'static str, &'static str> = phf_map! {
    "webm" => "webm",
    "mp4" => "mp4",
    "ogg" => "ogv",
};

pub const APPLICATION_TYPE_EXTENSIONS: phf::Map<&'static str, &'static str> = phf_map! {
    "pdf" => "pdf",
    "json" => "json",
    "yaml" => "yaml",
    "toml" => "toml",
    "x-tar" => "tar",
    "x-bzip" => "bz",
    "x-bzip2" => "bz2",
    "xml" => "xml",
    "zip" => "zip",
    "x-7z-compressed" => "7z",
    "octet-stream" => "bin",
    "gzip" => "gz",
    "java-archive" => "jar",
    "x-sh" => "sh"
};

pub const TEXT_TYPE_EXTENSIONS: phf::Map<&'static str, &'static str> = phf_map! {
    "plain" => "txt",
    "html" => "html",
    "css" => "css",
    "csv" => "csv",
    "javascript" => "js",
};

pub const FONT_TYPE_EXTENSIONS: phf::Map<&'static str, &'static str> = phf_map! {
    "otf" => "otf",
    "ttf" => "ttf",
    "woff" => "woff",
    "woff2" => "woff2",
};

pub fn content_type_to_extension<'a>(
    content_type: &ContentType,
    user_ext: &str,
) -> Result<&'a str, String> {
    let top = content_type
        .media_type()
        .top()
        .as_str()
        .to_string()
        .to_lowercase();
    let sub = content_type
        .media_type()
        .sub()
        .as_str()
        .to_string()
        .to_lowercase();
    let found = match &top[..] {
        "image" => IMAGE_TYPE_EXTENSIONS.get(&sub),
        "audio" => AUDIO_TYPE_EXTENSIONS.get(&sub),
        "video" => VIDEO_TYPE_EXTENSIONS.get(&sub),
        "application" => APPLICATION_TYPE_EXTENSIONS.get(&sub),
        "text" => TEXT_TYPE_EXTENSIONS.get(&sub),
        "font" => FONT_TYPE_EXTENSIONS.get(&sub),
        _ => None,
    };

    let ext = if let Some(e) = found {
        e
    } else {
        match SAFE_EXTS.get_key(user_ext) {
            Some(e) => e,
            None => match ALTERNATE_EXTS.get(user_ext) {
                Some(e) => e,
                None => "bin",
            },
        }
    };
    Ok(ext)
}

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

pub struct UpsertObjectCommand<'a> {
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub content_type: &'a str,
    pub length: i64,
    pub object_path: &'a str,
    pub file_path: &'a str,
    pub content_hash: &'a str,
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
                // TODO content encoding
                content_encoding: "identity".to_string(),
                length: command.length,
                object_path: command.object_path.to_string(),
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
    if objects.is_empty() {
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
    if objects.is_empty() {
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
    let to_keep_ids: HashSet<i32> = has_ids.intersection(&to_have).copied().collect();
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
    // TODO supply extension so it can try the path with and without the extension
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
    // TODO find only related objects that match content type
    // TODO consider content encoding
    let objects = find_related_objects_to_virtual_object(conn, &virtual_object)?;
    println!("Found objects {:?}", objects);
    if objects.is_empty() {
        println!("Bailing out early, objects is empty");
        return Ok(None);
    }
    // TODO switch to content type matching instead of ext
    let same_extension: Vec<Object> = match extension {
        // This is kind of dumb
        None => objects.to_vec(),
        Some(ext) => objects
            .iter()
            .filter(|o| o.file_path.ends_with(ext))
            .cloned()
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
        println!(
            "Folding left:{:?}, right:{:?}",
            (left.id, left.width, left.height),
            (right.id, right.width, right.height)
        );
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
            (Some(wl), _, Some(wr), _, Some(w), Some(h))
                if wr >= w && (wr < wl || wl < w) && h <= w =>
            {
                right
            }
            (Some(wl), _, Some(wr), _, Some(w), None) if wr >= w && (wr < wl || wl < w) => right,
            // Bias right if smaller than left but greater than desired height
            (_, Some(hl), _, Some(hr), Some(w), Some(h))
                if hr >= h && (hr < hl || hl < h) && w <= h =>
            {
                right
            }
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
    Ok(closest.cloned())
}

pub fn hash_file(path: &Path) -> Result<String, String> {
    let mut open_file = std::fs::File::open(path).map_err(|err| format!("{:?}", err))?;
    let mut buffer: [u8; 128] = [0; 128];
    let mut key: [u8; blake3::KEY_LEN] = [0; blake3::KEY_LEN];
    let keystr = "todo key here".as_bytes();
    key[..keystr.len()].copy_from_slice(keystr);
    let mut hasher = blake3::Hasher::new_keyed(&key);
    let mut read_bytes = open_file
        .read(&mut buffer)
        .map_err(|err| format!("{:?}", err))?;
    // let mut total_bytes = 0;
    while read_bytes > 0 {
        // total_bytes += read_bytes;
        hasher.update(&buffer[0..read_bytes]);
        // continue
        read_bytes = open_file
            .read(&mut buffer)
            .map_err(|err| format!("{:?}", err))?;
    }
    let hash = hasher.finalize();
    let hash_bytes = hash.as_bytes();
    let content_hash =
        Base64UrlSafeNoPadding::encode_to_string(&hash_bytes).map_err(|e| format!("{}", e))?;
    Ok(content_hash)
}

#[derive(Debug)]
pub struct ExistingFileRequestQuery {
    pub raw_path: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub extension: Option<String>,
}

pub fn parse_existing_file_request(req: &Request<'_>) -> ExistingFileRequestQuery {
    // r for resize
    // TODO detect if requested path begins with r<width>x<height>/
    // TODO extract extension
    // TODO extract encoding (identity, br, gzip, etc.)
    let raw_path = req.routed_segments(0..).collect::<Vec<_>>().join("/");
    // TODO or use path supplied width & height
    let width = req.query_value::<i32>("w").transpose().unwrap_or(None);
    let height = req.query_value::<i32>("h").transpose().unwrap_or(None);
    let extension = Path::new(&raw_path)
        .extension()
        .and_then(|os| os.to_str().map(|s| s.to_string()));

    ExistingFileRequestQuery {
        raw_path,
        width,
        height,
        extension,
    }
}

pub fn search_existing_file_query(
    conn: &SqliteConnection,
    query: ExistingFileRequestQuery,
) -> Result<Option<models::Object>, String> {
    find_object_by_parameters(
        conn,
        &query.raw_path,
        query.width,
        query.height,
        query.extension.as_deref(),
    )
    .and_then(|opt| match opt {
        Some(object) => Ok(Some(object)),
        None => find_object_by_object_path(conn, &query.raw_path),
    })
}
