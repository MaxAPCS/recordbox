use crate::{autotag::MetadataSource, util};

#[derive(Default)]
pub(super) struct Deezer {
    client: reqwest::Client,
}

impl Deezer {
    fn rmparens(input: &str) -> String {
        let mut result = String::new();
        let mut segment = String::new();
        let mut depth = 0;

        for ch in input.chars() {
            match ch {
                '(' | '[' => {
                    if depth == 0 {
                        result.push_str(&segment);
                        segment.clear();
                    }
                    depth += 1;
                }
                ')' | ']' => depth -= 1,
                _ => {
                    if depth == 0 {
                        segment.push(ch);
                    }
                }
            }
        }
        result.push_str(&segment);

        result.trim().to_string()
    }
}

impl MetadataSource for Deezer {
    async fn get_track(
        &self,
        meta: &util::Metadata,
        fuzzy: bool,
    ) -> Result<Vec<util::Metadata>, String> {
        let query = if fuzzy {
            if let Some(isrc) = &meta.isrc {
                format!("isrc: \"{}\"", isrc)
            } else if let Some(title) = &meta.title {
                let title = Deezer::rmparens(title);
                if let Some(artist) = meta.artists.first() {
                    format!("{} - {}", artist, title)
                } else {
                    title
                }
            } else {
                "".to_string()
            }
        } else {
            let mut query = Vec::with_capacity(3);
            if let Some(title) = &meta.title {
                query.push(("track", title));
            }
            if let Some(artist) = meta.artists.first() {
                query.push(("artist", artist));
            }
            if let Some(isrc) = &meta.isrc {
                query.push(("isrc", isrc));
            }
            query
                .into_iter()
                .map(|(k, v)| format!("{}:\"{}\"", k, v))
                .collect::<Vec<String>>()
                .join(" ")
        };
        if query.is_empty() {
            return Err("Required Fields: title or isrc".to_string());
        }

        let resp = self
            .client
            .get("https://api.deezer.com/search/track")
            .query(&[("q", query)])
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<DeezerResp>()
            .await
            .map_err(|e| e.to_string())?;
        Ok(resp
            .data
            .into_iter()
            .map(|t| util::Metadata {
                title: Some(t.title_short),
                artists: vec![t.artist.name],
                album: Some(t.album.title),
                isrc: Some(t.isrc),
                ..Default::default()
            })
            .collect())
    }
}

#[derive(serde::Deserialize)]
#[allow(unused)]
struct DeezerResp {
    data: Vec<DeezerTrack>,
    total: i64,
    next: Option<String>,
}

#[derive(serde::Deserialize)]
#[allow(unused)]
struct DeezerTrack {
    id: i64,
    readable: bool,
    title: String,
    #[serde(rename = "title_short")]
    title_short: String,
    #[serde(rename = "title_version")]
    title_version: String,
    isrc: String,
    link: String,
    duration: i64,
    rank: i64,
    #[serde(rename = "explicit_lyrics")]
    explicit_lyrics: bool,
    #[serde(rename = "explicit_content_lyrics")]
    explicit_content_lyrics: i64,
    #[serde(rename = "explicit_content_cover")]
    explicit_content_cover: i64,
    preview: String,
    #[serde(rename = "md5_image")]
    md5_image: String,
    artist: Artist,
    album: Album,
    #[serde(rename = "type")]
    track_type: String,
}

#[derive(serde::Deserialize)]
#[allow(unused)]
struct Artist {
    id: i64,
    name: String,
    link: String,
    picture: String,
    #[serde(rename = "picture_small")]
    picture_small: String,
    #[serde(rename = "picture_medium")]
    picture_medium: String,
    #[serde(rename = "picture_big")]
    picture_big: String,
    #[serde(rename = "picture_xl")]
    picture_xl: String,
    tracklist: String,
    #[serde(rename = "type")]
    artist_type: String,
}

#[derive(serde::Deserialize)]
#[allow(unused)]
struct Album {
    id: i64,
    title: String,
    cover: String,
    #[serde(rename = "cover_small")]
    cover_small: String,
    #[serde(rename = "cover_medium")]
    cover_medium: String,
    #[serde(rename = "cover_big")]
    cover_big: String,
    #[serde(rename = "cover_xl")]
    cover_xl: String,
    #[serde(rename = "md5_image")]
    md5_image: String,
    tracklist: String,
    #[serde(rename = "type")]
    album_type: String,
}
