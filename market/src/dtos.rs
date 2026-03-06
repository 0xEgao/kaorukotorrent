// This file contains all the data transfer objects

// This represents an Offer message which the sender sends to list it's offer, that what it's is providing
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct Offer {
    pub address: String,
    pub item: String,
    pub item_info: String,
    pub item_size: u64,
    pub version: f32,
}

#[derive(Serialize)]
pub struct CreateOfferResponse {
    pub offer_id: String,
}

#[derive(Deserialize)]
pub struct DeleteOfferRequest {
    pub address: String,
    pub item: String,
}

#[derive(Serialize)]
pub struct DeleteOfferResponse {
    pub deleted: bool,
}
