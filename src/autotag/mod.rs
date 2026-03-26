use crate::util;
use tokio::join;
mod deezer;
mod lrclib;
mod musicbrainz;
mod spotifydb;

pub(crate) trait MetadataSource {
    async fn get_track(
        &self,
        meta: &util::Metadata,
        fuzzy: bool,
    ) -> Result<Vec<util::Metadata>, String>;
}

pub struct MetadataSources {
    /// Complete, +Genre
    musicbrainz: musicbrainz::MusicBrainz,
    /// Correct
    deezer: deezer::Deezer,
    /// Strict, Correct, +Lyrics
    lrclib: lrclib::LRCLib,
    /// Strict, Correct
    spotifydb: Option<spotifydb::SpotifyDB>,
}

impl MetadataSources {
    pub(crate) fn new(spotifydbfile: Option<String>) -> Self {
        Self {
            spotifydb: spotifydbfile.map(|f| spotifydb::SpotifyDB::new(f)),
            musicbrainz: musicbrainz::MusicBrainz::default(),
            deezer: deezer::Deezer::default(),
            lrclib: lrclib::LRCLib::default(),
        }
    }
}

impl MetadataSource for MetadataSources {
    async fn get_track(
        &self,
        meta: &util::Metadata,
        fuzzy: bool,
    ) -> Result<Vec<util::Metadata>, String> {
        let mut candidates: Vec<_> = if !fuzzy {
            if let Some(spotifydb) = &self.spotifydb {
                <[Result<Vec<util::Metadata>, String>; 4]>::from(join!(
                    spotifydb.get_track(meta, fuzzy),
                    self.lrclib.get_track(meta, fuzzy),
                    self.deezer.get_track(meta, fuzzy),
                    self.musicbrainz.get_track(meta, fuzzy)
                ))
                .to_vec()
            } else {
                <[Result<Vec<util::Metadata>, String>; 3]>::from(join!(
                    self.lrclib.get_track(meta, fuzzy),
                    self.deezer.get_track(meta, fuzzy),
                    self.musicbrainz.get_track(meta, fuzzy)
                ))
                .to_vec()
            }
        } else {
            <[Result<Vec<util::Metadata>, String>; 2]>::from(join!(
                self.deezer.get_track(meta, fuzzy),
                self.musicbrainz.get_track(meta, fuzzy)
            ))
            .to_vec()
        }
        .into_iter()
        .filter_map(|c| c.ok())
        .flatten()
        .collect();

        candidates.sort_by(|a, b| meta.similarity(b).total_cmp(&meta.similarity(a)));

        Ok(candidates)
    }
}
