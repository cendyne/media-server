use image::imageops::{blur, crop, overlay, resize, FilterType};
use image::io::Reader as ImageReader;
use image::{ImageBuffer, ImageOutputFormat, Rgba, RgbaImage};
use std::io::{Cursor, Read, Seek, SeekFrom};

use crate::file_things::upload_path;
use crate::transformations::{Transformation, TransformationList};

pub fn open_image(input_path: &str) -> Result<RgbaImage, String> {
    let mut path = upload_path()?;
    path.push(input_path);
    let img = ImageReader::open(path)
        .map_err(|e| format!("{}", e))?
        .decode()
        .map_err(|e| format!("{}", e))?
        .into_rgba8();
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
                resize(&image, w, h, FilterType::Triangle)
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
        }
    });
    Ok(result)
}

pub fn encode_in_memory(image: RgbaImage, sub: &'static str) -> Result<Vec<u8>, String> {
    let mut buffer = Cursor::new(Vec::new());
    let format = match sub {
        "png" => ImageOutputFormat::Png,
        "jpeg" => ImageOutputFormat::Jpeg(75), // quality?
        "gif" => ImageOutputFormat::Gif,
        "avif" => ImageOutputFormat::Avif,
        // TODO webp
        _ => {
            return Err(format!("Unknown type {}", sub));
        }
    };
    let dimensions = image.dimensions();
    println!(
        "Output image with dimensions {}x{}",
        dimensions.0, dimensions.1
    );
    image
        .write_to(&mut buffer, format)
        .map_err(|e| format!("{}", e))?;
    println!("Cursor length is {}", buffer.position());
    let mut out = Vec::new();
    // Rewind cursor
    buffer
        .seek(SeekFrom::Start(0))
        .map_err(|e| format!("{}", e))?;
    buffer.read_to_end(&mut out).map_err(|e| format!("{}", e))?;
    println!("Output length is {}", out.len());
    Ok(out)
}
