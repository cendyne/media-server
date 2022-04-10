use bytes::BytesMut;
use ct_codecs::{Base64UrlSafeNoPadding, Decoder, Encoder};
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

pub fn hash_base64_url_safe_no_padding(b64: &str) -> Result<String, String> {
    let input_bytes =
        Base64UrlSafeNoPadding::decode_to_vec(b64, None).map_err(|e| format!("{}", e))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(&input_bytes);
    let hash = hasher.finalize();
    let hash_bytes = hash.as_bytes();
    let content_hash =
        Base64UrlSafeNoPadding::encode_to_string(&hash_bytes).map_err(|e| format!("{}", e))?;
    Ok(content_hash)
}

pub async fn hash_file(path: &Path) -> Result<String, String> {
    let mut open_file = File::open(path).await.map_err(|err| format!("{:?}", err))?;
    let mut buffer = BytesMut::with_capacity(128);
    let mut key: [u8; blake3::KEY_LEN] = [0; blake3::KEY_LEN];
    let keystr = "todo key here".as_bytes();
    key[..keystr.len()].copy_from_slice(keystr);
    let mut hasher = blake3::Hasher::new_keyed(&key);
    let mut read_bytes = open_file
        .read_buf(&mut buffer)
        .await
        .map_err(|err| format!("{:?}", err))?;
    let mut total_bytes = 0;
    while read_bytes > 0 {
        total_bytes += read_bytes;
        hasher.update(&buffer[0..read_bytes]);
        // continue
        buffer.clear();
        read_bytes = open_file
            .read_buf(&mut buffer)
            .await
            .map_err(|err| format!("{:?}", err))?;
    }
    let hash = hasher.finalize();
    let hash_bytes = hash.as_bytes();
    let content_hash =
        Base64UrlSafeNoPadding::encode_to_string(&hash_bytes).map_err(|e| format!("{}", e))?;
    println!("Finished hashing {} bytes to {}", total_bytes, content_hash);
    Ok(content_hash)
}

pub async fn copy_temp(from_path: &Path, to_path: &Path) -> Result<(), String> {
    println!("Copying temp file from {:?} to {:?}", from_path, to_path);

    let mut from_file = File::open(from_path)
        .await
        .map_err(|err| format!("{:?}", err))?;
    println!("Opening {:?}", to_path);
    let mut to_file = File::create(to_path)
        .await
        .map_err(|err| format!("{:?}", err))?;
    let mut buffer = BytesMut::with_capacity(1024);
    let mut read_bytes = from_file
        .read_buf(&mut buffer)
        .await
        .map_err(|err| format!("{:?}", err))?;
    to_file
        .write(&buffer[0..read_bytes])
        .await
        .map_err(|err| format!("{:?}", err))?;
    println!("Wrote first chunk {}", read_bytes);

    let mut total_bytes = 0;
    while read_bytes > 0 {
        buffer.clear();
        total_bytes += read_bytes;
        // continue
        read_bytes = from_file
            .read_buf(&mut buffer)
            .await
            .map_err(|err| format!("{:?}", err))?;
        to_file
            .write(&buffer[0..read_bytes])
            .await
            .map_err(|err| format!("{:?}", err))?;
    }
    to_file.flush().await.map_err(|err| format!("{:?}", err))?;
    println!("Done writing {} bytes", total_bytes);
    Ok(())
}

pub fn upload_path() -> Result<PathBuf, String> {
    // TODO cache
    let path = std::env::var("UPLOAD_PATH").unwrap_or_else(|_| "./files".to_string());
    create_dir_all(&path).map_err(|err| format!("{}", err))?;
    let absolute_path = Path::new(&path)
        .canonicalize()
        .map_err(|err| format!("{}", err))?;
    Ok(absolute_path)
}
