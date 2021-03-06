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
extern crate rocket;
extern crate diesel;
extern crate media_server;
// use diesel::prelude::*;
use media_server::*;
use rocket::form::Form;
use rocket::fs::TempFile;
use rocket::http::ContentType;

use rocket::serde::json::Json;
use rocket::State;

use either::Either;
use std::path::{Path, PathBuf};

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/robots.txt")]
async fn robots_txt() -> &'static str {
    "User-agent: *\nDisallow: /"
}

const TINY_GIF: [u8; 37] = [
    0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x21, 0xf9, 0x04,
    0x01, 0x0a, 0x00, 0x01, 0x00, 0x2c, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x02,
    0x02, 0x4c, 0x01, 0x00, 0x3b,
];

#[get("/favicon.ico")]
fn favicon() -> Result<ByteContent, String> {
    ByteContent::from_static_bytes(
        &TINY_GIF,
        ("image", "gif"),
        ContentEncodingValue::Identity,
        Some(86400),
    )
}

#[allow(clippy::too_many_arguments)]
#[put("/object/<input_path..>?<width>&<height>&<enc>&<ext>", data = "<file>")]
async fn upload_object(
    input_path: PathBuf,
    file: Form<TempFile<'_>>,
    width: Option<i32>,
    height: Option<i32>,
    pool: &State<Pool>,
    image_semaphore: &State<ImageSemaphore>,
    enc: Option<ContentEncodingValue>,
    ext: Option<&str>,
) -> Result<Json<models::UpsertObjectResponse>, String> {
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
    let content_hash = keyed_hash_file_b64(temp_path).await?;

    // Build internal file path
    let file_path = format!("{}.{}", &content_hash[..20], fs_ext);
    let virtual_object_path = &content_hash[..20];
    destination.push(&file_path);

    let length = file.len() as i64;

    // Always overwrite the file
    copy_temp(temp_path, &destination).await?;

    let mut width = width;
    let mut height = height;

    if "image" == content_type.media_type().top() && (width.is_none() || height.is_none()) {
        if let Ok((w, h)) = open_image_dimensions_only(&file_path, image_semaphore).await {
            width.replace(w as i32);
            height.replace(h as i32);
        }
    }

    let upserted_object_rl = upsert_object(
        &conn,
        UpsertObjectCommand {
            content_hash: &content_hash,
            width,
            height,
            content_type: &content_type_str,
            length,
            file_path: &file_path,
            content_encoding: encoding,
        },
    )?;

    // Left is if we have inserted instead of updated
    let upserted_object = match upserted_object_rl {
        Either::Left(object) => object,
        Either::Right(object) => object,
    };

    // Create virtual object for content hash
    let virtual_object = find_or_create_virtual_object_by_object_path(&conn, virtual_object_path)?;
    let objects = vec![upserted_object.clone()];
    replace_virtual_object_relations(&conn, &objects, &virtual_object)?;
    if let Some(object) = objects.get(0) {
        println!(
            "Setting primary object {} to {}",
            virtual_object_path, object.file_path
        );
        set_primary_object_if_none(&conn, virtual_object.id, object.id)?;
    }

    if !path.is_empty() {
        let virtual_object = find_or_create_virtual_object_by_object_path(&conn, path)?;
        replace_virtual_object_relations(&conn, &objects, &virtual_object)?;
        if let Some(object) = objects.get(0) {
            println!("Setting primary object {} to {}", path, object.file_path);
            set_primary_object(&conn, virtual_object.id, object.id)?;
        }
    }

    Ok(Json(models::UpsertObjectResponse {
        path: upserted_object.file_path,
        content_type: upserted_object.content_type,
        content_encoding: upserted_object.content_encoding,
        content_length: upserted_object.length,
        width: upserted_object.width,
        height: upserted_object.height,
    }))
}

