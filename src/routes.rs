use axum::http::Method;
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use tracing::{debug, info};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

use crate::api::{
    HostTime, LongestSession, compute_time_per_host, get_all_locations,
    get_longest_session_per_host,
};
use crate::appstate::AppState;

async fn longest_session_handler(
    Path(user): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<LongestSession>>, String> {
    info!("GET /users/{user}/longest");

    let token = state.get_token().await.map_err(|e| e.to_string())?;

    let locations = get_all_locations(&state, &token, &user)
        .await
        .map_err(|e| e.to_string())?;

    let computed = get_longest_session_per_host(&locations);

    Ok(Json(computed))
}

async fn time_per_host_handler(
    Path(user): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<HostTime>>, String> {
    info!("GET /users/{user}/total");

    let token = state.get_token().await.map_err(|e| e.to_string())?;

    let locations = get_all_locations(&state, &token, &user)
        .await
        .map_err(|e| e.to_string())?;

    let computed = compute_time_per_host(&locations);

    Ok(Json(computed))
}

pub async fn run_server(state: Arc<AppState>) {
    let cors = CorsLayer::new()
        .allow_origin(Any) // allow all origins (dev only!)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);

    let app = Router::new()
        .route("/users/{id}/total", get(time_per_host_handler))
        .route("/users/{id}/longest", get(longest_session_handler))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:15000")
        .await
        .unwrap();

    info!("Server running on http://localhost:15000");
    axum::serve(listener, app).await.unwrap();
}
