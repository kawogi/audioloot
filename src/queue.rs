use std::{collections::HashMap, iter::FromIterator, sync::Arc, time::Duration};

use serenity::{
    http::Http,
    model::id::{ChannelId, GuildId, UserId},
};
use songbird::{input::Restartable, tracks::TrackHandle, Call, Event, EventContext, TrackEvent};

use crate::{
    command::{CommandError, CommandResult},
    message::MessageChannel,
    track::{
        selection::{IndexResolve, TrackIndex, TrackIndexSelection},
        Track,
    },
};

use serenity::async_trait;
use songbird::EventHandler as VoiceEventHandler;

use crate::guild::GUILD_STATES;
use tokio::sync::Mutex;

struct TrackEndNotifier {
    guild_id: GuildId,
}

#[async_trait]
impl VoiceEventHandler for TrackEndNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(tracks) = ctx {
            let guild = GUILD_STATES.get_guild_state(self.guild_id).await;
            for &(track_state, _track_handle) in *tracks {
                match track_state.playing {
                    songbird::tracks::PlayMode::Stop => {
                        println!("stopped track_state: {track_state:?}");
                        //let _ = guild.handle_track_end(track_handle).await;
                    }
                    songbird::tracks::PlayMode::End => {
                        println!("stopped track_state: {track_state:?}");
                        let () = guild.handle_track_end().await;
                    }
                    _ => {}
                }
            }
        }

        None
    }
}

#[derive(Clone)]
pub struct QueueHandle(Arc<Mutex<Queue>>);

impl QueueHandle {}

pub struct Queue {
    guild_id: GuildId,
    default_message_channel: MessageChannel,
    tracks: Vec<EnqueuedTrack>,
    deferred_track_index: usize,
    current_track_index: usize,
    current_track_handle: Option<TrackHandle>,
    voice_connection: Option<Arc<Mutex<Call>>>,
    is_active: bool,
    quota: Option<usize>,
}

impl Queue {
    pub fn new(guild_id: GuildId, default_message_channel: MessageChannel) -> Self {
        Self {
            guild_id,
            default_message_channel,
            tracks: Vec::default(),
            deferred_track_index: 0,
            current_track_index: 0,
            current_track_handle: None,
            voice_connection: None,
            is_active: false,
            quota: None,
        }
    }

    pub fn set_http(&mut self, http: Arc<Http>) {
        self.default_message_channel.set_http(http);
    }

    pub fn set_default_message_channel(&mut self, channel_id: ChannelId) {
        self.default_message_channel.set_channel(channel_id);
    }

    pub fn connect(&mut self, voice_connection: Arc<Mutex<Call>>) {
        self.voice_connection = Some(voice_connection);
    }

    pub fn disconnect(&mut self) {
        let _ = self.stop();
        self.voice_connection = None;
    }

    fn try_enqueue_deferred(&mut self) {
        let current_track_index = self.current_track_index;
        let track_count = self.tracks.len();

        // unconditionally materialize deferred tracks
        self.deferred_track_index = if current_track_index < track_count {
            self.deferred_track_index.max(current_track_index + 1)
        } else {
            track_count
        };

        // now let's see if we can materialize more tracks

        let mut counts: HashMap<UserId, usize> = HashMap::new();
        // count remaining users in the official list
        for track in &self.tracks[current_track_index..self.deferred_track_index] {
            *counts.entry(track.track.adding_user().id).or_default() += 1;
        }

        // try to append pending tracks
        for i in self.deferred_track_index..track_count {
            let pending_track = &self.tracks[i];
            let count = counts
                .entry(pending_track.track.adding_user().id)
                .or_default();
            let materialize = self.quota.map_or(true, |quota| *count < quota);
            if materialize {
                *count += 1;
                let track = self.tracks.remove(i);
                self.tracks.insert(self.deferred_track_index, track);
                self.deferred_track_index += 1;
            }
        }
    }

