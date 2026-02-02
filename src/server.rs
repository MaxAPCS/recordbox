use crate::{config, sync, tagser};
use axum::{Router, extract, routing};
use std::sync::Arc;

pub async fn serve(configuration: config::Configuration) {
    let router = Router::new()
        .route("/health", routing::get(async || "Working!"))
        .route("/trackadd", routing::post(trackadd))
        .route("/trackls", routing::get(trackls))
        .route("/track/{provider}/{id}", routing::delete(trackrm))
        .route("/track/{provider}/{id}", routing::get(trackinfo))
        .route("/track/{provider}/{id}", routing::put(trackedit))
        .with_state(Arc::new(configuration));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:4000").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

async fn trackadd(
    extract::State(state): extract::State<Arc<config::Configuration>>,
    extract::Json(tracks): extract::Json<Vec<sync::Track>>,
) -> axum::response::Result<()> {
    let downloads = state.get_directory("downloads")?;
    for track in tracks {
        sync::track_download(track, downloads.as_path()).await?
    }
    Ok(())
}

async fn trackls(
    extract::State(state): extract::State<Arc<config::Configuration>>,
) -> axum::response::Result<extract::Json<Vec<sync::Track>>> {
    Ok(extract::Json(sync::track_list(
        state.get_directory("downloads")?.as_path(),
    )?))
}

async fn trackrm(
    extract::State(state): extract::State<Arc<config::Configuration>>,
    extract::Path(track): extract::Path<sync::Track>,
) -> axum::response::Result<()> {
    Ok(sync::track_delete(
        track,
        state.get_directory("downloads")?.as_path(),
    )?)
}

async fn trackinfo(
    extract::State(state): extract::State<Arc<config::Configuration>>,
    extract::Path(track): extract::Path<sync::Track>,
) -> axum::response::Result<extract::Json<tagser::Mp4TagSer>> {
    Ok(extract::Json(
        sync::track_info(track, state.get_directory("downloads")?.as_path())?.into(),
    ))
}

async fn trackedit(
    extract::State(state): extract::State<Arc<config::Configuration>>,
    extract::Path(track): extract::Path<sync::Track>,
    extract::Json(tagser::Mp4TagSer { tag }): extract::Json<tagser::Mp4TagSer>,
) -> axum::response::Result<()> {
    Ok(sync::track_edit(
        track,
        state.get_directory("downloads")?.as_path(),
        tag,
    )?)
}
