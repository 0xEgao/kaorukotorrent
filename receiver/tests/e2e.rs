use std::{
    future::IntoFuture,
    net::SocketAddr,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::Router;
use reqwest::Client;
use tokio::{net::TcpListener, sync::oneshot, time::Duration};

fn make_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should move forward")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("{}_{}", prefix, nanos));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

async fn serve(app: Router) -> (SocketAddr, oneshot::Sender<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("local addr");

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    tokio::spawn(async move {
        let server = axum::serve(listener, app).with_graceful_shutdown(async {
            let _ = shutdown_rx.await;
        });

        if let Err(err) = server.into_future().await {
            eprintln!("server error: {}", err);
        }
    });

    (addr, shutdown_tx)
}

async fn wait_for_ok(url: &str) {
    let client = Client::new();
    for _ in 0..40 {
        match client.get(url).send().await {
            Ok(resp) if resp.status().is_success() => return,
            _ => tokio::time::sleep(Duration::from_millis(25)).await,
        }
    }

    panic!("service did not become ready: {}", url);
}

#[tokio::test]
async fn sender_market_receiver_end_to_end_downloads_data() {
    // --- Start market
    let market_state = market::store::AppState::new(market::store::OfferStore::default());
    let market_app = market::routes::app(market_state);
    let (market_addr, market_shutdown) = serve(market_app).await;
    let market_url = format!("http://{}", market_addr);
    wait_for_ok(&format!("{}/api/health", market_url)).await;

    // --- Prepare sender data on disk
    let sender_root = make_temp_dir("kaoruko_sender_data");
    let item = "sample";
    let item_dir = sender_root.join(item);
    std::fs::create_dir_all(&item_dir).expect("create item dir");

    let payload_path = item_dir.join("payload.txt");
    let payload_bytes = b"hello from sender";
    std::fs::write(&payload_path, payload_bytes).expect("write payload");

    // --- Start sender (configured to publish to our market)
    let sender_state = sender::state::AppState::new(sender::config::Config {
        bind_addr: "127.0.0.1:0".parse().expect("bind addr"),
        public_addr: "".to_string(),
        market_base_url: market_url.clone(),
        data_dir: sender_root.clone(),
        version: 1.0,
    });

    // Bind ourselves so we can know the chosen port and set public_addr accordingly.
    let sender_listener = TcpListener::bind("127.0.0.1:0").await.expect("bind sender");
    let sender_addr = sender_listener.local_addr().expect("local addr");
    let sender_url = format!("http://{}", sender_addr);

    let mut sender_state = sender_state;
    sender_state.config.bind_addr = sender_addr;
    sender_state.config.public_addr = sender_url.clone();

    let sender_app = sender::routes::app(sender_state);
    let (sender_shutdown_tx, sender_shutdown_rx) = oneshot::channel::<()>();

    tokio::spawn(async move {
        let server = axum::serve(sender_listener, sender_app).with_graceful_shutdown(async {
            let _ = sender_shutdown_rx.await;
        });
        if let Err(err) = server.into_future().await {
            eprintln!("sender server error: {}", err);
        }
    });

    wait_for_ok(&format!("{}/api/health", sender_url)).await;

    // --- Sender publishes an offer (this writes into market)
    let client = Client::new();
    let publish_resp = client
        .post(format!("{}/api/offer/publish", sender_url))
        .json(&serde_json::json!({
            "item": item,
            "item_path": null
        }))
        .send()
        .await
        .expect("publish request");

    assert!(publish_resp.status().is_success());

    // --- Receiver fetches offers and downloads bytes
    let offers = receiver::list_offers(&market_url)
        .await
        .expect("list offers");

    assert_eq!(offers.len(), 1);
    assert_eq!(offers[0].item, item);
    assert_eq!(offers[0].address, sender_url);

    let out_dir = make_temp_dir("kaoruko_receiver_out");
    let (_metadata, files) = receiver::download_offer(&offers[0], &out_dir)
        .await
        .expect("download");

    assert_eq!(files.len(), 1);

    let downloaded =
        std::fs::read(out_dir.join(item).join("payload.txt")).expect("read downloaded file");
    assert_eq!(downloaded, payload_bytes);

    // --- Shutdown
    let _ = sender_shutdown_tx.send(());
    let _ = market_shutdown.send(());
}