    pub async fn append(&mut self, user_track: Box<dyn Track>) {
        //let user_id = user_track.adding_user();

        self.tracks.push(EnqueuedTrack { track: user_track });
        self.try_enqueue_deferred();

        // // find proper insert location
        // let user_track_count = self.tracks[self.current_track_index..self.deferred_track_index].iter().filter(|track| track.track.adding_user() == user_id).count();
        // if user_track_count < MAX_PLANNED {
        //     self.tracks.insert(self.deferred_track_index, EnqueuedTrack { track: user_track });
        //     self.deferred_track_index += 1;
        // } else {
        //     self.tracks.push(EnqueuedTrack { track: user_track });
        // }
        // // shouldn't be necessary, but doesn't hurt
        // //self.try_enqueue_pending();

        // auto-start playback
        if self.current_track_handle.is_none() && self.is_active {
            let _ = self.play().await;
        }
    }

    pub async fn handle_track_end(&mut self) {
        let _ = self.next().await;
    }

    pub fn stop(&mut self) -> CommandResult {
        println!("queue::stop");
        if let Some(track) = self.current_track_handle.take() {
            track.stop()?;
        } else if !self.is_active {
            return Err(CommandError::Execution(
                "There's nothing to be stopped.".to_owned(),
            ));
        }
        self.is_active = false;
        Ok(())
    }

    pub fn pause(&mut self) -> CommandResult {
        if let Some(track) = &self.current_track_handle {
            track.pause()?;
        } else {
            return Err(CommandError::Execution(
                "There's nothing to be paused.".to_owned(),
            ));
        }
        Ok(())
    }

    pub fn seek(&mut self, position: Duration) -> CommandResult {
        if let Some(track) = &self.current_track_handle {
            track.seek_time(position)?;
        } else {
            return Err(CommandError::Execution(
                "There's nothing to seek in.".to_owned(),
            ));
        }
        Ok(())
    }

    pub fn resume(&mut self) -> CommandResult {
        if let Some(track) = &self.current_track_handle {
            track.play()?;
        } else {
            return Err(CommandError::Execution(
                "There's nothing to be resumed.".to_owned(),
            ));
        }
        Ok(())
    }

    pub async fn play(&mut self) -> CommandResult {
        println!("queue::play");

        self.is_active = true;

        let voice_connection = self
            .voice_connection
            .as_ref()
            .ok_or(CommandError::BotVoiceChannelRequired)?;

        if let Some(track) = self.current_track_handle.take() {
            println!("queue::play > stop previous {track:?}");
            let _ = track.stop();
        }

        let track = self.tracks.get(self.current_track_index)
                .ok_or_else(|| CommandError::Execution("There's no track in the queue to be played. Use the `enqueue` command to add some tracks.".to_owned()))?;

        self.current_track_handle = Some(
            track
                .play(
                    voice_connection,
                    &self.default_message_channel,
                    self.guild_id,
                )
                .await?,
        );
        println!(
            "queue::play > current_track_handle {:?}",
            self.current_track_handle
        );
        Ok(())
    }

    pub async fn deafen(&mut self) -> CommandResult {
        let voice_connection = self
            .voice_connection
            .as_ref()
            .ok_or(CommandError::BotVoiceChannelRequired)?;
        let handler_lock = voice_connection;

        let mut handler = handler_lock.lock().await;

        handler.deafen(true).await?;
        Ok(())
    }

    pub async fn print(
        &self,
        out: &MessageChannel,
        track_selection: &TrackIndexSelection,
    ) -> CommandResult {
        if self.tracks.is_empty() {
            return Err(CommandError::Execution(
                "The playback queue is empty. Use the `enqueue` command to add some tracks."
                    .to_owned(),
            ));
        }

        let set = track_selection.collect(self.current_track_index, self.tracks.len());
        if set.is_empty() {
            return Err(CommandError::Execution(
                "None of the given track numbers was found.".to_owned(),
            ));
        }

        let mut tracks = Vec::from_iter(set);
        tracks.sort_unstable();
        println!("printing tracks: {tracks:?}");

        let mut message = "Current queue:".to_owned();
        for (index, track) in tracks.iter().map(|&index| (index, &self.tracks[index])) {
            if index == self.current_track_index {
                message.push_str(&format!("\n`▶{:>4}:` {} `◀`", index + 1, track.caption()));
            } else if index < self.deferred_track_index {
                message.push_str(&format!("\n` {:>4}:` {}", index + 1, track.caption()));
            } else {
                message.push_str(&format!("\n`?{:>4}:` {}", index + 1, track.caption()));
            }
        }

        if let Some(&last_index) = tracks.last() {
            if last_index + 1 < self.tracks.len() {
                message.push_str(&format!(
                    "\n`  ...:` (and {} more track(s))",
                    self.tracks.len() - last_index - 1
                ));
            } else if self.current_track_index == self.tracks.len() {
                message.push_str("\n`▶ END:` (you've reached the end of the queue) `◀`");
            } else {
                message.push_str("\n`  END:` (you've reached the end of the queue)");
            }
        }

        out.print(message).await;
        Ok(())
    }

