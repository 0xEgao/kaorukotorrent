use std::time::Duration;

use axum::{
    Json, Router,
    body::Body,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response, sse::{Event, KeepAlive, Sse}},
    routing::{get, post},
};
use serde_json::json;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use tokio_util::io::ReaderStream;

use crate::{
    dtos::{ErrorResponse, MarketOffer, OfferAnnouncement, PublishOfferRequest, PublishOfferResponse},
    market,
    metadata,
    state::AppState,
};

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/api/offer/publish", post(publish_offer))
        .route("/api/offer/latest", get(latest_offer))
        .route("/api/offer/subscribe", get(subscribe_offers))
        .route("/api/files/*path", get(download_file))
        .route("/api/health", get(health))
        .with_state(state)
}

async fn publish_offer(
    State(state): State<AppState>,
    Json(body): Json<PublishOfferRequest>,
) -> Result<Json<PublishOfferResponse>, AppError> {
    let item_path = body.item_path.as_deref().unwrap_or(&body.item);
    let full_path = metadata::resolve_item_path(&state.config.data_dir, item_path)?;

    let item_metadata = metadata::build_metadata(&body.item, &full_path)?;
    let item_info = serde_json::to_string(&item_metadata)
        .map_err(|err| AppError::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    let offer = MarketOffer {
        address: state.config.public_addr.clone(),
        item: body.item,
        item_info,
        item_size: item_metadata.total_size,
        version: state.config.version,
    };

    let offer_id = market::publish_offer(&state.config.market_base_url, &offer)
        .await
        .map_err(|err| AppError::new(StatusCode::BAD_GATEWAY, err.to_string()))?;

    let announcement = OfferAnnouncement {
        offer_id: offer_id.clone(),
        offer: offer.clone(),
        metadata: item_metadata.clone(),
    };

    let _ = state.announcer().send(announcement.clone());
    state.set_latest(announcement.clone()).await;

    Ok(Json(PublishOfferResponse {
        offer_id,
        offer: announcement.offer,
        metadata: announcement.metadata,
    }))
}

async fn latest_offer(State(state): State<AppState>) -> Result<Json<OfferAnnouncement>, AppError> {
    match state.latest().await {
        Some(offer) => Ok(Json(offer)),
        None => Err(AppError::new(StatusCode::NOT_FOUND, "no offers yet")),
    }
}

async fn subscribe_offers(
    State(state): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, std::convert::Infallible>>> {
    let receiver = state.announcer().subscribe();
    let stream = BroadcastStream::new(receiver).filter_map(|message| {
        match message {
            Ok(offer) => {
                let payload = serde_json::to_string(&offer)
                    .unwrap_or_else(|_| json!({"error": "serialization failed"}).to_string());
                Some(Ok(Event::default().data(payload)))
            }
            Err(_) => None,
        }
    });

    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)).text("keep-alive"))
}

async fn download_file(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> Result<Response, AppError> {
    let clean = metadata::sanitize_relative_path(&path)?;
    let full_path = state.config.data_dir.join(clean);
    let metadata = tokio::fs::metadata(&full_path)
        .await
        .map_err(|_| AppError::new(StatusCode::NOT_FOUND, "file not found"))?;

    if !metadata.is_file() {
        return Err(AppError::new(StatusCode::BAD_REQUEST, "not a file"));
    }

    let file = tokio::fs::File::open(&full_path)
        .await
        .map_err(|_| AppError::new(StatusCode::NOT_FOUND, "file not found"))?;
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);
    Ok(Response::new(body))
}

async fn health() -> &'static str {
    "OK"
}

#[derive(Debug)]
struct AppError {
    status: StatusCode,
    message: String,
}

impl AppError {
    fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = Json(ErrorResponse {
            message: self.message,
        });
        (self.status, body).into_response()
    }
}

impl From<metadata::MetadataError> for AppError {
    fn from(err: metadata::MetadataError) -> Self {
        AppError::new(StatusCode::BAD_REQUEST, err.to_string())
    }
}
