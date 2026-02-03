use crate::{autotag::MetadataSource, util};
use musicbrainz_rs::{MusicBrainzClient, Search, entity::recording::Recording as MBRecording};

#[derive(Default)]
pub(crate) struct MusicBrainz {
    client: MusicBrainzClient,
}

impl MetadataSource for MusicBrainz {
    async fn get_track(&self, meta: util::Metadata) -> Result<Vec<util::Metadata>, String> {
        let mut query = Vec::with_capacity(3);
        if let Some(title) = &meta.title {
            query.push(("recording", title));
        }
        if let Some(artist) = meta.artists.first() {
            query.push(("artistname", artist));
        }
        // if let Some(date) = &track.date {
        //     query.push(("date", date));
        // }
        let resp = MBRecording::search(format!(
            "query={}",
            query
                .into_iter()
                .map(|(k, v)| format!("{}:\"{}\"", k, v))
                .collect::<Vec<String>>()
                .join(" AND ")
        ))
        .execute_with_client(&self.client)
        .await
        .map_err(|e| e.to_string())?;
        Ok(resp
            .entities
            .into_iter()
            .map(|f| util::Metadata {
                title: Some(f.title),
                artists: f
                    .artist_credit
                    .unwrap_or_default()
                    .into_iter()
                    .map(|a| a.name)
                    .collect(),
                album: f.releases.map(|a| Some(a.first()?.title.clone())).flatten(),
                genres: match f.tags {
                    Some(tags) => tags.iter().map(|g| g.name.clone()).collect(),
                    None => Vec::new(),
                },
                date: f.first_release_date.map(|d| d.0),
                lyrics: None,
            })
            .collect())
    }
}