    pub async fn goto(&mut self, track_index: TrackIndex) -> CommandResult {
        match track_index.resolve(self.current_track_index, self.tracks.len()) {
            IndexResolve::Ok(index) | IndexResolve::End(index) => {
                self.current_track_index = index;
                self.try_enqueue_deferred();
                if self.is_active {
                    self.play().await
                } else {
                    Ok(())
                }
            }
            IndexResolve::TooSmall(index) => Err(CommandError::Execution(format!(
                "Track #{} doesn't exist",
                index + 1
            ))),
            IndexResolve::TooBig(index) => Err(CommandError::Execution(format!(
                "Track #{} doesn't exist",
                index + 1
            ))),
        }
    }

    pub async fn next(&mut self) -> CommandResult {
        self.goto(TrackIndex::Current(1)).await
    }

    pub async fn prev(&mut self) -> CommandResult {
        self.goto(TrackIndex::Current(-1)).await
    }

    pub async fn remove(
        &mut self,
        out: &MessageChannel,
        track_selection: &TrackIndexSelection,
    ) -> CommandResult {
        let set = track_selection.collect(self.current_track_index, self.tracks.len());
        let track_count = set.len();
        println!("removing tracks: {set:?}");

        // if set.is_empty() {
        //     msg.print("None of the given track numbers was found.").await;
        //     return Ok(());
        // }

        let mut tracks = Vec::from_iter(set);
        tracks.sort_by(|a, b| b.cmp(a));
        println!("removing tracks: {tracks:?}");

        let mut killed_current = false;
        for track in tracks {
            self.tracks.remove(track);
            match track.cmp(&self.current_track_index) {
                std::cmp::Ordering::Less => self.current_track_index -= 1,
                std::cmp::Ordering::Equal => killed_current = true,
                std::cmp::Ordering::Greater => {}
            }
            if track < self.deferred_track_index {
                self.deferred_track_index -= 1;
            }
        }
        self.try_enqueue_deferred();

        out.print(format!("Removed {track_count} track(s) from the queue."))
            .await;

        if killed_current && self.is_active {
            // FIXME triggers a false error if next track isn't available - looks like remove failed - likely more errors of this kind for other commands with auto-play
            self.play().await
        } else {
            Ok(())
        }
    }

    pub async fn now(&self, out: &MessageChannel) -> CommandResult {
        if let (true, Some(track), Some(handle)) = (
            self.is_active,
            self.tracks.get(self.current_track_index),
            &self.current_track_handle,
        ) {
            track.announce(out).await;
            track.announce_position(out, handle).await;
            Ok(())
        } else {
            out.print("There's no current track. Use the `enqueue` command to add some tracks.")
                .await;
            Ok(())
        }
    }

    pub async fn reverse(&mut self, track_selection: &TrackIndexSelection) -> CommandResult {
        let set = track_selection.collect(self.current_track_index, self.tracks.len());

        let mut tracks = Vec::from_iter(set);
        tracks.sort_unstable();
        let mut killed_current = false;
        for (i, j) in (0..tracks.len() / 2).map(|i| (tracks[i], tracks[tracks.len() - 1 - i])) {
            killed_current |= self.current_track_index == i;
            killed_current |= self.current_track_index == j;
            self.tracks.swap(i, j);
        }

        self.try_enqueue_deferred();

        if killed_current && self.is_active {
            self.play().await
        } else {
            Ok(())
        }
    }

