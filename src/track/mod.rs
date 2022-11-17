use std::time::Duration;

use serenity::model::prelude::User;

pub mod audiotool;
pub mod youtube;
pub mod selection;

#[serenity::async_trait]
pub trait Track: Send + Sync {

    fn caption(&self) -> String {
        let artist = self.artist().as_ref().map(|v| v.to_string()).unwrap_or_else(|| "(unknown artist)".to_string());
        let title = self.title().as_ref().map(|v| v.to_string()).unwrap_or_else(|| "(unknown title)".to_string());

        let duration = if let Some(duration) = self.duration() {
            let seconds = duration.as_secs();
            format!(" ({}:{:0>2})", seconds / 60, seconds % 60)
        } else {
            "".to_owned()
        };

        let genre_bpm = match (&self.genre(), self.bpm()) {
            (None, None) => "".to_owned(),
            (None, Some(bpm)) => format!("[@{} bpm]", bpm.round().max(0.0) as u32),
            (Some(genre), None) => format!(" [{}]", genre),
            (Some(genre), Some(bpm)) => format!(" [{}@{} bpm]", genre, bpm.round().max(0.0) as u32),
        };

        let comment = if let Some(comment) = &self.comment() {
            format!(" \"{}\"", comment)
        } else {
            "".to_owned()
        };

        let mention = format!("`@{}`", self.adding_user().name);
        format!("**{}** - **{}**{}{}{} {}", artist, title, duration, genre_bpm, comment, mention)
    }

    fn track_page_url(&self) -> &str;
    fn playback_url(&self) -> &str;
    fn duration(&self) -> Option<Duration>;
    fn title(&self) -> Option<String>;
    fn cover_url(&self) -> Option<String>;
    fn bpm(&self) -> Option<f64>;
    fn genre(&self) -> Option<String>;
    fn artist(&self) -> Option<String>;
    fn created(&self) -> Option<String>;
    fn comment(&self) -> Option<String>;
    fn adding_user(&self) -> &User;
}

#[serenity::async_trait]
pub trait TrackRefDispatcher: Send + Sync {
    async fn dispatch(&self, track_ref: &str, comment: Option<String>, user: &User) -> Option<Vec<Result<Box<dyn Track>, String>>>;
}
