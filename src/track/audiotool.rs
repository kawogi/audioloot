use std::{string::ToString, time::Duration};

use json::JsonValue;
use regex::Regex;
use serenity::model::prelude::User;
//use serenity::prelude::TypeMapKey;

use crate::AUDIOTOOL_HTTP_CLIENT;

use super::{Track, TrackRefDispatcher};

// struct MetaData {
//     duration: Option<f64>,
//     name: Option<String>,
//     cover_url: Option<String>,
//     bpm: Option<f64>,
//     genre_key: Option<String>,
//     user_key: Option<String>,
//     user_name: Option<String>,
//     created: Option<String>,
//     comment_count: Option<u32>,
// }

// pub struct TrackKey {}

// impl TypeMapKey for TrackKey {
//     type Value = Box<dyn Track>;
// }

pub struct AudiotoolTrack {
    // chan_id: ChannelId,
    // http: Arc<Http>,
    comment: Option<String>,

    //_construction_url: String,
    //track_key: String,
    track_page_url: String,
    pub track_playback_url: String,

    duration: Option<Duration>,
    name: Option<String>,
    cover_url: Option<String>,
    bpm: Option<f64>,
    genre_key: Option<String>,
    //user_key: Option<String>,
    user_name: Option<String>,
    created: Option<String>,
    //comment_count: Option<u32>,
    adding_user: User,
}

impl AudiotoolTrack {
    pub async fn try_from_track_key(
        track_key: &str,
        comment: Option<String>,
        user: &User,
    ) -> Result<Self, String> {
        let track_page_url = format!("https://www.audiotool.com/track/{track_key}/");

        let duration;
        let name;
        let cover_url;
        let bpm;
        let genre_name;
        //let user_key;
        let user_name;
        let created;
        //let comment_count;

        // e.g. https://www.audiotool.com/track/5zcqbylu5mb/details.json
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        if let Ok(details) = AUDIOTOOL_HTTP_CLIENT.request_track_details(track_key).await {
            duration = details["duration"]
                .as_f64()
                .map(|millis| Duration::from_millis(millis.round().max(0.0) as u64));
            name = details["name"].as_str().map(ToOwned::to_owned);
            cover_url = details["coverUrl"].as_str().map(ToOwned::to_owned);
            bpm = details["bpm"].as_f64();
            genre_name = details["genreName"].as_str().map(ToOwned::to_owned);
            //user_key = details["user"]["key"].as_str().map(|s| s.to_owned());
            user_name = details["user"]["name"].as_str().map(ToOwned::to_owned);
            created = details["created"].as_str().map(ToOwned::to_owned);
            //comment_count = details["comments"].as_u32();
        } else {
            duration = None;
            name = None;
            cover_url = None;
            bpm = None;
            genre_name = None;
            //user_key = None;
            user_name = None;
            created = None;
            //comment_count = None;
        }

        let cookies = AUDIOTOOL_HTTP_CLIENT.cular_cookie();

        let cular_cookie = cookies
            .get("cular-session")
            .ok_or("cular cookie is invalid")?;
        let track_playback_url = format!(
            "https://api.audiotool.com/track/{track_key}/play.ogg?platform=1&ref=website&X-Cular-Session={cular_cookie}"
        );

        Ok(Self {
            //_construction_url: construction_url.to_owned(),
            //track_key: track_key.to_owned(),
            comment,
            track_page_url,
            track_playback_url,

            duration,
            name,
            cover_url,
            bpm,
            genre_key: genre_name,
            //user_key,
            user_name,
            created,
            adding_user: user.clone(),
            //comment_count,

            // chan_id,
            // http,
        })
    }
}

pub struct AudiotoolTrackRefDispatcher {}

#[serenity::async_trait]
impl TrackRefDispatcher for AudiotoolTrackRefDispatcher {
    async fn dispatch(
        &self,
        track_ref: &str,
        comment: Option<String>,
        user: &User,
    ) -> Option<Vec<Result<Box<dyn Track>, String>>> {
        let track_url_regex = Regex::new(r"http[s]?://www.audiotool.com/track/([^/]+)")
            .unwrap_or_else(|err| panic!("failed fo create regex: {}", err));

        let track_key = match track_url_regex.captures(track_ref) {
            Some(cap) => cap[1].to_owned(),
            None => {
                return None;
            }
        };

        let mut results = Vec::new();
        let track = AudiotoolTrack::try_from_track_key(&track_key, comment, user)
            .await
            .map(|track| Box::new(track) as Box<dyn Track>);

        results.push(track);

        Some(results)
    }
}

pub struct AudiotoolSingleChartsDispatcher {}

impl AudiotoolSingleChartsDispatcher {}