    #[allow(clippy::unnecessary_wraps)] // for symmetry with other commands
    pub fn set_quota(&mut self, quota: Option<usize>) -> CommandResult {
        self.quota = quota;
        self.try_enqueue_deferred();
        Ok(())
    }

    pub async fn print_quota(&self, out: &MessageChannel) -> CommandResult {
        match self.quota {
            None => out.print("Quota is _off_. Users are allowed to enqueue as many tracks as they like.").await,
            Some(quota) => out.print(format!("Quota is _on_. Each user is allowed to enqueue up to {quota} track(s) until other users will be given a higher priority.")).await,
        }
        Ok(())
    }

    pub async fn move_tracks(
        &mut self,
        out: &MessageChannel,
        track_selection: &TrackIndexSelection,
        index: TrackIndex,
    ) -> CommandResult {
        let mut insert_index = match index.resolve(self.current_track_index, self.tracks.len()) {
            IndexResolve::Ok(index) => index,
            IndexResolve::TooSmall(index) => {
                return Err(CommandError::Execution(format!(
                    "Destination slot #{} doesn't exist",
                    index + 1
                )));
            }
            IndexResolve::TooBig(index) | IndexResolve::End(index) => {
                return Err(CommandError::Execution(format!(
                    "Destination slot #{} doesn't exist",
                    index + 1
                )));
            }
        };

        let set = track_selection.collect(self.current_track_index, self.tracks.len());
        let track_count = set.len();
        println!("moving tracks: {set:?}");

        // if set.is_empty() {
        //     msg.print("None of the given track numbers was found.").await;
        //     return Ok(());
        // }

        let mut tracks_to_move = Vec::from_iter(set);
        tracks_to_move.sort_by(|a, b| b.cmp(a));
        println!("moving tracks: {tracks_to_move:?}");

        let mut moved_tracks = Vec::new();

        let mut killed_current = false;
        for track_to_move in tracks_to_move {
            moved_tracks.push(self.tracks.remove(track_to_move));
            match track_to_move.cmp(&self.current_track_index) {
                std::cmp::Ordering::Less => self.current_track_index -= 1,
                std::cmp::Ordering::Equal => killed_current = true,
                std::cmp::Ordering::Greater => {}
            }
            if track_to_move < self.deferred_track_index {
                self.deferred_track_index -= 1;
            }
            if track_to_move < insert_index {
                insert_index -= 1;
            }
        }

        let tail = self.tracks.split_off(insert_index);
        self.tracks.extend(moved_tracks);
        self.tracks.extend(tail);

        out.print(format!("Moved {track_count} track(s).")).await;

        self.try_enqueue_deferred();

        if killed_current && self.is_active {
            self.play().await
        } else {
            Ok(())
        }
    }

    pub async fn when(&self, out: &MessageChannel, track_index: TrackIndex) -> CommandResult {
        let track_index = match track_index.resolve(self.current_track_index, self.tracks.len()) {
            IndexResolve::Ok(index) | IndexResolve::End(index) => index,
            IndexResolve::TooSmall(index) => {
                return Err(CommandError::Execution(format!(
                    "Track #{} doesn't exist.",
                    index + 1
                )));
            }
            IndexResolve::TooBig(index) => {
                return Err(CommandError::Execution(format!(
                    "Track #{} doesn't exist.",
                    index + 1
                )));
            }
        };

        match track_index.cmp(&self.current_track_index) {
            std::cmp::Ordering::Less => {
                return Err(CommandError::Execution(format!(
                    "Track #{} has already been played.",
                    track_index + 1
                )))
            }
            std::cmp::Ordering::Equal => {
                return Err(CommandError::Execution(format!(
                    "Track #{} is currently playing.",
                    track_index + 1
                )))
            }
            std::cmp::Ordering::Greater => {}
        }

        let mut unsure = false;

        let mut wait: Duration = if let (Some(current_handle), Some(current_track)) = (
            &self.current_track_handle,
            self.tracks.get(self.current_track_index),
        ) {
            if let (Some(duration), Ok(state)) = (
                current_track.track.duration(),
                current_handle.get_info().await,
            ) {
                let position = state.position.min(duration);
                duration - position
            } else {
                unsure = true;
                Duration::ZERO
            }
        } else {
            Duration::ZERO
        };

        for track in &self.tracks[self.current_track_index..track_index] {
            if let Some(add) = track.track.duration() {
                wait += add;
            } else {
                unsure = true;
            }
        }

        let secs = wait.as_secs();
        let unsure = if unsure {
            " (or later because some tracks have an unknown length)"
        } else {
            ""
        };
        if track_index == self.tracks.len() {
            out.print(format!(
                "The queue will end in {}:{:0>2}:{:0>2}{}",
                secs / 3600,
                secs / 60 % 60,
                secs % 60,
                unsure
            ))
            .await;
        } else {
            out.print(format!(
                "Track #{} will start in {}:{:0>2}:{:0>2}{}",
                track_index + 1,
                secs / 3600,
                secs / 60 % 60,
                secs % 60,
                unsure
            ))
            .await;
        }

        Ok(())
    }
}

