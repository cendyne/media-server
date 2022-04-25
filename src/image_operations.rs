use image::codecs::avif::AvifEncoder;
use image::codecs::gif::GifEncoder;
use image::imageops::{blur, crop, overlay, resize, FilterType};
use image::io::Reader as ImageReader;
use image::{ColorType, ImageBuffer, ImageOutputFormat, Rgba, RgbaImage};
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::str::FromStr;
use tokio::fs::File;
use tokio::sync::{Semaphore, SemaphorePermit};

use crate::file_things::upload_path;
use crate::transformations::{Transformation, TransformationList};

#[allow(clippy::upper_case_acronyms, dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum ImageFormat {
    PNG,
    JPEG,
    GIF,
    AVIF,
    WEBP,
    UNKNOWN,
}

impl ImageFormat {
    pub fn to_str(&self) -> Result<&'static str, String> {
        match self {
            Self::PNG => Ok("png"),
            Self::JPEG => Ok("jpeg"),
            Self::GIF => Ok("gif"),
            Self::AVIF => Ok("avif"),
            Self::WEBP => Ok("webp"),
            Self::UNKNOWN => Err("Unknown type".to_string()),
        }
    }
    pub fn content_type(&self) -> Result<(&'static str, &'static str), String> {
        self.to_str().map(|sub| ("image", sub))
    }
    #[allow(dead_code)]
    pub fn to_extension(&self) -> Result<&'static str, String> {
        match self {
            Self::PNG => Ok("png"),
            Self::JPEG => Ok("jpg"),
            Self::GIF => Ok("gif"),
            Self::AVIF => Ok("avif"),
            Self::WEBP => Ok("webp"),
            Self::UNKNOWN => Err("Unknown type".to_string()),
        }
    }
}

impl<'r> rocket::form::FromFormField<'r> for ImageFormat {
    fn from_value(field: rocket::form::ValueField<'r>) -> rocket::form::Result<'r, Self> {
        field
            .value
            .parse::<ImageFormat>()
            .map_err(|err| rocket::form::Errors::from(rocket::form::Error::validation(err)))
    }
}

impl FromStr for ImageFormat {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "png" => Ok(Self::PNG),
            "jpeg" => Ok(Self::JPEG),
            "jpg" => Ok(Self::JPEG),
            "gif" => Ok(Self::GIF),
            "avif" => Ok(Self::AVIF),
            "webp" => Ok(Self::WEBP),
            _ => Err(format!("Unrecognized type {}", s)),
        }
    }
}

pub struct LimitedImage<'a> {
    image: RgbaImage,
    permit: SemaphorePermit<'a>,
}

pub struct ImageSemaphore {
    semaphore: Semaphore,
}

impl ImageSemaphore {
    pub fn new(permits: usize) -> Self {
        Self {
            semaphore: Semaphore::new(permits),
        }
    }
}

pub async fn open_image<'a>(
    input_path: &str,
    sem: &'a ImageSemaphore,
) -> Result<LimitedImage<'a>, String> {
    let permit = sem
        .semaphore
        .acquire()
        .await
        .map_err(|e| format!("{}", e))?;
    let mut path = upload_path()?;
    path.push(input_path);
    let img = if input_path.ends_with(".webp") {
        let data = {
            use tokio::io::AsyncReadExt;
            let mut f = File::open(path).await.map_err(|e| format!("{}", e))?;
            let mut data = Vec::new();
            f.read_to_end(&mut data)
                .await
                .map_err(|e| format!("{}", e))?;
            println!("Read WebP data {} bytes", data.len());
            data
        };

        let decoder = webp::Decoder::new(&data);
        match decoder.decode() {
            None => {
                return Err("Could not decode webp".to_string());
            }
            Some(webp_image) => {
                let internal_img = webp_image.to_image().into_rgba8();
                let new_img = RgbaImage::from_raw(
                    internal_img.width(),
                    internal_img.height(),
                    internal_img.to_vec(),
                );
                match new_img {
                    Some(img) => {
                        println!("Parsed webp image!");
                        img
                    }
                    None => {
                        return Err("Could not copy webp data".to_string());
                    }
                }
            }
        }
    } else {
        let result = tokio::task::spawn_blocking(|| blocking_image_open(path))
            .await
            .map_err(|e| format!("{}", e))?;
        result?
    };

    let dimensions = img.dimensions();
    println!(
        "Parsed image {} with dimensions {}x{}",
        input_path, dimensions.0, dimensions.1
    );
    Ok(LimitedImage { image: img, permit })
}

fn blocking_image_open(path: PathBuf) -> Result<RgbaImage, String> {
    let image = ImageReader::open(path)
        .map_err(|e| format!("{}", e))?
        .decode()
        .map_err(|e| format!("{}", e))?
        .into_rgba8();
    Ok(image)
}

