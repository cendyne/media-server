use crate::content_encoding::*;
use crate::content_type::*;
use crate::file_things::*;
use crate::image_operations::*;
use crate::models::*;
use crate::object::*;
use crate::sqlite::*;
use crate::transformations::*;
use crate::virtual_object::*;
use std::time::SystemTime;

pub async fn read_transform_encode(
    file_path: &str,
    transformations: TransformationList,
    quality: Option<u8>,
    format: ImageFormat,
    sem: &ImageSemaphore,
) -> Result<EncodedImage, String> {
    let opened_image = open_image(file_path, sem).await?;
    let transformed_image = apply_transformations(opened_image, transformations).await?;
    encode_in_memory(transformed_image, format, quality).await
}

pub async fn derive_transformed_image(
    object: &Object,
    vobj: Option<&VirtualObject>,
    transformations: TransformationList,
    quality: Option<u8>,
    format: Option<ImageFormat>,
    sem: &ImageSemaphore,
    pool: &Pool,
) -> Result<(Object, VirtualObject), String> {
    let encoding = ContentEncodingValue::from_database(&object.content_encoding);
    if encoding != ContentEncodingValue::Identity {
        return Err(format!(
            "Object has content encoding {} which is not supported",
            encoding
        ));
    }
    // Ensures the image format is supported
    let sub = if let Some((top, sub)) = find_known_content_type(&object.content_type) {
        if top != "image" {
            return Err(format!(
                "Content type \"{}\" is not supported",
                object.content_type
            ));
        }
        sub
    } else {
        return Err(format!(
            "Content type \"{}\" on object is unknown",
            object.content_type
        ));
    };

    let input_format = match sub.parse::<ImageFormat>() {
        Err(_) => {
            return Err(format!(
                "Content type \"{}\" is not supported",
                object.content_type
            ))
        }
        Ok(supported_format) => supported_format,
    };

    // By default use the same format as the input
    let encoded_format = format.unwrap_or(input_format);
    let (content_type, fs_ext) = {
        let (top, sub) = encoded_format.content_type()?;
        let content_type = format!("{}/{}", top, sub);
        let fs_ext = content_type_to_ext(top, sub)?;
        (content_type, fs_ext)
    };

    let transformation_string = transformations.to_string();
    let transformations_hash = hash_bytes_b64(transformation_string.as_bytes())?;
    let content_quality = quality;
    let encoded_image = read_transform_encode(
        &object.file_path,
        transformations,
        quality,
        encoded_format,
        sem,
    )
    .await?;
    let content_hash = keyed_hash_bytes_b64(&encoded_image.bytes)?;
    let length = encoded_image.bytes.len() as i64;
    let created = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|err| format!("{}", err))?
        .as_secs() as i64;
    let modified: i64 = created;
    let content_encoding = ContentEncodingValue::Identity.to_string();
    // Build internal file path
    let file_path = format!("{}.{}", &content_hash[..10], fs_ext);
    let virtual_object_path = content_hash[..10].to_string();
    let mut destination = upload_path()?;
    destination.push(&file_path);
    write_bytes_to_file(destination.as_path(), &encoded_image.bytes).await?;

    let new_object = NewObject {
        content_hash,
        content_type,
        content_encoding,
        length,
        file_path,
        created,
        modified,
        derived_object_id: Some(object.id),
        transforms: Some(transformation_string),
        transforms_hash: Some(transformations_hash),
        width: Some(encoded_image.width as i32),
        height: Some(encoded_image.height as i32),
        content_headers: None,
        quality: content_quality.map(|q| q as i32),
    };
    let conn = pool.get().map_err(|e| format!("{}", e))?;
    let (default_jpeg_bg, derived_virtual_object_id) = match vobj {
        Some(v) => (v.default_jpeg_bg.clone(), Some(v.id)),
        None => (None, None),
    };
    let (object, virtual_object) = tokio::task::spawn_blocking(move || {
        create_object(&conn, &new_object)?;
        let object = match find_object_by_hash(&conn, &new_object.content_hash)? {
            Some(object) => object,
            None => return Err("Internal error, could not find object just created".to_string()),
        };
        let mut virtual_object =
            find_or_create_virtual_object_by_object_path(&conn, &virtual_object_path)?;
        let id = virtual_object.id;

        // Update the virtual object in memory for later return
        virtual_object.default_jpeg_bg = default_jpeg_bg.clone();
        virtual_object.derived_virtual_object_id = derived_virtual_object_id;
        virtual_object.primary_object_id = Some(object.id);
        virtual_object.transforms = object.transforms.clone();
        virtual_object.transforms_hash = object.transforms_hash.clone();

        let update = UpdateTransformedVirtualObject {
            default_jpeg_bg,
            derived_virtual_object_id,
            primary_object_id: Some(object.id),
            transforms: object.transforms.clone(),
            transforms_hash: object.transforms_hash.clone(),
        };

        update_transformed_virtual_object(&conn, id, update)?;

        Ok((object, virtual_object))
    })
    .await
    .map_err(|e| format!("{}", e))??;

    Ok((object, virtual_object))
}
