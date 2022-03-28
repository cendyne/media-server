use phf::{phf_map, phf_set};
use rocket::http::{ContentType, MediaType};

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

pub const TOP_LEVEL_TYPES: phf::Map<&'static str, phf::Map<&'static str, &'static str>> = phf_map! {
    "image" => IMAGE_TYPE_EXTENSIONS,
    "audio" => AUDIO_TYPE_EXTENSIONS,
    "video" => VIDEO_TYPE_EXTENSIONS,
    "application" => APPLICATION_TYPE_EXTENSIONS,
    "text" => TEXT_TYPE_EXTENSIONS,
    "font" => FONT_TYPE_EXTENSIONS,
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
    let found = TOP_LEVEL_TYPES.get(&top[..]).and_then(|m| m.get(&sub));

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

pub fn content_type_or_from_safe_ext(ct: &ContentType, user_ext: &str) -> ContentType {
    if ct == &ContentType::Binary {
        match EXTENSION_CONTENT_TYPES.get(user_ext) {
            Some((top, sub)) => ContentType::from(MediaType::const_new(top, sub, &[])),
            None => ContentType::Binary,
        }
    } else {
        ct.clone()
    }
}
