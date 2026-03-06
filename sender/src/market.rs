use std::{error::Error, fmt};

use reqwest::Client;

use crate::dtos::MarketOffer;

#[derive(Debug)]
pub struct MarketError {
    message: String,
}

impl MarketError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for MarketError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for MarketError {}

#[derive(serde::Deserialize)]
struct CreateOfferResponse {
    offer_id: String,
}

pub async fn publish_offer(
    market_base_url: &str,
    offer: &MarketOffer,
) -> Result<String, MarketError> {
    let url = format!("{}/api/offers", market_base_url.trim_end_matches('/'));

    let client = Client::new();
    let response = client
        .post(url)
        .json(offer)
        .send()
        .await
        .map_err(|err| MarketError::new(format!("market request failed: {}", err)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "<body unreadable>".to_string());
        return Err(MarketError::new(format!(
            "market rejected offer: {} {}",
            status, body
        )));
    }

    let payload = response
        .json::<CreateOfferResponse>()
        .await
        .map_err(|err| MarketError::new(format!("invalid market response: {}", err)))?;

    Ok(payload.offer_id)
}
