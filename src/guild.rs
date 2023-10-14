use lazy_static::lazy_static;
use std::{
    collections::{HashMap, HashSet},
    fmt,
    fmt::Write,
    sync::Arc,
    time::Duration,
};

use serenity::{
    client::Context,
    http::Http,
    model::{
        channel::{Message, ReactionType},
        id::{ChannelId, GuildId},
        prelude::User,
    },
};

#[allow(clippy::wildcard_imports)]
use crate::{
    command::*,
    help::HelpTopic,
    message::MessageChannel,
    queue::Queue,
    track::{
        audiotool::{
            AudiotoolAlbumDispatcher, AudiotoolGenreChartsDispatcher,
            AudiotoolSingleChartsDispatcher, AudiotoolTrackRefDispatcher,
        },
        selection::{TrackIndex, TrackIndexSelection},
        youtube::YoutubeTrackRefDispatcher,
        TrackRefDispatcher,
    },
};

use tokio::sync::Mutex;

lazy_static! {
    pub static ref GUILD_STATES: GuildStates = GuildStates::new();
}

pub struct GuildStates {
    guild_states: Mutex<HashMap<GuildId, GuildStateHandle>>,
}

impl GuildStates {
    pub fn new() -> Self {
        Self {
            guild_states: Mutex::default(),
        }
    }

    pub async fn get_guild_state(&self, guild_id: GuildId) -> GuildStateHandle {
        let mut queues = self.guild_states.lock().await;
        queues
            .entry(guild_id)
            .or_insert_with(|| GuildStateHandle {
                guild_id,
                state: Arc::new(Mutex::new(GuildState::new(guild_id))),
            })
            .clone()
    }
}

#[derive(Clone)]
pub struct GuildStateHandle {
    #[allow(unused)]
    guild_id: GuildId,
    state: Arc<Mutex<GuildState>>,
}

