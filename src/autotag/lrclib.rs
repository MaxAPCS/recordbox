use crate::{autotag::MetadataSource, util};

#[derive(Default)]
pub(crate) struct LRCLib {
    client: reqwest::Client,
}

impl MetadataSource for LRCLib {
    async fn get_track(&self, meta: crate::util::Metadata) -> Result<Vec<util::Metadata>, String> {
        let resp = self
            .client
            .get("https://lrclib.net/api/get")
            .query(&[
                (
                    "track_name",
                    meta.title.ok_or("Required Field: title")?.replace(" ", "+"),
                ),
                (
                    "artist_name",
                    meta.artists
                        .first()
                        .ok_or("Required Field: artists")?
                        .replace(" ", "+"),
                ),
            ])
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<LRCResp>()
            .await
            .map_err(|e| e.to_string())?;
        Ok(vec![util::Metadata {
            title: Some(resp.track_name),
            artists: vec![resp.artist_name],
            album: Some(resp.album_name),
            lyrics: resp.synced_lyrics.or(resp.plain_lyrics),
            ..Default::default()
        }])
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct LRCResp {
    id: isize,
    name: String,
    track_name: String,
    artist_name: String,
    album_name: String,
    duration: f32,
    instrumental: bool,
    plain_lyrics: Option<String>,
    synced_lyrics: Option<String>,
}
