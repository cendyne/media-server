use crate::image_operations::*;
use crate::transformations::*;

pub async fn read_transform_encode(
    file_path: &str,
    transformations: TransformationList,
    quality: Option<u8>,
    format: Option<ImageFormat>,
    sem: &ImageSemaphore,
) -> Result<EncodedImage, String> {
    let opened_image = open_image(file_path, sem).await?;
    let transformed_image = apply_transformations(opened_image, transformations).await?;
    encode_in_memory(
        transformed_image,
        format.unwrap_or(ImageFormat::PNG),
        quality,
    )
    .await
}
