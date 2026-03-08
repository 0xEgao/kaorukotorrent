use std::{
    error::Error,
    fmt,
    path::{Path, PathBuf},
};

use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Offer {
    pub address: String,
    pub item: String,
    pub item_info: String,
    pub item_size: u64,
    pub version: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemMetadata {
    pub item: String,
    pub info_hash: String,
    pub total_size: u64,
    pub files: Vec<FileEntry>,
}

#[derive(Debug)]
pub struct ReceiverError {
    message: String,
}

impl ReceiverError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ReceiverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for ReceiverError {}

pub async fn list_offers(market_base_url: &str) -> Result<Vec<Offer>, ReceiverError> {
    let url = format!("{}/api/offers", market_base_url.trim_end_matches('/'));
    let client = Client::new();

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|err| ReceiverError::new(format!("market request failed: {}", err)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "<body unreadable>".to_string());
        return Err(ReceiverError::new(format!(
            "market returned error: {} {}",
            status, body
        )));
    }

    response
        .json::<Vec<Offer>>()
        .await
        .map_err(|err| ReceiverError::new(format!("invalid market response: {}", err)))
}

pub fn parse_offer_metadata(offer: &Offer) -> Result<ItemMetadata, ReceiverError> {
    serde_json::from_str::<ItemMetadata>(&offer.item_info)
        .map_err(|err| ReceiverError::new(format!("invalid offer metadata: {}", err)))
}

pub async fn download_offer(
    offer: &Offer,
    output_dir: &Path,
) -> Result<(ItemMetadata, Vec<PathBuf>), ReceiverError> {
    let metadata = parse_offer_metadata(offer)?;

    let item_root = output_dir.join(&offer.item);
    tokio::fs::create_dir_all(&item_root)
        .await
        .map_err(|err| ReceiverError::new(format!("create output dir failed: {}", err)))?;

    let client = Client::new();
    let mut written_files = Vec::new();

    for entry in &metadata.files {
        let relative = entry.path.trim_start_matches('/');

        let download_path = if relative.is_empty() {
            offer.item.clone()
        } else {
            format!("{}/{}", offer.item.trim_end_matches('/'), relative)
        };

        let url = format!(
            "{}/api/files/{}",
            offer.address.trim_end_matches('/'),
            encode_path_preserving_slashes(&download_path)
        );

        let response = client
            .get(url)
            .send()
            .await
            .map_err(|err| ReceiverError::new(format!("sender request failed: {}", err)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "<body unreadable>".to_string());
            return Err(ReceiverError::new(format!(
                "sender returned error for {}: {} {}",
                download_path, status, body
            )));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|err| ReceiverError::new(format!("read sender response failed: {}", err)))?;

        let out_path = if relative.is_empty() {
            item_root.join(&offer.item)
        } else {
            item_root.join(relative)
        };

        if let Some(parent) = out_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|err| ReceiverError::new(format!("create parent dir failed: {}", err)))?;
        }

        tokio::fs::write(&out_path, &bytes)
            .await
            .map_err(|err| ReceiverError::new(format!("write file failed: {}", err)))?;

        written_files.push(out_path);
    }

    Ok((metadata, written_files))
}

fn encode_path_preserving_slashes(path: &str) -> String {
    // Encode each path segment, but keep the `/` separators.
    let mut out = String::new();
    for (i, segment) in path.split('/').enumerate() {
        if i > 0 {
            out.push('/');
        }
        out.push_str(&urlencoding::encode(segment));
    }
    out
}
