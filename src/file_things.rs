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

use bytes::BytesMut;
use ct_codecs::{Base64UrlSafeNoPadding, Decoder, Encoder};
use once_cell::sync::OnceCell;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

struct ContentHMACKey {
    key: [u8; blake3::KEY_LEN],
}

static CONTENT_HMAC_KEY: OnceCell<ContentHMACKey> = OnceCell::new();
static UPLOAD_PATH: OnceCell<PathBuf> = OnceCell::new();

pub fn hash_base64_url_safe_no_padding(b64: &str) -> Result<String, String> {
    let input_bytes =
        Base64UrlSafeNoPadding::decode_to_vec(b64, None).map_err(|e| format!("{}", e))?;
    hash_bytes(&input_bytes)
}

pub fn hash_bytes(input_bytes: &[u8]) -> Result<String, String> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(input_bytes);
    let hash = hasher.finalize();
    let hash_bytes = hash.as_bytes();
    let content_hash =
        Base64UrlSafeNoPadding::encode_to_string(&hash_bytes).map_err(|e| format!("{}", e))?;
    Ok(content_hash)
}

fn load_content_hmac_key() -> Result<ContentHMACKey, String> {
    let input = std::env::var("CONTENT_HMAC_KEY").map_err(|e| format!("{}", e))?;
    if input.len() == 64 {
        let decoded = hex::decode(input).map_err(|e| format!("{}", e))?;
        let mut key: [u8; blake3::KEY_LEN] = [0; blake3::KEY_LEN];
        key.copy_from_slice(&decoded[..32]);
        println!("Loaded CONTENT_HMAC_KEY");
        Ok(ContentHMACKey { key })
    } else {
        // It must be hashed
        println!("Input CONTENT_HMAC_KEY is being hashed, this is not recommended, it should be a 32 byte sequence encoded as 64 hex characters");
        let mut hasher = blake3::Hasher::new();
        hasher.update(input.as_bytes());
        let hash = hasher.finalize();
        let key = *hash.as_bytes();
        Ok(ContentHMACKey { key })
    }
}

pub async fn hash_file(path: &Path) -> Result<String, String> {
    let mut open_file = File::open(path).await.map_err(|err| format!("{:?}", err))?;
    let mut buffer = BytesMut::with_capacity(128);
    let key_wrapper = CONTENT_HMAC_KEY.get_or_try_init(load_content_hmac_key)?;
    let mut hasher = blake3::Hasher::new_keyed(&key_wrapper.key);
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
    UPLOAD_PATH
        .get_or_try_init(internal_upload_path)
        .map(|p| p.clone())
}

fn internal_upload_path() -> Result<PathBuf, String> {
    let path = std::env::var("UPLOAD_PATH").unwrap_or_else(|_| {
        println!("Warning UPLOAD_PATH is not set, will use ./files");
        "./files".to_string()
    });
    create_dir_all(&path).map_err(|err| format!("{}", err))?;
    println!("Directory {} exists now", path);
    let absolute_path = Path::new(&path)
        .canonicalize()
        .map_err(|err| format!("{}", err))?;
    Ok(absolute_path)
}
