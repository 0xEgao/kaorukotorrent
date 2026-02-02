// This file contains all the data transfer objects

// This represents an Offer message which the sender sends to list it's offer, that what it's is providing
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Offer {
    pub address: String,
    pub item: String,
    pub item_info: String,
    pub item_size: u64, // will be getting item size in MegaBYTES
    pub version: f32,
}

#[derive(Serialize)]
pub struct OfferResponse {
    pub address: String,
    pub item: String,
}
