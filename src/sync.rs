use crate::util;
use axum::http::Uri;
use mp4ameta::{ReadConfig, WriteConfig};
use reqwest::StatusCode;
use std::{fs, path::Path};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Track {
    pub(crate) provider: String,
    pub(crate) id: String,
}

impl Track {
    fn parse(str: &str) -> Option<Self> {
        let (p, i) = str.split_once("_")?;
        Some(Self {
            provider: p.to_string(),
            id: i.to_string(),
        })
    }
}

impl std::fmt::Display for Track {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}_{}", self.provider, self.id)
    }
}

pub async fn track_download(track: Track, dst_dir: &Path) -> Result<(), (StatusCode, String)> {
    if dst_dir
        .join(track.to_string())
        .with_added_extension("m4a")
        .is_file()
    {
        return Err((
            StatusCode::NOT_MODIFIED,
            "Track Already Downloaded".to_string(),
        ));
    }
    let uri = match track.provider.to_lowercase().as_str() {
        "youtube" => Uri::builder()
            .authority("youtube.com")
            .scheme("https")
            .path_and_query(format!("/watch?v={}", track.id))
            .build()
            .or(Err((
                StatusCode::BAD_REQUEST,
                "Invalid Track ID".to_string(),
            )))?,
        _ => {
            return Err((StatusCode::BAD_REQUEST, "Invalid Provider".to_string()));
        }
    };
    let cmd = tokio::process::Command::new("yt-dlp")
        .current_dir(dst_dir)
        .args([
            uri.to_string().as_str(),
            "--ignore-config",
            "--no-overwrites",
            "-o",
            track
                .to_string()
                .replace(std::path::MAIN_SEPARATOR_STR, "")
                .replace(".", "")
                .as_str(),
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
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
        Ok(mut child) => match child.wait().await {
            Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
            Ok(e) if !e.success() => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
            Ok(_) => Ok(()),
        },
    }
}

pub fn track_list(dst_dir: &Path) -> Result<Vec<Track>, (StatusCode, String)> {
    Ok(fs::read_dir(dst_dir)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .filter_map(|f| {
            let Ok(file) = f else {
                return None;
            };
            let file = file.path();
            if file.starts_with(".") {
                return None;
            }
            let Some(path) = file.file_prefix().map(|fp| fp.to_str()).flatten() else {
                return Some(Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Track Path Invalid".to_string(),
                )));
            };
            Some(Track::parse(path).ok_or((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!(
                    "Track Parse Failed. Got: {}, Correct Format: {{provider}}_{{id}}.m4a",
                    path
                ),
            )))
        })
        .collect::<Result<Vec<_>, _>>()?)
}

pub fn track_delete(track: Track, dst_dir: &Path) -> Result<(), (StatusCode, String)> {
    fs::remove_file(dst_dir.join(track.to_string()).with_added_extension("m4a")).map_err(
        |e| match e.kind() {
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
        },
    )
}

pub fn track_info(track: &Track, dst_dir: &Path) -> Result<mp4ameta::Tag, (StatusCode, String)> {
    mp4ameta::Tag::read_with_path(
        dst_dir.join(track.to_string()).with_added_extension("m4a"),
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
    track: Track,
    dst_dir: &Path,
    meta: util::Metadata,
) -> Result<(), (StatusCode, String)> {
    let mut tag = track_info(&track, dst_dir)?;
    meta.apply(&mut tag);
    tag.write_with_path(
        dst_dir.join(track.to_string()).with_added_extension("m4a"),
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
