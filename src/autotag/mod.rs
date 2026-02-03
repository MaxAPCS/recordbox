use crate::util;
mod deezer;
mod lrclib;
mod musicbrainz;

pub(crate) trait MetadataSource {
    async fn get_track(&self, meta: util::Metadata) -> Result<Vec<util::Metadata>, String>;
}

#[derive(Default)]
pub struct MetadataSources {
    musicbrainz: musicbrainz::MusicBrainz,
    deezer: deezer::Deezer,
    lrclib: lrclib::LRCLib,
}

impl MetadataSource for MetadataSources {
    async fn get_track(&self, meta: util::Metadata) -> Result<Vec<util::Metadata>, String> {
        // self.musicbrainz.get_track(meta).await
        // self.deezer.get_track(meta).await
        self.lrclib.get_track(meta).await
    }
}
