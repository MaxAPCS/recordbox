use crate::{autotag::MetadataSource, sync, util};
use axum::{Router, extract, routing};
use std::sync::Arc;

pub async fn serve(configuration: util::Configuration) {
    let router = Router::new()
        .route("/health", routing::get(async || "Working!"))
        .route("/trackadd", routing::post(trackadd))
        .route("/trackls", routing::get(trackls))
        .route("/track/{provider}/{id}", routing::delete(trackrm))
        .route("/track/{provider}/{id}", routing::get(trackinfo))
        .route("/track/{provider}/{id}", routing::patch(trackedit))
        .route("/track/{provider}/{id}/autotag", routing::get(trackautotag))
        .with_state(Arc::new(configuration));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:4000").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

async fn trackadd(
    extract::State(cfg): extract::State<Arc<util::Configuration>>,
    extract::Json(tracks): extract::Json<Vec<sync::Track>>,
) -> axum::response::Result<()> {
    let downloads = cfg.get_directory("downloads")?;
    for track in tracks {
        sync::track_download(track, downloads.as_path()).await?
    }
    Ok(())
}

async fn trackls(
    extract::State(cfg): extract::State<Arc<util::Configuration>>,
) -> axum::response::Result<extract::Json<Vec<sync::Track>>> {
    Ok(extract::Json(sync::track_list(
        cfg.get_directory("downloads")?.as_path(),
    )?))
}

async fn trackrm(
    extract::State(cfg): extract::State<Arc<util::Configuration>>,
    extract::Path(track): extract::Path<sync::Track>,
) -> axum::response::Result<()> {
    Ok(sync::track_delete(
        track,
        cfg.get_directory("downloads")?.as_path(),
    )?)
}

async fn trackinfo(
    extract::State(cfg): extract::State<Arc<util::Configuration>>,
    extract::Path(track): extract::Path<sync::Track>,
) -> axum::response::Result<extract::Json<util::Metadata>> {
    Ok(extract::Json(
        sync::track_info(&track, cfg.get_directory("downloads")?.as_path())?.into(),
    ))
}

async fn trackedit(
    extract::State(cfg): extract::State<Arc<util::Configuration>>,
    extract::Path(track): extract::Path<sync::Track>,
    extract::Json(meta): extract::Json<util::Metadata>,
) -> axum::response::Result<()> {
    Ok(sync::track_edit(
        track,
        cfg.get_directory("downloads")?.as_path(),
        meta,
    )?)
}

async fn trackautotag(
    extract::State(cfg): extract::State<Arc<util::Configuration>>,
    extract::Path(track): extract::Path<sync::Track>,
) -> axum::response::Result<extract::Json<Vec<util::Metadata>>> {
    let meta = sync::track_info(&track, cfg.get_directory("downloads")?.as_path())?.into();
    Ok(extract::Json(cfg.metadatasources.get_track(meta).await?))
}