#[put("/virtual-object/<input_path..>", data = "<body>")]
async fn upsert_virtual_object(
    input_path: PathBuf,
    body: Json<models::UpsertVirtualObjectRequest>,
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
        match find_object_by_file_path(&conn, &object.path)? {
            None => return Err(format!("Could not find object by path {}", object.path)),
            Some(ob) => objects.push(ob),
        }
    }
    replace_virtual_object_relations(&conn, &objects, &virtual_object)?;
    Ok("OK".to_string())
}

#[get("/virtual-object/<input_path..>")]
async fn get_virtual_object(
    input_path: PathBuf,
    pool: &State<Pool>,
) -> Result<Json<models::VirtualObjectInfoResponse>, String> {
    let path = input_path
        .to_str()
        .ok_or_else(|| "Could not parse path for some reason".to_string())?;
    println!("Get vobj {}", path);
    let conn = pool.get().map_err(|e| format!("{}", e))?;
    let virtual_object = find_virtual_object_by_object_path(&conn, path)?;
    match virtual_object {
        None => Err("not found".to_string()),
        Some(vobj) => {
            println!("Found vobj {:?}", vobj);
            let objects = find_related_objects_to_virtual_object(&conn, &vobj)?;
            println!("Found objects: {:?}", objects);
            Ok(Json(models::VirtualObjectInfoResponse {
                path: vobj.object_path,
                objects: objects
                    .into_iter()
                    .map(|o| models::VirtualObjectInfoResponseObject {
                        path: o.file_path,
                        content_type: o.content_type,
                        content_encoding: ContentEncodingValue::from_database(&o.content_encoding),
                        content_length: o.length,
                        width: o.width,
                        height: o.height,
                    })
                    .collect(),
            }))
        }
    }
}

#[post("/derive-object/<input_path..>", data = "<body>")]
async fn derive_objects(
    input_path: PathBuf,
    pool: &State<Pool>,
    sem: &State<ImageSemaphore>,
    body: Json<models::DeriveTransformedObjectsRequest>,
) -> Result<Json<models::DeriveTransformedObjectsResponse>, String> {
    let path = input_path
        .to_str()
        .ok_or_else(|| "Could not parse path for some reason".to_string())?;
    let conn = pool.get().map_err(|e| format!("{}", e))?;
    let virtual_object = find_virtual_object_by_object_path(&conn, path)?;
    let vobj = match virtual_object {
        None => return Err("not found".to_string()),
        Some(vobj) => vobj,
    };
    let object_id = match vobj.primary_object_id {
        None => {
            return Err("Finding dominant object not implemented".to_string());
        }
        Some(id) => id,
    };

    let obj = if let Some(object) = find_object_by_id(&conn, object_id)? {
        object
    } else {
        return Err("Internal error, could not find object".to_string());
    };

    let mut response = models::DeriveTransformedObjectsResponse {
        objects: Vec::with_capacity(body.objects.len()),
        blur_hash: Vec::with_capacity(body.blur_hash.len()),
    };
    let vobj_opt = Some(&vobj);
    for derived_object in &body.objects {
        let transforms = derived_object
            .transforms
            .clone()
            .unwrap_or_else(TransformationList::empty);
        let mut blur_hashes = Vec::with_capacity(derived_object.blur_hash.len());
        match derive_transformed_image(
            &obj,
            vobj_opt,
            transforms,
            derived_object.quality,
            derived_object.content_type.parse::<ImageFormat>().ok(),
            sem,
            pool,
        )
        .await
        {
            Ok((object, virtual_object)) => {
                // TODO refactor
                let vobj =
                    find_or_create_virtual_object_by_object_path(&conn, &derived_object.path)?;
                let update = models::UpdateTransformedVirtualObject {
                    default_jpeg_bg: virtual_object.default_jpeg_bg,
                    derived_virtual_object_id: virtual_object.derived_virtual_object_id,
                    primary_object_id: Some(object.id),
                    transforms: object.transforms.clone(),
                    transforms_hash: object.transforms_hash.clone(),
                };

                let objects = vec![object.clone()];
                replace_virtual_object_relations(&conn, &objects, &vobj)?;
                println!("Replaced object relations {:?}", objects);
                println!("Updating virtual object {:?}", update);
                update_transformed_virtual_object(&conn, vobj.id, update)?;
                if let Some(object) = objects.get(0) {
                    for blur_hash in &derived_object.blur_hash {
                        let bg = blur_hash.bg.clone();
                        let hash = create_blur_hash(
                            object,
                            blur_hash.x.unwrap_or(3),
                            blur_hash.y.unwrap_or(3),
                            blur_hash.bg.clone(),
                            sem,
                            pool,
                        )
                        .await?;
                        blur_hashes.push(models::DeriveTransformedObjectsResponseBlurHash {
                            x: blur_hash.x,
                            y: blur_hash.y,
                            bg,
                            hash,
                        });
                    }
                }
            }
            Err(e) => {
                return Err(e);
            }
        }
        response
            .objects
            .push(models::DeriveTransformedObjectsResponseObject {
                path: derived_object.path.clone(),
                blur_hash: blur_hashes,
            });

        println!("Output {:?}", derived_object);
    }

    for blur_hash in &body.blur_hash {
        let bg = blur_hash.bg.clone();
        let hash = create_blur_hash(
            &obj,
            blur_hash.x.unwrap_or(3),
            blur_hash.y.unwrap_or(3),
            blur_hash.bg.clone(),
            sem,
            pool,
        )
        .await?;
        response
            .blur_hash
            .push(models::DeriveTransformedObjectsResponseBlurHash {
                x: blur_hash.x,
                y: blur_hash.y,
                bg,
                hash,
            });
    }

    Ok(Json::from(response))
}

