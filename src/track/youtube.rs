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

    pub async fn try_from_url(track_url: &str, comment: Option<String>, user: &User) -> Result<Self, String> {

        let cover_url;
        let bpm;
        let genre_name;
        let created;

        println!("EnqueuedTrack::play > current_track_handle");
        println!("playing url: {}", track_url);

        // Here, we use lazy restartable sources to make sure that we don't pay
        // for decoding, playback on tracks which aren't actually live yet.
        let source = match Restartable::ytdl(track_url.to_string(), true).await {
            Ok(source) => source,
            Err(why) => {
                // TODO pass through error message
                // ERROR: Unsupported URL: https://open.spotify.com/track/03JZospDsa6M7Nkn6ab52x?si=2bc1ccb130b34ae4\n" }
                // Failed to play track: Json { error: Error("expected value", line: 1, column: 1), parsed_text: "ERROR: Video unavailable\nThis video is not available\n" }
                println!("Failed to enqueue track: {:?}", why);
                return Err(format!("Failed to enqueue track `{}`: {}", track_url, why));
            },
        };

        let (_track, track_handle) = create_player(source.into());
        println!("{:?}", track_handle.metadata());
        let metadata = track_handle.metadata();
        let name = metadata.title.clone();
        let user_name = metadata.artist.clone();
        let duration = metadata.duration;

        cover_url = None;
        bpm = None;
        genre_name = None;
        created = None;

        let track_playback_url = track_url.to_owned();

        Ok(
            Self {
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

            }
        )                    
    }

}

pub struct YoutubeTrackRefDispatcher {
   
}

impl YoutubeTrackRefDispatcher {
}

#[serenity::async_trait]
impl TrackRefDispatcher for YoutubeTrackRefDispatcher {
    async fn dispatch(&self, track_ref: &str, comment: Option<String>, user: &User) -> Option<Vec<Result<Box<dyn Track>, String>>> {

        // TODO check for the URL being youtube at all

        // let track_url_regex = Regex::new(r"http[s]?://www.youtube.com//([^/]+)")
        //     .unwrap_or_else(|err| panic!("failed fo create regex: {}", err));

        // let track_key = match track_url_regex.captures(&track_ref) {
        //     Some(cap) => {
        //         cap[1].to_owned()
        //     },
        //     None => {
        //         return None;
        //     },
        // };

        let mut results = Vec::new();
        let track = YoutubeTrack::try_from_url(track_ref, comment, user)
                .await
                .map(|track| Box::new(track) as Box<dyn Track>);

        results.push(track);

        Some(results)

        // match YoutubeTrack::try_from_track_key(track_key, chan_id, http).await {
        //     Ok(track) => {
        //     }
        //     Err(err) => {
        //         None
        //     }
        // }

        // // //let cap = track_url_regex.captures(&construction_url).ok_or_else(|| "not a valid audiotool track url".to_owned())?;
        // // //let track_id = cap[1].to_owned();
        // // let track_page_url = format!("https://www.audiotool.com/track/{}/", track_id);


        // // let client = reqwest::ClientBuilder::new()
        // //     .user_agent("Discord Bot")
        // //     .cookie_store(true)
        // //     .build()
        // //     .map_err(|err| format!("could not create http client: {}", err))?;

        // // let track_page_request = client.get(track_page_url.clone()).build().unwrap();
        // // let track_page_response = client.execute(track_page_request).await.expect("track page request failed");
        // // println!("res header: {:?}", track_page_response.headers());

        // //let cular_cookie = Self::extract_cular(track_page_response.headers()).ok_or_else(|| "failed to retrieve cookie".to_owned())?;
        // let cular_cookie = if let Some(cular_cookie) = AUDIOTOOL_HTTP_CLIENT.cular_cookie() {
        //     cular_cookie
        // } else {
        //     return None
        // };

        // let track_playback_url = format!("https://api.audiotool.com/track/{}/play.ogg?platform=1&ref=website&X-Cular-Session={}", track_id, cular_cookie);
        // let track_details_url = format!("https://www.audiotool.com/track/{}/details.json", track_id);
        
        // println!("Track details: {}", track_details_url);
        // //let playback_url = Self::retrieve_playback_url(&track_page_url, &track_id).await?;

        // // https://www.audiotool.com/track/5zcqbylu5mb/details.json
        // let track_details_request = client.get(track_details_url.clone()).build().unwrap();
        // let track_details_response = client.execute(track_details_request).await.expect("track details request failed");

        // let track_details_str = track_details_response.text().await.unwrap();

        // println!("{}", track_details_str);
        // println!("{}", json::parse(&track_details_str).unwrap());

        // let duration;
        // let name;
        // let cover_url;
        // let bpm;
        // let genre_key;
        // let user_key;
        // let user_name;
        // let created;
        // let comment_count;

        // // e.g. https://www.audiotool.com/track/5zcqbylu5mb/details.json
        // if let Ok(details) = AUDIOTOOL_HTTP_CLIENT.request_track_details(track_key) {
        //     duration = details["duration"].as_f64();
        //     name = details["name"].as_str().map(|s| s.to_owned());
        //     cover_url = details["coverUrl"].as_str().map(|s| s.to_owned());
        //     bpm = details["bpm"].as_f64();
        //     genre_key = details["genreKey"].as_str().map(|s| s.to_owned());
        //     user_key = details["user"]["key"].as_str().map(|s| s.to_owned());
        //     user_name = details["user"]["name"].as_str().map(|s| s.to_owned());
        //     created = details["created"].as_str().map(|s| s.to_owned());
        //     comment_count = details["comments"].as_u32();
        // } else {
        //     duration = None;
        //     name = None;
        //     cover_url = None;
        //     bpm = None;
        //     genre_key = None;
        //     user_key = None;
        //     user_name = None;
        //     created = None;
        //     comment_count = None;
        // }

        // Ok(
        //     Self {
        //         _construction_url: construction_url.to_owned(),
        //         track_id,
        //         track_page_url,
        //         track_playback_url,

        //         duration,
        //         name,
        //         cover_url,
        //         bpm,
        //         genre_key,
        //         user_key,
        //         user_name,
        //         created,
        //         comment_count,
            
        //         chan_id,
        //         http,
        //     }
        // );

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

    // fn channel_id(&self) -> ChannelId {
    //     self.chan_id
    // }

    // fn http(&self) -> Arc<Http> {
    //     self.http.clone()
    // }

}
