use std::{path::Path, process::Command, task::Poll};

pub async fn download_track(src_uri: axum::http::Uri, dst_dir: &Path) -> Result<(), String> {
    let cmd = Command::new("yt-dlp")
        .current_dir(dst_dir)
        .args([
            src_uri.to_string().as_str(),
            "--ignore-config",
            "--no-overwrites",
            "-o",
            "%(id)s.%(ext)s",
            "-f",
            "m4a/bestaudio/best",
            "-x",
            "--audio-quality",
            "0",
            "--audio-format",
            "m4a",
            "--embed-metadata",
            "--parse-metadata",
            "release_date:%(meta_date)s",
            "--parse-metadata",
            "%(artists)+l:%(meta_artist)s",
            "--embed-thumbnail",
            "--ppa",
            "ffmpeg: -c:v mjpeg -vf crop=\"'if(gt(ih,iw),iw,ih)':'if(gt(iw,ih),ih,iw)'\"",
        ])
        .env_clear()
        .spawn();
    match cmd {
        Err(e) => Err(e.to_string()),
        Ok(mut child) => {
            let res = std::future::poll_fn(|_| match child.try_wait() {
                Ok(None) => Poll::Pending,
                Err(e) => Poll::Ready(Err(e)),
                Ok(Some(a)) => Poll::Ready(Ok(a)),
            })
            .await;
            match res {
                Err(e) => Err(e.to_string()),
                Ok(e) if !e.success() => Err(e.to_string()),
                Ok(_) => Ok(()),
            }
        }
    }
}
