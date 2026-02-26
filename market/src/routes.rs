use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};

use crate::{
    dtos::{CreateOfferResponse, Offer},
    store::AppState,
};

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/api/offers", post(create_offer).get(list_offers))
        .route("/api/createoffer", post(create_offer))
        .route("/api/listoffer", get(list_offers))
        .route("/api/health", get(health))
        .fallback(serve_page)
        .with_state(state)
}

async fn create_offer(
    State(state): State<AppState>,
    Json(body): Json<Offer>,
) -> (StatusCode, Json<CreateOfferResponse>) {
    let offer_id = state.store().upsert(body).await;
    let response = CreateOfferResponse { offer_id };

    (StatusCode::CREATED, Json(response))
}

async fn list_offers(State(state): State<AppState>) -> Json<Vec<Offer>> {
    let offers = state.store().list().await;
    Json(offers)
}

async fn health() -> &'static str {
    "OK"
}

async fn serve_page() -> &'static str {
    "WELCOME TO KAORUKO TORRENT MARKETPLACE"
}

#[cfg(test)]
mod tests {
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use serde_json::json;
    use tower::ServiceExt;

    use super::app;
    use crate::{
        dtos::Offer,
        store::{AppState, OfferStore},
    };

    #[tokio::test]
    async fn create_and_list_offers() {
        let state = AppState::new(OfferStore::default());
        let app = app(state);

        let payload = json!({
            "address": "tor://peer-1.onion",
            "item": "ubuntu-iso",
            "item_info": "Ubuntu 24.04 ISO",
            "item_size": 2200,
            "version": 1.0
        });

        let create_request = Request::builder()
            .method("POST")
            .uri("/api/offers")
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .expect("request build should succeed");

        let create_response = app
            .clone()
            .oneshot(create_request)
            .await
            .expect("create request should succeed");

        assert_eq!(create_response.status(), StatusCode::CREATED);

        let list_request = Request::builder()
            .method("GET")
            .uri("/api/offers")
            .body(Body::empty())
            .expect("request build should succeed");

        let list_response = app
            .oneshot(list_request)
            .await
            .expect("list request should succeed");

        assert_eq!(list_response.status(), StatusCode::OK);

        let body = to_bytes(list_response.into_body(), usize::MAX)
            .await
            .expect("body collection should succeed");
        let offers: Vec<Offer> =
            serde_json::from_slice(&body).expect("response should deserialize into offers");

        assert_eq!(offers.len(), 1);
        assert_eq!(offers[0].address, "tor://peer-1.onion");
        assert_eq!(offers[0].item, "ubuntu-iso");
    }
}