pub async fn open_image_dimensions_only(
    input_path: &str,
    sem: &ImageSemaphore,
) -> Result<(u32, u32), String> {
    let image = open_image(input_path, sem).await?;
    Ok(image.image.dimensions())
}

fn blocking_apply_transformations(
    image: RgbaImage,
    transformations: TransformationList,
) -> Result<RgbaImage, String> {
    use Transformation::*;
    let ts: Vec<Transformation> = transformations.list();
    let result = ts.iter().fold(image, |mut image, t| {
        println!("Applying transform {}", t);
        match t {
            // TODO preserve aspect ratio with resize
            Resize(w, h) => resize(&image, *w, *h, FilterType::Lanczos3),
            Scale(f) => {
                let dimensions = image.dimensions();
                let w = (f * (dimensions.0 as f32) / 100.0) as u32;
                let h = (f * (dimensions.1 as f32) / 100.0) as u32;
                resize(&image, w, h, FilterType::Lanczos3)
            }
            Blur(sigma) => blur(&image, *sigma),
            Background(color) => {
                let dimensions = image.dimensions();
                let r: u8 = ((*color & 0xff0000) >> 16) as u8;
                let g: u8 = ((*color & 0xff00) >> 8) as u8;
                let b: u8 = (*color & 0xff) as u8;
                let pixel = Rgba([r, g, b, 255]);

                let mut background = ImageBuffer::from_pixel(dimensions.0, dimensions.1, pixel);
                overlay(&mut background, &image, 0, 0);
                background
            }
            Crop(x, y, w, h) => crop(&mut image, *x, *y, *w, *h).to_image(),
            Noop => image,
        }
    });
    Ok(result)
}

pub async fn apply_transformations(
    image: LimitedImage<'_>,
    transformations: TransformationList,
) -> Result<LimitedImage<'_>, String> {
    let img = image.image;
    let result =
        tokio::task::spawn_blocking(|| blocking_apply_transformations(img, transformations))
            .await
            .map_err(|e| format!("{}", e))?;
    Ok(LimitedImage {
        image: result?,
        permit: image.permit,
    })
}

fn cursor_to_vec(mut buffer: Cursor<Vec<u8>>) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    // Rewind cursor
    buffer
        .seek(SeekFrom::Start(0))
        .map_err(|e| format!("{}", e))?;
    buffer.read_to_end(&mut out).map_err(|e| format!("{}", e))?;
    println!("Output length is {}", out.len());
    Ok(out)
}

fn blocking_encode_in_memory(
    image: RgbaImage,
    sub: ImageFormat,
    quality: Option<u8>,
) -> Result<Vec<u8>, String> {
    let dimensions = image.dimensions();
    println!(
        "Output image with dimensions {}x{}",
        dimensions.0, dimensions.1
    );
    let format = match sub {
        ImageFormat::PNG => ImageOutputFormat::Png,
        ImageFormat::JPEG => ImageOutputFormat::Jpeg(quality.unwrap_or(75)),
        ImageFormat::GIF => {
            let mut buffer = Cursor::new(Vec::new());

            // Enclose this in a block so that we do not mutably borrow buffer
            // in more than two places at once
            {
                let mut encoder = GifEncoder::new_with_speed(&mut buffer, 25);
                encoder
                    .encode(
                        image.as_raw(),
                        image.width(),
                        image.height(),
                        ColorType::Rgba8,
                    )
                    .map_err(|e| format!("{}", e))?;
            }
            return cursor_to_vec(buffer);
        }
        ImageFormat::AVIF => {
            let mut buffer = Cursor::new(Vec::new());
            let encoder =
                AvifEncoder::new_with_speed_quality(&mut buffer, 8, quality.unwrap_or(75));
            encoder
                .write_image(
                    image.as_raw(),
                    image.width(),
                    image.height(),
                    ColorType::Rgba8,
                )
                .map_err(|e| format!("{}", e))?;
            return cursor_to_vec(buffer);
        }
        ImageFormat::WEBP => {
            let encoder = webp::Encoder::from_rgba(image.as_raw(), image.width(), image.height());
            let encoded = encoder.encode(75.0);
            return Ok(encoded.to_vec());
        }
        _ => {
            return Err(format!("Unknown type {:?}", sub));
        }
    };

    let mut buffer = Cursor::new(Vec::new());
    image
        .write_to(&mut buffer, format)
        .map_err(|e| format!("{}", e))?;
    cursor_to_vec(buffer)
}

pub struct EncodedImage {
    pub bytes: Vec<u8>,
    pub format: ImageFormat,
}

pub async fn encode_in_memory(
    image: LimitedImage<'_>,
    format: ImageFormat,
    quality: Option<u8>,
) -> Result<EncodedImage, String> {
    let img = image.image;
    let encode_format = format.clone();
    let bytes =
        tokio::task::spawn_blocking(move || blocking_encode_in_memory(img, encode_format, quality))
            .await
            .map_err(|e| format!("{}", e))??;
    Ok(EncodedImage { bytes, format })
}

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
