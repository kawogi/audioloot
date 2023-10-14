use std::time::Duration;

use serenity::{model::prelude::User, prelude::TypeMapKey};
use songbird::{create_player, input::Restartable};

use super::{Track, TrackRefDispatcher};

pub struct TrackKey {}

impl TypeMapKey for TrackKey {
    type Value = Box<dyn Track>;
}

pub struct YoutubeTrack {
    comment: Option<String>,

    track_page_url: String,
    pub track_playback_url: String,

    duration: Option<Duration>,
    name: Option<String>,
    cover_url: Option<String>,
    bpm: Option<f64>,
    genre_key: Option<String>,
    user_name: Option<String>,
    created: Option<String>,
    adding_user: User,
}

impl YoutubeTrack {
    pub async fn try_from_url(
        track_url: &str,
        comment: Option<String>,
        user: &User,
    ) -> Result<Self, String> {
        println!("YoutubeTrack::try_from_url > track_url");
        println!("playing url: {track_url}");

        // Here, we use lazy restartable sources to make sure that we don't pay
        // for decoding, playback on tracks which aren't actually live yet.
        let source = match Restartable::ytdl(track_url.to_string(), true).await {
            Ok(source) => source,
            Err(why) => {
                println!("Failed to enqueue track: {why:?}");
                return Err(format!("Failed to enqueue track `{track_url}`: {why}"));
            }
        };

        let (_track, track_handle) = create_player(source.into());
        println!("{:?}", track_handle.metadata());
        let metadata = track_handle.metadata();
        let name = metadata.title.clone();
        let user_name = metadata.artist.clone();
        let duration = metadata.duration;

        // we have, what we want
        track_handle.stop().map_err(|err| err.to_string())?;

        // println!("youtube metadata: {metadata:#?}");

        let cover_url = None;
        let bpm = None;
        let genre_name = None;
        let created = None;

        let track_playback_url = track_url.to_owned();

        Ok(Self {
            comment,
            track_page_url: track_url.to_owned(),
            track_playback_url,

            duration,
            name,
            cover_url,
            bpm,
            genre_key: genre_name,
            user_name,
            created,
            adding_user: user.clone(),
        })
    }
}

pub struct YoutubeTrackRefDispatcher {}

impl YoutubeTrackRefDispatcher {}

#[serenity::async_trait]
impl TrackRefDispatcher for YoutubeTrackRefDispatcher {
    async fn dispatch(
        &self,
        track_ref: &str,
        comment: Option<String>,
        user: &User,
    ) -> Option<Vec<Result<Box<dyn Track>, String>>> {
        // TODO check for the URL being youtube at all

        let mut results = Vec::new();
        let track = YoutubeTrack::try_from_url(track_ref, comment, user)
            .await
            .map(|track| Box::new(track) as Box<dyn Track>);

        results.push(track);

        Some(results)
    }
}

#[serenity::async_trait]
impl Track for YoutubeTrack {
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
}
