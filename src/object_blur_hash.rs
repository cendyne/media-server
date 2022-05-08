use crate::content_type::*;
use crate::image_operations::*;
use crate::models::Object;
use crate::transformations::*;
use crate::{ContentEncodingValue, ImageSemaphore, Pool};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

pub fn find_blur_hash(
    conn: &SqliteConnection,
    id: i32,
    x: i32,
    y: i32,
    bg: &str,
) -> Result<Option<String>, String> {
    use crate::schema::object_blur_hash;
    let result = object_blur_hash::table
        .select(object_blur_hash::hash)
        .filter(object_blur_hash::object_id.eq(id))
        .filter(object_blur_hash::x_components.eq(x))
        .filter(object_blur_hash::y_components.eq(y))
        .filter(object_blur_hash::background.eq(bg))
        .first(conn)
        .optional()
        .map_err(|err| format!("{}", err))?;
    Ok(result)
}

pub fn save_blur_hash(
    conn: &SqliteConnection,
    id: i32,
    x: i32,
    y: i32,
    bg: String,
    hash: String,
) -> Result<(), String> {
    use crate::schema::object_blur_hash;
    let count: i64 = object_blur_hash::table
        .count()
        .filter(object_blur_hash::object_id.eq(id))
        .filter(object_blur_hash::x_components.eq(x))
        .filter(object_blur_hash::y_components.eq(y))
        .filter(object_blur_hash::background.eq(&bg))
        .get_result(conn)
        .map_err(|err| format!("{}", err))?;
    if count == 0 {
        diesel::insert_into(object_blur_hash::table)
            .values((
                object_blur_hash::object_id.eq(id),
                object_blur_hash::x_components.eq(x),
                object_blur_hash::y_components.eq(y),
                object_blur_hash::background.eq(bg),
                object_blur_hash::hash.eq(hash),
            ))
            .execute(conn)
            .map_err(|err| format!("{}", err))?;
    } else {
        diesel::update(object_blur_hash::table)
            .set(object_blur_hash::hash.eq(hash))
            .filter(object_blur_hash::object_id.eq(id))
            .filter(object_blur_hash::x_components.eq(x))
            .filter(object_blur_hash::y_components.eq(y))
            .filter(object_blur_hash::background.eq(bg))
            .execute(conn)
            .map_err(|err| format!("{}", err))?;
    }
    Ok(())
}

pub async fn create_blur_hash(
    object: &Object,
    x: i32,
    y: i32,
    background: Option<String>,
    sem: &ImageSemaphore,
    pool: &Pool,
) -> Result<String, String> {
    let conn = pool.get().map_err(|e| format!("{}", e))?;
    let bg = background.unwrap_or_else(|| "".to_string());
    let color = if bg.is_empty() {
        0
    } else {
        u32::from_str_radix(&bg, 16)
            .map_err(|e| format!("Could not decode background '{}': {}", bg, e))?
    };

    let transformations = TransformationList::from(vec![
        Transformation::Resize(32, 32),
        Transformation::Background(color),
    ]);
    let encoding = ContentEncodingValue::from_database(&object.content_encoding);
    if encoding != ContentEncodingValue::Identity {
        return Err(format!(
            "Object has content encoding {} which is not supported",
            encoding
        ));
    }

    // Ensures the image format is supported
    if let Some((top, _)) = find_known_content_type(&object.content_type) {
        if top != "image" {
            return Err(format!(
                "Content type \"{}\" is not supported",
                object.content_type
            ));
        }
    } else {
        return Err(format!(
            "Content type \"{}\" on object is unknown",
            object.content_type
        ));
    };

    let opened_image = open_image(&object.file_path, sem).await?;
    let transformed_image = apply_transformations(opened_image, transformations).await?;

    let hash = blurhash_alt::encode(
        x as u32,
        y as u32,
        transformed_image.width(),
        transformed_image.height(),
        &transformed_image.rgba_vec(),
    )
    .map_err(|err| format!("{}", err))?;

    save_blur_hash(&conn, object.id, x, y, bg, hash.clone())?;

    Ok(hash)
}