#[get("/blur-hash/<input_path..>?<bg>&<x>&<y>")]
async fn blur_hash(
    x: Option<i32>,
    y: Option<i32>,
    bg: Option<String>,
    input_path: PathBuf,
    pool: &State<Pool>,
    sem: &State<ImageSemaphore>,
) -> Result<String, String> {
    let path = input_path
        .to_str()
        .ok_or_else(|| "Could not parse path for some reason".to_string())?;
    let conn = pool.get().map_err(|e| format!("{}", e))?;
    let virtual_object = match find_virtual_object_by_object_path(&conn, path)? {
        Some(obj) => obj,
        None => {
            return Err(format!("Could not find {}", path));
        }
    };
    let object_id = match virtual_object.primary_object_id {
        None => {
            return Err("Finding dominant object not implemented".to_string());
        }
        Some(id) => id,
    };
    let bg_str = bg.unwrap_or_else(|| "".to_string());
    if let Some(hash) = find_blur_hash(&conn, object_id, x.unwrap_or(3), y.unwrap_or(3), &bg_str)? {
        return Ok(hash);
    };

    let object = if let Some(object) = find_object_by_id(&conn, object_id)? {
        object
    } else {
        return Err("Internal error, could not find object".to_string());
    };

    let hash = create_blur_hash(
        &object,
        x.unwrap_or(3),
        y.unwrap_or(3),
        Some(bg_str),
        sem,
        pool,
    )
    .await?;

    Ok(hash)
}

#[launch]
fn rocket() -> _ {
    dotenv::dotenv().ok();
    let connection_pool = connect_pool();
    let image_semaphore = ImageSemaphore::new(1);
    rocket::build()
        .manage(connection_pool)
        .manage(image_semaphore)
        .mount(
            "/",
            routes![
                index,
                favicon,
                robots_txt,
                upload_object,
                upsert_virtual_object,
                get_virtual_object,
                derive_objects,
                blur_hash,
            ],
        )
        .mount("/", ExistingFileHandler())
        .attach(rocket::shield::Shield::new())
        .attach(ServerName::new("Cendyne Media"))
}
