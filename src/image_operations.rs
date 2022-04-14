use image::codecs::avif::AvifEncoder;
use image::codecs::gif::GifEncoder;
use image::imageops::{blur, crop, overlay, resize, FilterType};
use image::io::Reader as ImageReader;
use image::{ColorType, ImageBuffer, ImageOutputFormat, Rgba, RgbaImage};
use std::io::{Cursor, Read, Seek, SeekFrom};
use tokio::fs::File;

use crate::file_things::upload_path;
use crate::transformations::{Transformation, TransformationList};

pub async fn open_image(input_path: &str) -> Result<RgbaImage, String> {
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
        ImageReader::open(path)
            .map_err(|e| format!("{}", e))?
            .decode()
            .map_err(|e| format!("{}", e))?
            .into_rgba8()
    };

    let dimensions = img.dimensions();
    println!(
        "Parsed image {} with dimensions {}x{}",
        input_path, dimensions.0, dimensions.1
    );
    Ok(img)
}

pub fn apply_transformations(
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

pub fn encode_in_memory(
    image: RgbaImage,
    sub: &'static str,
    quality: Option<u8>,
) -> Result<Vec<u8>, String> {
    let dimensions = image.dimensions();
    println!(
        "Output image with dimensions {}x{}",
        dimensions.0, dimensions.1
    );
    let format = match sub {
        "png" => ImageOutputFormat::Png,
        "jpeg" => ImageOutputFormat::Jpeg(quality.unwrap_or(75)),
        "gif" => {
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
        "avif" => {
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
        "webp" => {
            let encoder = webp::Encoder::from_rgba(image.as_raw(), image.width(), image.height());
            let encoded = encoder.encode(75.0);
            return Ok(encoded.to_vec());
        }
        // TODO webp
        _ => {
            return Err(format!("Unknown type {}", sub));
        }
    };

    let mut buffer = Cursor::new(Vec::new());
    image
        .write_to(&mut buffer, format)
        .map_err(|e| format!("{}", e))?;
    cursor_to_vec(buffer)
}
