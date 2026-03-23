use crate::util;
use axum::http::Uri;
use mp4ameta::{ReadConfig, WriteConfig};
use reqwest::StatusCode;
use std::{fs, path::Path};

pub async fn track_download(url: Uri, dst_dir: &Path) -> Result<(), (StatusCode, String)> {
    let cmd = tokio::process::Command::new("yt-dlp")
        .current_dir(dst_dir)
        .args([
            url.to_string().as_str(),
            "--quiet",
            "--ignore-config",
            "--no-playlist",
            "--no-overwrites",
            "-o",
            "%(extractor)s_%(id)s.%(ext)s",
            "-f",
            "m4a/bestaudio/best",
            "-x",
            "--audio-quality",
            "0",
            "--audio-format",
            "m4a",
            "--embed-metadata",
            "--parse-metadata",
            "%(release_date,upload_date|)s:%(meta_date)s",
            "--parse-metadata",
            "%(artists|)+l:%(meta_artist)s",
            "--embed-thumbnail",
            "--ppa",
            "ffmpeg: -c:v mjpeg -vf crop=\"'if(gt(ih,iw),iw,ih)':'if(gt(iw,ih),ih,iw)'\"",
        ])
        .env_clear()
        .stdout(std::process::Stdio::null())
        .spawn();
    match cmd {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err((
            StatusCode::METHOD_NOT_ALLOWED,
            "Track Download requires yt-dlp".to_string(),
        )),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
        Ok(mut child) => match child.wait().await {
            Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
            Ok(e) if !e.success() => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
            Ok(_) => Ok(()),
        },
    }
}

pub fn track_list(dst_dir: &Path) -> Result<Vec<String>, (StatusCode, String)> {
    Ok(fs::read_dir(dst_dir)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .filter_map(|f| {
            let Ok(file) = f else {
                return None;
            };
            let file = file.path();
            if !file.is_file() {
                return None;
            }
            let Some(ext) = file.extension() else {
                return None;
            };
            if !ext.eq_ignore_ascii_case("m4a") {
                return None;
            }
            let Some(path) = file.file_prefix().map(|fp| fp.to_str()).flatten() else {
                return Some(Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Track Path Invalid".to_string(),
                )));
            };
            if path.starts_with(".") {
                return None;
            }
            Some(Ok(path.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?)
}

pub fn track_delete(track: &str, dst_dir: &Path) -> Result<(), (StatusCode, String)> {
    fs::remove_file(dst_dir.join(track).with_added_extension("m4a")).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => (
            StatusCode::NOT_MODIFIED,
            "Track Already Deleted".to_string(),
        ),
        std::io::ErrorKind::PermissionDenied => (
            StatusCode::FORBIDDEN,
            "Filesystem Permission Denied".to_string(),
        ),
        std::io::ErrorKind::ReadOnlyFilesystem => (
            StatusCode::FORBIDDEN,
            "Filesystem Read-Only".to_string(), //
        ),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    })
}

pub fn track_info(track: &str, dst_dir: &Path) -> Result<mp4ameta::Tag, (StatusCode, String)> {
    mp4ameta::Tag::read_with_path(
        dst_dir.join(track).with_added_extension("m4a"),
        &ReadConfig {
            read_image_data: false,
            read_chapter_list: false,
            read_chapter_track: false,
            read_audio_info: true,
            ..Default::default()
        },
    )
    .map_err(|e| match e.kind {
        mp4ameta::ErrorKind::Io(err) => match err.kind() {
            std::io::ErrorKind::NotFound => (StatusCode::NOT_FOUND, "Track Not Found".to_string()),
            std::io::ErrorKind::PermissionDenied => (
                StatusCode::FORBIDDEN,
                "Filesystem Permission Denied".to_string(),
            ),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        },
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Corrupted File".to_string(),
        ),
    })
}

pub fn track_edit(
    track: &str,
    dst_dir: &Path,
    meta: util::Metadata,
    patch: bool,
) -> Result<(), (StatusCode, String)> {
    let mut tag = track_info(track, dst_dir)?;
    if patch {
        meta.apply(&mut tag);
    } else {
        meta.write(&mut tag);
    }
    tag.write_with_path(
        dst_dir.join(track).with_added_extension("m4a"),
        &WriteConfig {
            write_chapter_list: false,
            write_chapter_track: false,
            ..Default::default()
        },
    )
    .map_err(|e| match e.kind {
        mp4ameta::ErrorKind::Io(err) => match err.kind() {
            std::io::ErrorKind::NotFound => (StatusCode::NOT_FOUND, "Track Not Found".to_string()),
            std::io::ErrorKind::PermissionDenied => (
                StatusCode::FORBIDDEN,
                "Filesystem Permission Denied".to_string(),
            ),
            std::io::ErrorKind::ReadOnlyFilesystem => (
                StatusCode::FORBIDDEN,
                "Filesystem Read-Only".to_string(), //
            ),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        },
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Corrupted File".to_string(),
        ),
    })
}
