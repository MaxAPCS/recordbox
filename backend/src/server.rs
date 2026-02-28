use crate::{autotag::MetadataSource, sync, util};
use axum::{Router, extract, routing};
use static_serve::embed_assets;
use std::sync::Arc;

embed_assets!(
    "frontend/pkg",
    compress = true,
    strip_html_ext = true,
    ignore_paths = [".gitignore"],
    cache_busted_paths = ["index.html", "favicon.svg"]
);

pub async fn serve(configuration: util::Configuration) {
    let address = configuration.address().unwrap();
    let router = Router::new()
        .merge(static_router())
        .route("/health", routing::get(async || "Working!"))
        .route("/trackadd", routing::post(trackadd))
        .route("/tracks", routing::get(trackls))
        .route("/track/{id}", routing::delete(trackrm))
        .route("/track/{id}", routing::get(trackinfo))
        .route("/track/{id}", routing::put(trackedit))
        .route("/track/{id}", routing::patch(trackpatch))
        .route("/track/{id}/autotag", routing::get(trackautotag))
        .with_state(Arc::new(configuration));
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

async fn trackadd(
    extract::State(cfg): extract::State<Arc<util::Configuration>>,
    extract::Json(tracks): extract::Json<Vec<String>>,
) -> axum::response::Result<()> {
    let Ok(tracks) = tracks
        .iter()
        .map(|s| axum::http::Uri::try_from(s))
        .collect::<Result<Vec<_>, _>>()
    else {
        return Err((reqwest::StatusCode::BAD_REQUEST, "Invalid URL(s)").into());
    };
    let library = cfg.get_library()?;
    for track in tracks {
        sync::track_download(track, library.as_path()).await?
    }
    Ok(())
}

async fn trackls(
    extract::State(cfg): extract::State<Arc<util::Configuration>>,
) -> axum::response::Result<extract::Json<Vec<String>>> {
    Ok(extract::Json(sync::track_list(
        cfg.get_library()?.as_path(),
    )?))
}

async fn trackrm(
    extract::State(cfg): extract::State<Arc<util::Configuration>>,
    extract::Path(track): extract::Path<String>,
) -> axum::response::Result<()> {
    Ok(sync::track_delete(&track, cfg.get_library()?.as_path())?)
}

async fn trackinfo(
    extract::State(cfg): extract::State<Arc<util::Configuration>>,
    extract::Path(track): extract::Path<String>,
) -> axum::response::Result<extract::Json<util::Metadata>> {
    Ok(extract::Json(
        sync::track_info(&track, cfg.get_library()?.as_path())?.into(),
    ))
}

async fn trackedit(
    extract::State(cfg): extract::State<Arc<util::Configuration>>,
    extract::Path(track): extract::Path<String>,
    extract::Json(meta): extract::Json<util::Metadata>,
) -> axum::response::Result<()> {
    Ok(sync::track_edit(
        &track,
        cfg.get_library()?.as_path(),
        meta,
        false,
    )?)
}
async fn trackpatch(
    extract::State(cfg): extract::State<Arc<util::Configuration>>,
    extract::Path(track): extract::Path<String>,
    extract::Json(meta): extract::Json<util::Metadata>,
) -> axum::response::Result<()> {
    Ok(sync::track_edit(
        &track,
        cfg.get_library()?.as_path(),
        meta,
        true,
    )?)
}

async fn trackautotag(
    extract::State(cfg): extract::State<Arc<util::Configuration>>,
    extract::Path(track): extract::Path<String>,
) -> axum::response::Result<extract::Json<Vec<util::Metadata>>> {
    let meta = sync::track_info(&track, cfg.get_library()?.as_path())?.into();
    Ok(extract::Json(
        cfg.metadatasources
            .get_track(&meta, meta.isrc.is_none())
            .await?,
    ))
}
