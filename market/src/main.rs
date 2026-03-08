use std::net::SocketAddr;

use tokio::net::TcpListener;

mod dtos;
mod routes;
mod store;

// Summary -:
// This is market module, it will act as backend server or we can say as a tracker
// which keeps record of metadata provided by different senders.
// The receiver can query about the metadata from it via GET request
// The sender can post the metadata to it via a post request
//
// Apart from that this tracker server also keeps triggering the peer acting as sender to ensure
// they are live, and to keep the offerbook as updated as possible.

// LIST OF API ROUTES THIS SERVER WORKS WITH
// 1-) /api/listoffer -> SENDER SEND POST REQUEST HERE TO LIST AN OFFER
// 2-) default route -> SERVE IT IF SOMEONE IT A WRONG ENDPOINT

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("market server failed: {}", err);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state = store::AppState::new(store::OfferStore::default());
    let app = routes::app(state);
    // How to implement cookie session in rust?
    //

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = TcpListener::bind(addr).await?;
    listener.set_ttl(34)?;

    println!("Server running on http://{}", addr);
    println!("Server TTL is : {:?}", listener.ttl());
    axum::serve(listener, app).await?;

    Ok(())
}
