use std::net::SocketAddr;

use tokio::net::TcpListener;

mod config;
mod dtos;
mod market;
mod metadata;
mod routes;
mod state;

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("sender failed: {}", err);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = config::Config::from_env()?;
    let addr: SocketAddr = config.bind_addr;
    let state = state::AppState::new(config);
    let app = routes::app(state);

    let listener = TcpListener::bind(addr).await?;
    println!("Sender running on http://{}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}
