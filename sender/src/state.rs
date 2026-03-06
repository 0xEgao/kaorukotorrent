use std::sync::Arc;

use tokio::sync::{broadcast, RwLock};

use crate::{config::Config, dtos::OfferAnnouncement};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    announcer: broadcast::Sender<OfferAnnouncement>,
    latest: Arc<RwLock<Option<OfferAnnouncement>>>,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let (announcer, _) = broadcast::channel(64);
        Self {
            config,
            announcer,
            latest: Arc::new(RwLock::new(None)),
        }
    }

    pub fn announcer(&self) -> broadcast::Sender<OfferAnnouncement> {
        self.announcer.clone()
    }

    pub async fn set_latest(&self, offer: OfferAnnouncement) {
        let mut guard = self.latest.write().await;
        *guard = Some(offer);
    }

    pub async fn latest(&self) -> Option<OfferAnnouncement> {
        let guard = self.latest.read().await;
        guard.clone()
    }
}
