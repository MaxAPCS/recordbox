use mp4ameta::Tag;
use serde::{
    Deserialize,
    de::{MapAccess, Visitor},
    ser::SerializeMap,
};
use std::fmt;

/// Wrapper struct that implements Serialize for mp4ameta::Tag
#[derive(Debug)]
pub(crate) struct Mp4TagSer {
    pub(crate) tag: Tag,
}

impl From<Tag> for Mp4TagSer {
    fn from(value: Tag) -> Self {
        Self { tag: value }
    }
}

impl serde::Serialize for Mp4TagSer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(None)?;

        // Serialize standard metadata fields
        if let Some(title) = self.tag.title() {
            map.serialize_entry("title", title)?;
        }

        let artists: Vec<&str> = self.tag.artists().collect();
        if !artists.is_empty() {
            map.serialize_entry("artists", &artists)?;
        }

        if let Some(album) = self.tag.album() {
            map.serialize_entry("album", album)?;
        }

        if let Some(album_artist) = self.tag.album_artist() {
            map.serialize_entry("album_artist", album_artist)?;
        }

        if let Some(genre) = self.tag.genre() {
            map.serialize_entry("genre", genre)?;
        }

        if let Some(composer) = self.tag.composer() {
            map.serialize_entry("composer", composer)?;
        }

        if let Some(year) = self.tag.year() {
            map.serialize_entry("year", &year)?;
        }

        if let Some(track_number) = self.tag.track_number() {
            map.serialize_entry("track_number", &track_number)?;
        }

        if let Some(total_tracks) = self.tag.total_tracks() {
            map.serialize_entry("total_tracks", &total_tracks)?;
        }

        if let Some(disc_number) = self.tag.disc_number() {
            map.serialize_entry("disc_number", &disc_number)?;
        }

        if let Some(total_discs) = self.tag.total_discs() {
            map.serialize_entry("total_discs", &total_discs)?;
        }

        if let Some(bpm) = self.tag.bpm() {
            map.serialize_entry("bpm", &bpm)?;
        }

        if let Some(comment) = self.tag.comment() {
            map.serialize_entry("comment", comment)?;
        }

        if let Some(lyrics) = self.tag.lyrics() {
            map.serialize_entry("lyrics", lyrics)?;
        }

        if let Some(grouping) = self.tag.grouping() {
            map.serialize_entry("grouping", grouping)?;
        }

        map.end()
    }
}

impl<'de> Deserialize<'de> for Mp4TagSer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(Mp4TagVisitor)
    }
}

struct Mp4TagVisitor;

impl<'de> Visitor<'de> for Mp4TagVisitor {
    type Value = Mp4TagSer;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a map containing MP4 metadata fields")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut deserializer = Mp4TagSer {
            tag: Tag::default(),
        };

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "title" => {
                    let value: Option<String> = map.next_value()?;
                    if let Some(v) = value {
                        deserializer.tag.set_title(v);
                    }
                }
                "artists" => {
                    let value: Option<Vec<String>> = map.next_value()?;
                    if let Some(v) = value {
                        deserializer.tag.set_artists(v);
                    }
                }
                "album" => {
                    let value: Option<String> = map.next_value()?;
                    if let Some(v) = value {
                        deserializer.tag.set_album(v);
                    }
                }
                "album_artist" => {
                    let value: Option<String> = map.next_value()?;
                    if let Some(v) = value {
                        deserializer.tag.set_album_artist(v);
                    }
                }
                "genre" => {
                    let value: Option<String> = map.next_value()?;
                    if let Some(v) = value {
                        deserializer.tag.set_genre(v);
                    }
                }
                "composer" => {
                    let value: Option<String> = map.next_value()?;
                    if let Some(v) = value {
                        deserializer.tag.set_composer(v);
                    }
                }
                "year" => {
                    if let Some(year) = map.next_value::<Option<String>>()? {
                        deserializer.tag.set_year(year);
                    }
                }
                "track_number" => {
                    if let Some(track) = map.next_value::<Option<u16>>()? {
                        deserializer.tag.set_track_number(track);
                    }
                }
                "total_tracks" => {
                    if let Some(total) = map.next_value::<Option<u16>>()? {
                        deserializer.tag.set_total_tracks(total);
                    }
                }
                "disc_number" => {
                    if let Some(disc) = map.next_value::<Option<u16>>()? {
                        deserializer.tag.set_disc_number(disc);
                    }
                }
                "total_discs" => {
                    if let Some(total) = map.next_value::<Option<u16>>()? {
                        deserializer.tag.set_total_discs(total);
                    }
                }
                "bpm" => {
                    if let Some(bpm) = map.next_value::<Option<u16>>()? {
                        deserializer.tag.set_bpm(bpm);
                    }
                }
                "comment" => {
                    let value: Option<String> = map.next_value()?;
                    if let Some(v) = value {
                        deserializer.tag.set_comment(v);
                    }
                }
                "lyrics" => {
                    let value: Option<String> = map.next_value()?;
                    if let Some(v) = value {
                        deserializer.tag.set_lyrics(v);
                    }
                }
                "grouping" => {
                    let value: Option<String> = map.next_value()?;
                    if let Some(v) = value {
                        deserializer.tag.set_grouping(v);
                    }
                }
                // Unknown fields are ignored
                _ => map.next_value()?,
            }
        }

        Ok(deserializer)
    }
}