struct EnqueuedTrack {
    track: Box<dyn Track>,
}

impl EnqueuedTrack {
    pub fn caption(&self) -> String {
        self.track.caption()
    }

    async fn announce(&self, out: &MessageChannel) {
        let message = format!(
            "Now playing: {}\n{}",
            self.caption(),
            self.track.track_page_url()
        );
        out.print(message).await;
    }

    async fn announce_position(&self, out: &MessageChannel, track: &TrackHandle) {
        const WIDTH: usize = 40;

        if let Some(duration) = self.track.duration() {
            let duration = duration.as_secs();

            if let Ok(state) = track.get_info().await {
                let position = state.position.as_secs().min(duration);
                let (position_minutes, position_seconds) = (position / 60, position % 60);
                let remaining = duration - position;
                let (remaining_minutes, remaining_seconds) = (remaining / 60, remaining % 60);

                #[allow(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    clippy::cast_precision_loss
                )]
                let shift = (WIDTH as f32 * position as f32 / duration as f32).round() as usize;

                let mut message = "```".to_owned();
                message.push_str("╔══════╕                                       ╒══════╗");
                message.push_str("\n║");
                (0..shift).for_each(|_| message.push('•'));
                message.push_str(&format!("[{position_minutes:0>2}:{position_seconds:0>2}│{remaining_minutes:0>2}:{remaining_seconds:0>2}]"));
                (shift..WIDTH).for_each(|_| message.push('•'));
                message.push('║');
                message.push_str("\n╚══════╛                                       ╘══════╝");
                message.push_str("```");
                out.print(message).await;
            } else {
                out.print("Could not determine the track's current state.")
                    .await;
            }
        } else {
            out.print("Could not determine the track's duration.").await;
        }
    }

    pub async fn play(
        &self,
        voice_connection: &Mutex<Call>,
        out: &MessageChannel,
        guild_id: GuildId,
    ) -> CommandResult<TrackHandle> {
        println!("EnqueuedTrack::play > current_track_handle");
        let user_track = &self.track;
        let url = user_track.playback_url().to_owned();
        println!("playing url: {url}");

        // Here, we use lazy restartable sources to make sure that we don't pay
        // for decoding, playback on tracks which aren't actually live yet.
        println!("ytdl");
        let source = Restartable::ytdl(url, false).await.map_err(|why| {
            eprintln!("ytdl error: {why}");
            CommandError::Discord(format!(
                "Failed to play track: {}\n{}",
                self.track.caption(),
                why
            ))
        })?;

        let mut voice_session = voice_connection.lock().await;
        println!("play_only_source: {source:#?}");
        let track_handle = voice_session.play_only_source(source.into());

        track_handle.add_event(Event::Track(TrackEvent::End), TrackEndNotifier { guild_id })?;

        println!("EnqueuedTrack::play > current_track_handle");

        self.announce(out).await;

        Ok(track_handle)
    }
}
