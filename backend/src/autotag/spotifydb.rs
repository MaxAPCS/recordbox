use std::borrow::Cow;

use tokio::sync::Mutex;

use crate::{autotag::MetadataSource, util};

pub(super) struct SpotifyDB {
    client: Mutex<rusqlite::Connection>,
}

impl SpotifyDB {
    pub(super) fn new(dbfile: String) -> Self {
        Self {
            client: Mutex::new(rusqlite::Connection::open(dbfile).unwrap()),
        }
    }
}

impl SpotifyDB {
    fn build_query(meta: &util::Metadata) -> (String, Vec<(String, Cow<'_, str>)>) {
        let mut where_clause = Vec::new();
        let mut named_params = Vec::new();

        if let Some(title) = &meta.title {
            where_clause.push("    tracks.name = :title".to_string());
            named_params.push((":title".to_string(), Cow::from(title)));
        }
        if !meta.artists.is_empty() {
            let mut artist_list = Vec::new();
            for (i, artist) in meta.artists.iter().enumerate() {
                let placeholder = format!(":artist_{}", i);
                named_params.push((placeholder.clone(), Cow::from(artist)));
                artist_list.push(placeholder);
            }
            where_clause.push(format!(
                "    EXISTS (
        SELECT 1
        FROM track_artists ta
        JOIN artists a ON ta.artist_rowid = a.rowid
        WHERE ta.track_rowid = tracks.rowid
            AND a.name IN ({})
    )",
                artist_list.join(", ")
            ));
        }
        if let Some(album) = &meta.album {
            where_clause.push("    albums.name = :album".to_string());
            named_params.push((":album".to_string(), Cow::from(album)));
        }
        if let Some(isrc) = &meta.isrc {
            where_clause.push("    tracks.external_id_isrc = :isrc".to_string());
            named_params.push((":isrc".to_string(), Cow::from(isrc)));
        }

        (
            format!(
                "SELECT 
    tracks.name AS track,
    albums.name AS album,
    jsonb_group_array(artists.name) AS artists,
    tracks.external_id_isrc AS isrc
FROM tracks
    JOIN albums ON tracks.album_rowid = albums.rowid
    JOIN track_artists ON tracks.rowid = track_artists.track_rowid
    JOIN artists ON track_artists.artist_rowid = artists.rowid
WHERE
{}
GROUP BY tracks.rowid;",
                where_clause.join(" AND\n")
            ),
            named_params,
        )
    }
}

impl MetadataSource for SpotifyDB {
    async fn get_track(
        &self,
        meta: &util::Metadata,
        _fuzzy: bool, // not supported
    ) -> Result<Vec<util::Metadata>, String> {
        let client = self.client.lock().await;

        let (query, params) = SpotifyDB::build_query(meta);

        let mut query = client.prepare(query.as_str()).map_err(|e| e.to_string())?;
        let params = params
            .iter()
            .map(|(a, b)| (a.as_str(), &**b))
            .collect::<Vec<(&str, &str)>>();

        let req = query.query(params.as_slice()).map_err(|e| e.to_string())?;
        let res = req.mapped(|row| {
            Ok(util::Metadata {
                title: row.get(0).ok(),
                album: row.get(1).ok(),
                artists: row
                    .get::<_, Vec<u8>>(2)
                    .ok()
                    .map(|a| serde_sqlite_jsonb::from_slice::<Vec<String>>(a.as_slice()).ok())
                    .flatten()
                    .unwrap_or_default(),
                isrc: row.get(3).ok(),
                ..Default::default()
            })
        });
        Ok(res.map_while(|m| m.ok()).collect())
    }
}