impl GuildStateHandle {
    pub async fn handle_command_line(&self, command_line: &str, ctx: &Context, msg: &Message) {
        match self.execute_command(command_line, ctx, msg).await {
            Ok(reaction) => {
                if let Some(reaction) = reaction {
                    let _ = msg.react(ctx, reaction).await;
                }
            }
            Err(err) => {
                let reply_channel = MessageChannel::new(msg.channel_id, ctx.http.clone());
                match err {
                    CommandError::Usage { message, topic } => {
                        reply_channel
                            .print(format!("{}\n{}", message, topic.message()))
                            .await;
                        //reply_channel.print(topic.message()).await;
                        // if let Some(topic) = topic {
                        //     match topic.parse::<HelpTopic>() {
                        //     }
                        //     let _ = HelpTopicCommand::help_topic(&reply_channel, topic).await;
                        // } else {
                        //     let _ = Command::help(&reply_channel).await;
                        // }
                    }
                    CommandError::Discord(message) => {
                        let message = format!("Internal error: {message}");
                        reply_channel.print(&message).await;
                        println!("{message}");
                    }
                    CommandError::UserVoiceChannelRequired => {
                        reply_channel
                            .print("You need to be in a voice channel to use this command")
                            .await;
                    }
                    CommandError::BotVoiceChannelRequired => {
                        reply_channel.print("I need to be in a voice chat to play a track. Use the `join` command to invite me.").await;
                    }
                    CommandError::Execution(message) => {
                        reply_channel.print(message).await;
                    }
                    CommandError::NotInCommandChannel => {
                        reply_channel
                            .print("This channel isn't available for bot commands.")
                            .await;
                    }
                }
                let _ = msg.react(ctx, ReactionType::Unicode("🚫".to_owned())).await;
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    pub async fn execute_command(
        &self,
        command_line: &str,
        ctx: &Context,
        msg: &Message,
    ) -> CommandResult<Option<ReactionType>> {
        if !self.is_command_channel(msg.channel_id).await {
            return Err(CommandError::NotInCommandChannel);
        }

        let reply_channel = MessageChannel::new(msg.channel_id, ctx.http.clone());

        let command = Command::from_str(command_line, &reply_channel).await;
        if command.requires_vc() {
            let guild = msg
                .guild(&ctx.cache)
                .ok_or_else(|| CommandError::Discord("Could not retrieve guild".to_owned()))?;
            let is_in_voice_channel = guild
                .voice_states
                .get(&msg.author.id)
                .and_then(|voice_state| voice_state.channel_id)
                .is_some();
            if !is_in_voice_channel {
                return Err(CommandError::UserVoiceChannelRequired);
            }
        }

        match command {
            Command::Help(topic) => {
                reply_channel.print(topic.message()).await;
                Ok(None)
            }
            Command::Join => self
                .command_join(&reply_channel, ctx, msg)
                .await
                .map(|()| Some(ReactionType::Unicode("🎧".to_owned()))),
            Command::Leave => self
                .command_leave(&reply_channel, ctx)
                .await
                .map(|()| Some(ReactionType::Unicode("👋".to_owned()))),
            Command::Enqueue(tracks) => self
                .command_enqueue(&reply_channel, tracks, &msg.author)
                .await
                .map(|()| Some(ReactionType::Unicode("✅".to_owned()))),
            Command::Pause => self
                .command_pause()
                .await
                .map(|()| Some(ReactionType::Unicode("⏸".to_owned()))),
            Command::Resume => self
                .command_resume()
                .await
                .map(|()| Some(ReactionType::Unicode("⏯".to_owned()))),
            Command::Play => self
                .command_play()
                .await
                .map(|()| Some(ReactionType::Unicode("🔊".to_owned()))),
            Command::Stop => self
                .command_stop()
                .await
                .map(|()| Some(ReactionType::Unicode("⏹".to_owned()))),
            Command::Print(ref selection) => self
                .command_print(&reply_channel, selection)
                .await
                .map(|()| None),
            Command::Goto(index) => self
                .command_goto(index)
                .await
                .map(|()| Some(ReactionType::Unicode("⏬".to_owned()))),
            Command::Next => self
                .command_next()
                .await
                .map(|()| Some(ReactionType::Unicode("⏭".to_owned()))),
            Command::Prev => self
                .command_prev()
                .await
                .map(|()| Some(ReactionType::Unicode("⏮".to_owned()))),
            Command::Remove(ref selection) => self
                .command_remove(&reply_channel, selection)
                .await
                .map(|()| Some(ReactionType::Unicode("❎".to_owned()))),
            Command::Seek(position) => self
                .command_seek(position)
                .await
                .map(|()| Some(ReactionType::Unicode("🔎".to_owned()))),
            Command::Now => self.command_now(&reply_channel).await.map(|()| None),
            Command::Reverse(ref selection) => self
                .command_reverse(selection)
                .await
                .map(|()| Some(ReactionType::Unicode("🔃".to_owned()))),
            Command::Quota(quota) => {
                if let Some(quota) = quota {
                    let quota = if quota == 0 { None } else { Some(quota) };
                    self.command_set_quota(quota)
                        .await
                        .map(|()| Some(ReactionType::Unicode("🛑".to_owned())))
                } else {
                    self.command_print_quota(&reply_channel)
                        .await
                        .map(|()| None)
                }
            }
            Command::Move(ref selection, index) => self
                .command_move(&reply_channel, selection, index)
                .await
                .map(|()| Some(ReactionType::Unicode("🔀".to_owned()))),
            Command::When(index) => self
                .command_when(&reply_channel, index)
                .await
                .map(|()| None),
        }
    }

    pub async fn is_command_channel(&self, channel: ChannelId) -> bool {
        self.state.lock().await.is_command_channel(channel)
    }

    pub async fn handle_track_end(&self) {
        self.state.lock().await.queue.handle_track_end().await;
    }

    pub async fn command_pause(&self) -> CommandResult {
        self.state.lock().await.queue.pause()
    }

    pub async fn command_stop(&self) -> CommandResult {
        self.state.lock().await.queue.stop()
    }

    pub async fn command_resume(&self) -> CommandResult {
        self.state.lock().await.queue.resume()
    }

    pub async fn command_play(&self) -> CommandResult {
        self.state.lock().await.queue.play().await
    }

    pub async fn command_print(
        &self,
        reply_channel: &MessageChannel,
        tracks: &TrackIndexSelection,
    ) -> CommandResult {
        self.state
            .lock()
            .await
            .queue
            .print(reply_channel, tracks)
            .await
    }

    pub async fn command_goto(&self, track_index: TrackIndex) -> CommandResult {
        self.state.lock().await.queue.goto(track_index).await
    }

    pub async fn command_next(&self) -> CommandResult {
        self.state.lock().await.queue.next().await
    }

    pub async fn command_prev(&self) -> CommandResult {
        self.state.lock().await.queue.prev().await
    }

    pub async fn command_remove(
        &self,
        reply_channel: &MessageChannel,
        tracks: &TrackIndexSelection,
    ) -> CommandResult {
        self.state
            .lock()
            .await
            .queue
            .remove(reply_channel, tracks)
            .await
    }

    async fn command_join(
        &self,
        reply_channel: &MessageChannel,
        ctx: &Context,
        msg: &Message,
    ) -> CommandResult {
        self.state.lock().await.join(reply_channel, ctx, msg).await
    }

    async fn command_leave(&self, reply_channel: &MessageChannel, ctx: &Context) -> CommandResult {
        self.state.lock().await.leave(reply_channel, ctx).await
    }

    async fn command_enqueue(
        &self,
        reply_channel: &MessageChannel,
        track_refs: Vec<(String, Option<String>)>,
        user: &User,
    ) -> CommandResult {
        self.state
            .lock()
            .await
            .enqueue(reply_channel, track_refs, user)
            .await
    }

    async fn command_seek(&self, position: Duration) -> CommandResult {
        self.state.lock().await.queue.seek(position)
    }

    pub async fn command_now(&self, reply_channel: &MessageChannel) -> CommandResult {
        self.state.lock().await.queue.now(reply_channel).await
    }

    pub async fn command_reverse(&self, tracks: &TrackIndexSelection) -> CommandResult {
        self.state.lock().await.queue.reverse(tracks).await
    }

    pub async fn command_set_quota(&self, quota: Option<usize>) -> CommandResult {
        self.state.lock().await.queue.set_quota(quota)
    }

    pub async fn command_print_quota(&self, reply_channel: &MessageChannel) -> CommandResult {
        self.state
            .lock()
            .await
            .queue
            .print_quota(reply_channel)
            .await
    }

    pub async fn command_move(
        &self,
        reply_channel: &MessageChannel,
        tracks: &TrackIndexSelection,
        index: TrackIndex,
    ) -> CommandResult {
        self.state
            .lock()
            .await
            .queue
            .move_tracks(reply_channel, tracks, index)
            .await
    }

    pub async fn command_when(
        &self,
        reply_channel: &MessageChannel,
        track_index: TrackIndex,
    ) -> CommandResult {
        self.state
            .lock()
            .await
            .queue
            .when(reply_channel, track_index)
            .await
    }

    pub async fn print_default(&self, message: impl fmt::Display) {
        println!("print {message}");
        self.state.lock().await.print(message).await;
    }

    pub async fn set_http(&self, http: Arc<Http>) {
        println!("set_http");
        self.state.lock().await.set_http(http);
    }

    pub async fn set_default_output_channel(&self, channel_id: ChannelId) {
        println!("set_default_output_channel");
        self.state
            .lock()
            .await
            .set_default_output_channel(channel_id);
    }

    pub async fn add_command_channel(&self, channel_id: ChannelId) {
        println!("add_command_channel");
        self.state.lock().await.add_command_channel(channel_id);
    }
}

struct GuildState {
    id: GuildId,
    queue: Queue,
    default_reply_channel: MessageChannel,
    command_channels: HashSet<ChannelId>,
}

impl GuildState {
    pub fn new(id: GuildId) -> Self {
        let default_reply_channel = MessageChannel::default();
        Self {
            id,
            queue: Queue::new(id, default_reply_channel.clone()),
            default_reply_channel,
            command_channels: HashSet::new(),
        }
    }

    pub fn is_command_channel(&mut self, channel: ChannelId) -> bool {
        self.command_channels.contains(&channel)
    }

    pub async fn print(&mut self, message: impl fmt::Display) {
        self.default_reply_channel.print(message).await;
    }

    pub fn set_http(&mut self, http: Arc<Http>) {
        self.default_reply_channel.set_http(http.clone());
        self.queue.set_http(http);
    }

    pub fn set_default_output_channel(&mut self, channel_id: ChannelId) {
        self.default_reply_channel.set_channel(channel_id);
        self.queue.set_default_message_channel(channel_id);
    }

    pub fn add_command_channel(&mut self, channel_id: ChannelId) {
        self.command_channels.insert(channel_id);
    }

    async fn join(&mut self, out: &MessageChannel, ctx: &Context, msg: &Message) -> CommandResult {
        let guild = msg
            .guild(&ctx.cache)
            .ok_or_else(|| CommandError::Discord("Could not retrieve guild".to_owned()))?;

        let channel_id = guild
            .voice_states
            .get(&msg.author.id)
            .and_then(|voice_state| voice_state.channel_id);

        let Some(connect_to) = channel_id else {
            out.print("Not in a voice channel").await;
            return Err(CommandError::UserVoiceChannelRequired);
        };

        let songbird = songbird::get(ctx)
            .await
            .expect("Songbird Voice client placed in at initialisation.")
            .clone();

        let (voice_connection, _) = songbird.join(self.id, connect_to).await;

        self.queue.connect(voice_connection);
        let _ = self.queue.deafen().await;

        // this is a public announcement and not a direct reply to the issuer
        self.print("Hello my friends! Stay a while and listen!")
            .await;

        Ok(())
    }

    // TODO move into wrapped guild
    async fn leave(&mut self, out: &MessageChannel, ctx: &Context) -> CommandResult {
        println!("command: leave");

        let manager = songbird::get(ctx)
            .await
            .expect("Songbird Voice client placed in at initialisation.")
            .clone();
        let has_handler = manager.get(self.id).is_some();

        if has_handler {
            if let Err(e) = manager.remove(self.id).await {
                out.print(format!("Failed: {e:?}")).await;
            }
        } else {
            out.print("Not in a voice channel").await;
        }

        self.queue.disconnect();

        Ok(())
    }

    async fn enqueue(
        &mut self,
        out: &MessageChannel,
        track_refs: Vec<(String, Option<String>)>,
        user: &User,
    ) -> CommandResult {
        if track_refs.is_empty() {
            return Err(CommandError::Usage {
                message: "Please specify an URL or another locator for the track to enqeue.\ne.g. `enqueue https://example.com/path/to/track`".to_owned(),
                topic: HelpTopic::Enqueue,
            });
        }

        let mut tracks = Vec::new();

        let dispatchers: Vec<Box<dyn TrackRefDispatcher>> = vec![
            Box::new(AudiotoolTrackRefDispatcher {}),
            Box::new(AudiotoolSingleChartsDispatcher {}),
            Box::new(AudiotoolGenreChartsDispatcher {}),
            Box::new(AudiotoolAlbumDispatcher {}),
            Box::new(YoutubeTrackRefDispatcher {}),
        ];

        let mut errors = String::new();

        'next_track_ref: for (track_ref, comment) in track_refs {
            for dispatcher in &dispatchers {
                if let Some(maybe_tracks) =
                    dispatcher.dispatch(&track_ref, comment.clone(), user).await
                {
                    for track in maybe_tracks {
                        match track {
                            Ok(track) => {
                                tracks.push(track);
                            }
                            Err(err) => {
                                writeln!(errors, "{err}").unwrap();
                            }
                        }
                    }
                    continue 'next_track_ref;
                }
            }

            out.print(format!("Failed to interpret `{track_ref}` as a track reference.\nMight be from an unsupported provider.")).await;
        }

        let success = !tracks.is_empty();

        for user_track in tracks {
            self.queue.append(user_track).await;
        }

        if success {
            Ok(())
        } else {
            Err(CommandError::Execution(errors))
        }
    }
}
