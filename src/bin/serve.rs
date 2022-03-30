#[macro_use]
extern crate rocket;
extern crate diesel;
extern crate media_server;
// use diesel::prelude::*;
use media_server::*;
use rocket::form::Form;
use rocket::fs::FileServer;
use rocket::fs::TempFile;
use rocket::http::{ContentType, MediaType};

use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::State;

use either::Either;
use std::path::{Path, PathBuf};

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

const TINY_GIF: [u8; 37] = [
    0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x21, 0xf9, 0x04,
    0x01, 0x0a, 0x00, 0x01, 0x00, 0x2c, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x02,
    0x02, 0x4c, 0x01, 0x00, 0x3b,
];

#[get("/favicon.ico")]
fn favicon() -> (ContentType, &'static [u8]) {
    (ContentType::from(MediaType::GIF), &TINY_GIF)
}

#[derive(Deserialize, Debug)]
struct UpsertVirtualObjectRequestObjectReference {
    path: String,
}

#[derive(Deserialize, Debug)]
struct UpsertVirtualObjectRequest {
    objects: Vec<UpsertVirtualObjectRequestObjectReference>,
}

#[derive(Serialize, Debug)]
struct UpsertObjectResponse {
    path: String,
    unique_path: String,
    content_type: String,
    content_encoding: String,
    file_size: i64,
    width: Option<i32>,
    height: Option<i32>,
}

#[put("/object/<input_path..>?<width>&<height>&<enc>&<ext>", data = "<file>")]
async fn upload_object(
    input_path: PathBuf,
    file: Form<TempFile<'_>>,
    width: Option<i32>,
    height: Option<i32>,
    pool: &State<Pool>,
    enc: Option<ContentEncodingValue>,
    ext: Option<&str>,
) -> Result<Json<UpsertObjectResponse>, String> {
    let conn = pool.get().map_err(|e| format!("{}", e))?;
    let path = input_path
        .to_str()
        .ok_or_else(|| "Could not parse path for some reason".to_string())?;
    println!(
        "Input '{}' for {:?} enc: {:?} ext: {:?}",
        path, file, enc, ext
    );
    let mut destination = upload_path()?;
    let user_ext = ext
        .or_else(|| {
            file.raw_name()
                .map(|fname| fname.dangerous_unsafe_unsanitized_raw())
                .map(|rawname| rawname.as_str())
                // TODO look for second extension like .tar.gz
                // and set content encoding properly
                .and_then(|name| Path::new(name).extension())
                .and_then(|os| os.to_str())
        })
        .ok_or("bin")?;
    // Not all clients know that .jxl is image/jxl
    // The following will try to find out what it is
    // based on the user provided file extension,
    // should the content type be seen as binary
    // TODO also look at file content encoding output (usually identity?)
    let content_type = if let Some(user_ext) = ext {
        content_type_or_from_safe_ext(&ContentType::Binary, user_ext)
    } else {
        file.content_type().map_or_else(
            || ContentType::Binary,
            |ct| content_type_or_from_safe_ext(ct, user_ext),
        )
    };
    let content_type_str = format!(
        "{}/{}",
        content_type.media_type().top(),
        content_type.media_type().sub()
    );
    // Force into a safe known extension
    let fs_content_ext = content_type_to_extension(&content_type, user_ext)?;
    let encoding = enc.unwrap_or(ContentEncodingValue::Identity);
    let fs_ext = if encoding.has_fs_extension() {
        format!("{}{}", fs_content_ext, encoding.fs_extension())
    } else {
        fs_content_ext.to_string()
    };

    // Need path to temp file
    let temp_path = file
        .path()
        .ok_or_else(|| "File upload is unsupported".to_string())?;
    // Read temp file and generate a content hash (will be used as etag too)
    let content_hash = hash_file(temp_path)?;

    // Build internal file path
    let file_path = format!("{}.{}", &content_hash[..10], fs_ext);
    destination.push(&file_path);
    let object_path = if path.is_empty() { &file_path } else { path };

    let length = file.len() as i64;

    let upserted_object_rl = upsert_object(
        &conn,
        UpsertObjectCommand {
            content_hash: &content_hash,
            width,
            height,
            content_type: &content_type_str,
            length,
            object_path,
            file_path: &file_path,
            content_encoding: encoding,
        },
    )?;
    let upserted_object = match upserted_object_rl {
        Either::Left(object) => {
            copy_temp(temp_path, &destination)?;
            object
        }
        Either::Right(object) => object,
    };

    Ok(Json(UpsertObjectResponse {
        path: upserted_object.object_path,
        unique_path: upserted_object.file_path,
        content_type: upserted_object.content_type,
        content_encoding: upserted_object.content_encoding,
        file_size: upserted_object.length,
        width: upserted_object.width,
        height: upserted_object.height,
    }))
}

#[put("/virtual-object/<input_path..>", data = "<body>")]
async fn upsert_virtual_object(
    input_path: PathBuf,
    body: Json<UpsertVirtualObjectRequest>,
    pool: &State<Pool>,
) -> Result<String, String> {
    let path = input_path
        .to_str()
        .ok_or_else(|| "Could not parse path for some reason".to_string())?;
    let conn = pool.get().map_err(|e| format!("{}", e))?;
    let virtual_object = find_or_create_virtual_object_by_object_path(&conn, path)?;
    let mut objects = Vec::with_capacity(body.objects.len());
    // This is technically an N query, but N < 20
    // can reduce with map, and_then, collect, ok_or_else
    for object in &body.objects {
        match find_object_by_object_path(&conn, &object.path)? {
            None => return Err(format!("Could not find object by path {}", object.path)),
            Some(ob) => objects.push(ob),
        }
    }
    replace_virtual_object_relations(&conn, &objects, &virtual_object)?;
    Ok("OK".to_string())
}

#[get("/robots.txt")]
async fn robots_txt() -> &'static str {
    "User-agent: *\nDisallow: /"
}

#[launch]
fn rocket() -> _ {
    dotenv::dotenv().ok();
    let connection_pool = connect_pool();
    let static_path = upload_path().unwrap();
    rocket::build()
        .manage(connection_pool)
        // make sure find_object is LAST, ALWAYS
        .mount(
            "/",
            routes![
                index,
                favicon,
                robots_txt,
                upload_object,
                upsert_virtual_object,
            ],
        )
        .mount("/", ExistingFileHandler())
        .mount("/f", FileServer::from(static_path.as_path()))
        .attach(rocket::shield::Shield::new())
}
