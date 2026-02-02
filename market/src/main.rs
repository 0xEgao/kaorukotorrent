use std::net::SocketAddr;

use axum::{Json, Router, routing::post};
use tokio::net::TcpListener;

use crate::dtos::{Offer, OfferResponse};

mod dtos;

// This is market module, it will act as backend server or we can say as a tracker
// which keeps record of metadata provided by different peer.
// The receiver can query about the metadata from it via GET request
// The sender can post the metadata to it via a post request
//
// Apart from that this tracker server also keeps triggering the peer acting as sender to ensure
// they are live, and to keep the offerbook as updated as possible.

// LIST OF API ROUTES THIS SERVER WORKS WITH
// 1-) /api/listoffer -> SENDER SEND POST REQUEST HERE TO LIST AN OFFER
// 2-) default route -> SERVE IT IF SOMEONE IT A WRONG ENDPOINT

// We will be using Redis for caching the live offer, idk offer i mean like just key value pair of maybe the
// sender's address and it's product id.
// The if someone wants to use that particular sender, which sends request to our main database that is the
// postgress sql database.
#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/api/createoffer", post(create_offer))
        .fallback(serve_page);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listner = TcpListener::bind(addr).await.unwrap();
    axum::serve(listner, app).await.unwrap();

    print!("Server running on http://{}", addr);
}

// List an offer on the marketplace
async fn create_offer(Json(body): Json<Offer>) -> Json<OfferResponse> {
    let request = format!(
        "GOT THE OFFER FROM {}, providing {}, item size is {}, item info is {}, version : {}",
        body.address, body.item, body.item_size, body.item_info, body.version
    );

    print!("{}", request);

    let response = OfferResponse {
        address: body.address,
        item: body.item,
    };
    Json(response)
}

async fn serve_page() -> &'static str {
    "WELCOME TO KAORUKO TORRENT MARKETPLACE"
}
