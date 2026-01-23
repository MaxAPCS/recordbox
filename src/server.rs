use crate::{config, sync};
use axum::{
    Router, extract,
    http::{StatusCode, Uri},
    routing,
};
use std::sync::Arc;

pub async fn serve(configuration: config::Configuration) {
    let router = Router::new()
        .route("/health", routing::get(async || "Working!"))
        .route("/addtrack", routing::post(route_addtrack))
        .with_state(Arc::new(configuration));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:4000").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

#[derive(serde::Deserialize)]
struct AddTrack {
    provider: String,
    id: String,
}

async fn route_addtrack(
    extract::State(state): extract::State<Arc<config::Configuration>>,
    extract::Json(payload): extract::Json<AddTrack>,
) -> axum::response::Result<()> {
    let uri = match payload.provider.to_lowercase().as_str() {
        "youtube" => Uri::builder()
            .authority("youtube.com")
            .scheme("https")
            .path_and_query(format!("/watch?v={}", payload.id))
            .build()
            .unwrap(),
        _ => return Err(StatusCode::BAD_REQUEST.into()),
    };
    Ok(sync::download_track(uri, state.get_directory("downloads")?.as_path()).await?)
}