#[serenity::async_trait]
impl TrackRefDispatcher for AudiotoolSingleChartsDispatcher {
    async fn dispatch(
        &self,
        track_ref: &str,
        comment: Option<String>,
        user: &User,
    ) -> Option<Vec<Result<Box<dyn Track>, String>>> {
        // https://www.audiotool.com/genre/trap/charts/2021-35

        if track_ref != "at:single-charts" {
            return None;
        }

        let mut results = Vec::new();

        if let Ok(charts) = AUDIOTOOL_HTTP_CLIENT.request_single_charts_details().await {
            if let JsonValue::Array(tracks) = &charts["tracks"] {
                for (index, track_detail) in tracks.iter().enumerate() {
                    let comment = comment
                        .clone()
                        .unwrap_or_else(|| format!("#{} in Single Charts", index + 1));
                    if let Some(track_key) = track_detail["key"].as_str() {
                        let track =
                            AudiotoolTrack::try_from_track_key(track_key, Some(comment), user)
                                .await
                                .map(|track| Box::new(track) as Box<dyn Track>);

                        results.push(track);
                    }
                }
            }
        }
        results.reverse();

        Some(results)
    }
}

pub struct AudiotoolGenreChartsDispatcher {}

impl AudiotoolGenreChartsDispatcher {}

#[serenity::async_trait]
impl TrackRefDispatcher for AudiotoolGenreChartsDispatcher {
    async fn dispatch(
        &self,
        track_ref: &str,
        comment: Option<String>,
        user: &User,
    ) -> Option<Vec<Result<Box<dyn Track>, String>>> {
        // https://www.audiotool.com/genre/trap/charts/2021-35

        let track_url_regex =
            Regex::new(r"http[s]?://www.audiotool.com/genre/([^/]+)/charts/(\d{4}-\d{2})")
                .unwrap_or_else(|err| panic!("failed fo create regex: {}", err));

        let (genre_key, date) = match track_url_regex.captures(track_ref) {
            Some(cap) => (cap[1].to_owned(), cap[2].to_owned()),
            None => {
                return None;
            }
        };

        let mut results = Vec::new();

        if let Ok(charts) = AUDIOTOOL_HTTP_CLIENT
            .request_genre_charts_details(&genre_key, &date)
            .await
        {
            let name = charts["name"]
                .as_str()
                .map(ToString::to_string)
                .unwrap_or(format!("{genre_key} charts {date}"));
            if let JsonValue::Array(tracks) = &charts["tracks"] {
                for (index, track_detail) in tracks.iter().enumerate() {
                    let comment = comment
                        .clone()
                        .unwrap_or_else(|| format!("#{} in {}", index + 1, name));
                    if let Some(track_key) = track_detail["key"].as_str() {
                        let track =
                            AudiotoolTrack::try_from_track_key(track_key, Some(comment), user)
                                .await
                                .map(|track| Box::new(track) as Box<dyn Track>);

                        results.push(track);
                    }
                }
            }
        }
        results.reverse();

        Some(results)
    }
}

pub struct AudiotoolAlbumDispatcher {}

impl AudiotoolAlbumDispatcher {}

#[serenity::async_trait]
impl TrackRefDispatcher for AudiotoolAlbumDispatcher {
    async fn dispatch(
        &self,
        track_ref: &str,
        comment: Option<String>,
        user: &User,
    ) -> Option<Vec<Result<Box<dyn Track>, String>>> {
        // https://www.audiotool.com/genre/trap/charts/2021-35

        let track_url_regex = Regex::new(r"http[s]?://www.audiotool.com/album/([^/]+)")
            .unwrap_or_else(|err| panic!("failed fo create regex: {}", err));

        let album_key = match track_url_regex.captures(track_ref) {
            Some(cap) => cap[1].to_owned(),
            None => {
                return None;
            }
        };

        let mut results = Vec::new();

        if let Ok(charts) = AUDIOTOOL_HTTP_CLIENT
            .request_album_details(&album_key)
            .await
        {
            let name = charts["name"]
                .as_str()
                .map(ToString::to_string)
                .unwrap_or(format!("album: {album_key}"));
            if let JsonValue::Array(tracks) = &charts["tracks"] {
                for (index, track_detail) in tracks.iter().enumerate() {
                    let comment = comment
                        .clone()
                        .unwrap_or_else(|| format!("#{} in {}", index + 1, name));
                    if let Some(track_key) = track_detail["key"].as_str() {
                        let track =
                            AudiotoolTrack::try_from_track_key(track_key, Some(comment), user)
                                .await
                                .map(|track| Box::new(track) as Box<dyn Track>);

                        results.push(track);
                    }
                }
            }
        }

        Some(results)
    }
}

#[serenity::async_trait]
impl Track for AudiotoolTrack {
    fn track_page_url(&self) -> &str {
        &self.track_page_url
    }

    fn playback_url(&self) -> &str {
        &self.track_playback_url
    }

    fn duration(&self) -> Option<Duration> {
        self.duration
    }

    fn title(&self) -> Option<String> {
        self.name.clone()
    }

    fn cover_url(&self) -> Option<String> {
        self.cover_url.clone()
    }

    fn bpm(&self) -> Option<f64> {
        self.bpm
    }

    fn genre(&self) -> Option<String> {
        self.genre_key.clone()
    }

    fn artist(&self) -> Option<String> {
        self.user_name.clone()
    }

    fn created(&self) -> Option<String> {
        self.created.clone()
    }

    fn comment(&self) -> Option<String> {
        self.comment.clone()
    }

    fn adding_user(&self) -> &User {
        &self.adding_user
    }

    // fn channel_id(&self) -> ChannelId {
    //     self.chan_id
    // }

    // fn http(&self) -> Arc<Http> {
    //     self.http.clone()
    // }
}
