#[macro_use]
extern crate rocket;
extern crate diesel;
extern crate media_server;
// use self::models::*;
// use diesel::prelude::*;
use ct_codecs::{Base64UrlSafeNoPadding, Encoder};
use media_server::*;
use rocket::form::Form;
use rocket::fs::FileServer;
use rocket::fs::TempFile;
use rocket::http::{ContentType, MediaType};
use std::io::Read;
use std::path::Path;

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

pub const JXL: ContentType = ContentType(MediaType::const_new("image", "jxl", &[]));
pub const MP3: ContentType = ContentType(MediaType::const_new("audio", "mpeg", &[]));

#[put("/object?<path>", data = "<file>")]
async fn upload_object(path: &str, mut file: Form<TempFile<'_>>) -> Result<String, String> {
    println!("Input {} for {:?}", path, file);
    let mut destination = upload_path()?;
    let content_type = file.content_type().unwrap_or(&ContentType::Binary).clone();

    let ext = if ContentType::JPEG == content_type {
        "jpg"
    } else if ContentType::GIF == content_type {
        "gif"
    } else if ContentType::AVIF == content_type {
        "avif"
    } else if ContentType::PNG == content_type {
        "png"
    } else if ContentType::SVG == content_type {
        "svg"
    } else if ContentType::WEBP == content_type {
        // image/webp
        "webp"
    } else if ContentType::WEBM == content_type || ContentType::WEBA == content_type {
        // video/webm, audio/webm
        "webm"
    } else if ContentType::MP4 == content_type {
        "mp4"
    } else if ContentType::PDF == content_type {
        "pdf"
    } else if ContentType::Plain == content_type {
        "txt"
    } else if ContentType::HTML == content_type {
        "html"
    } else if ContentType::JSON == content_type {
        "json"
    } else if ContentType::AAC == content_type {
        "aac"
    } else if ContentType::OGG == content_type {
        "ogg"
    } else if MP3 == content_type {
        "mp3"
    } else if JXL == content_type {
        "jxl"
    } else if ContentType::ZIP == content_type {
        "zip"
    } else if ContentType::GZIP == content_type {
        "gz"
    } else if ContentType::TAR == content_type {
        "tar"
    } else if ContentType::CSV == content_type {
        "csv"
    } else {
        println!("Looking at name {:?}", file.raw_name());
        let ext = file
            .raw_name()
            .map(|fname| fname.dangerous_unsafe_unsanitized_raw())
            .map(|rawname| rawname.as_str())
            .and_then(|name| Path::new(name).extension())
            .and_then(|os| os.to_str())
            .ok_or("bin")?;
        // todo make better
        match ext {
            "jxl" => "jxl",
            _ => "bin",
        }
    };

    let temp_path = file
        .path()
        .ok_or_else(|| "File upload is unsupported".to_string())?;
    let mut open_file = std::fs::File::open(temp_path).map_err(|err| format!("{:?}", err))?;
    let mut buffer: [u8; 128] = [0; 128];
    let mut key: [u8; blake3::KEY_LEN] = [0; blake3::KEY_LEN];
    let keystr = "todo key here".as_bytes();
    key[..keystr.len()].copy_from_slice(keystr);
    let mut hasher = blake3::Hasher::new_keyed(&key);
    let mut read_bytes = open_file
        .read(&mut buffer)
        .map_err(|err| format!("{:?}", err))?;
    let mut total_bytes = 0;
    while read_bytes > 0 {
        total_bytes += read_bytes;
        hasher.update(&buffer[0..read_bytes]);
        // continue
        read_bytes = open_file
            .read(&mut buffer)
            .map_err(|err| format!("{:?}", err))?;
    }
    println!("Total bytes {} vs len {}", total_bytes, file.len());
    let hash = hasher.finalize();
    let hash_bytes = hash.as_bytes();
    let content_name = Base64UrlSafeNoPadding::encode_to_string(&hash_bytes[0..8])
        .map_err(|e| format!("{}", e))?;
    destination.push(format!("{}.{}", content_name, ext));
    file.persist_to(&destination)
        .await
        .map_err(|err| format!("{}", err))?;
    // TODO save to db
    Ok(format!("{:?}", destination))
}

#[launch]
fn rocket() -> _ {
    dotenv::dotenv().ok();
    let connection_pool = connect_pool();
    let static_path = upload_path().unwrap();
    rocket::build()
        .manage(connection_pool)
        .mount("/", routes![index, favicon, upload_object])
        .mount("/f", FileServer::from(static_path.as_path()))
        .attach(rocket::shield::Shield::new())
}
