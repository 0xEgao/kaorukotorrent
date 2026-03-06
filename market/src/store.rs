use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use crate::dtos::Offer;

// TODO-: Suppose the market goes down, then all the data will be lost man, so write the offer on the disk.
// A frontend or tui for showcasing different peers live on a market in different forms.

#[derive(Clone)]
pub struct AppState {
    store: OfferStore,
}

impl AppState {
    pub fn new(store: OfferStore) -> Self {
        Self { store }
    }

    pub fn store(&self) -> &OfferStore {
        &self.store
    }
}

#[derive(Clone, Default)]
pub struct OfferStore {
    inner: Arc<RwLock<HashMap<String, Offer>>>,
}

impl OfferStore {
    pub async fn upsert(&self, offer: Offer) -> String {
        let key = offer_key(&offer);
        let mut guard = self.inner.write().await;
        guard.insert(key.clone(), offer);
        key
    }

    pub async fn list(&self) -> Vec<Offer> {
        let guard = self.inner.read().await;
        guard.values().cloned().collect()
    }

    pub async fn delete_by_key(&self, key: &str) -> bool {
        let mut guard = self.inner.write().await;
        guard.remove(key).is_some()
    }
}

fn offer_key(offer: &Offer) -> String {
    format!("{}::{}", offer.address, offer.item)
}

pub fn offer_key_from_parts(address: &str, item: &str) -> String {
    format!("{}::{}", address, item)
}
