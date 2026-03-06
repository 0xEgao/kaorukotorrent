use serde::{Deserialize, Serialize};

use crate::metadata::ItemMetadata;

#[derive(Deserialize)]
pub struct PublishOfferRequest {
    pub item: String,
    pub item_path: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct MarketOffer {
    pub address: String,
    pub item: String,
    pub item_info: String,
    pub item_size: u64,
    pub version: f32,
}

#[derive(Serialize, Clone)]
pub struct OfferAnnouncement {
    pub offer_id: String,
    pub offer: MarketOffer,
    pub metadata: ItemMetadata,
}

#[derive(Serialize)]
pub struct PublishOfferResponse {
    pub offer_id: String,
    pub offer: MarketOffer,
    pub metadata: ItemMetadata,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub message: String,
}
