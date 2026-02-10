use std::{borrow::Cow, num::NonZero};

use crate::{autotag::MetadataSource, util};
use musicbrainz_rs::{
    MusicBrainzClient, Search,
    chrono::{Datelike, Local},
    entity::recording::Recording as MBRecording,
};

#[derive(Default)]
pub(super) struct MusicBrainz {
    client: MusicBrainzClient,
}

impl MusicBrainz {
    fn format_date(input: &str) -> Option<String> {
        match input.len() {
            10 if input.chars().nth(4) == Some('-') && input.chars().nth(7) == Some('-') => {
                let parts: Vec<&str> = input.split('-').collect();
                if parts.len() != 3 {
                    return None;
                }

                let year = parts[0].parse::<u32>().ok()?;
                let month = parts[1].parse::<NonZero<u32>>().ok()?;
                let day = parts[2].parse::<NonZero<u32>>().ok()?;

                if MusicBrainz::is_valid_date(year as i32, month, day) {
                    Some(input.to_string())
                } else {
                    None
                }
            }
            8 => {
                let year_str = &input[0..4];
                let month_str = &input[4..6];
                let day_str = &input[6..8];

                let year = year_str.parse::<u32>().ok()?;
                let month = month_str.parse::<NonZero<u32>>().ok()?;
                let day = day_str.parse::<NonZero<u32>>().ok()?;

                if MusicBrainz::is_valid_date(year as i32, month, day) {
                    return Some(format!("{}-{}-{}", year_str, month_str, day_str));
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn is_valid_date(year: i32, month: NonZero<u32>, day: NonZero<u32>) -> bool {
        // Validate year
        if year < 1900 || year > Local::now().year() {
            return false;
        }
        let days_in_month: u32 = match month.into() {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                    29
                } else {
                    28
                }
            }
            _ => return false,
        };

        u32::from(day) <= days_in_month
    }
}

impl MetadataSource for MusicBrainz {
    async fn get_track(
        &self,
        meta: &util::Metadata,
        fuzzy: bool,
    ) -> Result<Vec<util::Metadata>, String> {
        let query = if fuzzy {
            // TODO: TOO VIGILANT! ensure track titles overlap with query at least a little
            if let Some(isrc) = &meta.isrc {
                format!("isrc: \"{}\"", isrc)
            } else if let Some(title) = &meta.title {
                if let Some(artist) = meta.artists.first() {
                    format!("{} - {}", artist, title)
                } else {
                    title.to_string()
                }
            } else {
                "".to_string()
            }
        } else {
            let mut query = Vec::with_capacity(3);
            if let Some(title) = &meta.title {
                query.push(("recording", Cow::from(title)));
            }
            if let Some(artist) = meta.artists.first() {
                query.push(("artistname", Cow::from(artist)));
            }
            if let Some(isrc) = &meta.isrc {
                query.push(("isrc", Cow::from(isrc)));
            }
            let mut query = query
                .into_iter()
                .map(|(k, v)| format!("{}:\"{}\"", k, v))
                .collect::<Vec<String>>()
                .join(" AND ");

            if let Some(date) = &meta.date
                && let Some(datef) = MusicBrainz::format_date(&date)
            {
                query += format!(" date:\"{}\"", datef).as_str();
            }
            query
        };
        if query.is_empty() {
            return Err("Required Fields: title or isrc".to_string());
        }

        let resp = MBRecording::search(format!("query={}", query))
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
                album: f
                    .releases
                    .map(|rs| rs.first().map(|r| r.title.clone()))
                    .flatten(),
                genres: f
                    .tags
                    .map(|ts| ts.iter().map(|g| g.name.clone()).collect())
                    .unwrap_or_default(),
                date: f.first_release_date.map(|d| d.0),
                lyrics: None,
                isrc: f.isrcs.map(|i| i.first().cloned()).flatten(),
            })
            .collect())
    }
}
