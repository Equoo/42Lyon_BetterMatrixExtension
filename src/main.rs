use std::sync::Arc;

mod api;
mod appstate;
mod routes;
mod data;

use tracing::Level;

use crate::appstate::AppState;
use crate::routes::run_server;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    let state = Arc::new(AppState::new().await);
    run_server(state).await
}
