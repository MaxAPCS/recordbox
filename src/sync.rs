use crate::util;
use axum::http::Uri;
use mp4ameta::{ReadConfig, WriteConfig};
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

pub async fn track_download(track: Track, dst_dir: &Path) -> Result<(), String> {
    if dst_dir
        .join(track.to_string())
        .with_added_extension("m4a")
        .is_file()
    {
        return Ok(());
    }
    let uri = match track.provider.to_lowercase().as_str() {
        "youtube" => Uri::builder()
            .authority("youtube.com")
            .scheme("https")
            .path_and_query(format!("/watch?v={}", track.id))
            .build()
            .map_err(|_| "Invalid URL")?,
        _ => return Err("Invalid Provider".to_string()),
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
        Err(e) => Err(e.to_string()),
        Ok(mut child) => match child.wait().await {
            Err(e) => Err(e.to_string()),
            Ok(e) if !e.success() => Err(e.to_string()),
            Ok(_) => Ok(()),
        },
    }
}

pub fn track_list(dst_dir: &Path) -> Result<Vec<Track>, String> {
    Ok(fs::read_dir(dst_dir)
        .map_err(|e| e.to_string())?
        .map_while(|f| Track::parse(f.ok()?.path().file_prefix()?.to_str()?))
        .collect())
}

pub fn track_delete(track: Track, dst_dir: &Path) -> Result<(), String> {
    match fs::remove_file(dst_dir.join(track.to_string()).with_added_extension("m4a")) {
        Ok(_) => Ok(()),
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => Ok(()),
            std::io::ErrorKind::PermissionDenied => Err("Forbidden".to_string()),
            std::io::ErrorKind::ReadOnlyFilesystem => Err("Forbidden".to_string()),
            _ => Err(e.to_string()),
        },
    }
}

pub fn track_info(track: &Track, dst_dir: &Path) -> Result<mp4ameta::Tag, String> {
    match mp4ameta::Tag::read_with_path(
        dst_dir.join(track.to_string()).with_added_extension("m4a"),
        &ReadConfig {
            read_image_data: false,
            read_chapter_list: false,
            read_chapter_track: false,
            read_audio_info: true,
            ..Default::default()
        },
    ) {
        Ok(t) => Ok(t),
        Err(e) => Err(match e.kind {
            mp4ameta::ErrorKind::Io(err) => match err.kind() {
                std::io::ErrorKind::NotFound => "Not Found".to_string(),
                std::io::ErrorKind::PermissionDenied => "Forbidden".to_string(),
                _ => err.to_string(),
            },
            _ => "Corrupted File".to_string(),
        }),
    }
}

pub fn track_edit(track: Track, dst_dir: &Path, meta: util::Metadata) -> Result<(), String> {
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
            std::io::ErrorKind::NotFound => "Not Found".to_string(),
            std::io::ErrorKind::PermissionDenied => "Forbidden".to_string(),
            std::io::ErrorKind::ReadOnlyFilesystem => "Forbidden".to_string(),
            _ => err.to_string(),
        },
        _ => "Corrupted File".to_string(),
    })
}
