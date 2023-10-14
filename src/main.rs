#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use guild::GUILD_STATES;

#[macro_use]
extern crate lazy_static;

mod audiotool;
mod command;
mod guild;
mod message;

use audiotool::AudiotoolHttpClient;
use serenity::model::channel::Channel;
use serenity::model::event::MessageUpdateEvent;
use serenity::model::guild::{Guild, Member, PartialGuild, Role};
use serenity::model::id::{ChannelId, GuildId, RoleId};
use serenity::model::prelude::{CurrentUser, User, VoiceState};
use serenity::prelude::GatewayIntents;
use serenity::utils::validate_token;
use songbird::SerenityInit;

use serenity::client::Context;

const PREFIX: &str = "/al ";

lazy_static! {
    pub static ref AUDIOTOOL_HTTP_CLIENT: AudiotoolHttpClient = AudiotoolHttpClient::default();
}

use serenity::{
    async_trait,
    client::{Client, EventHandler},
    model::{channel::Message, gateway::Ready},
};

mod help;
mod queue;
mod track;

static STOPPED: AtomicBool = AtomicBool::new(false);

#[tokio::main]
async fn main() {
    #![allow(clippy::unreadable_literal)]
    tracing_subscriber::fmt::init();

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment variabel `DISCORD_TOKEN=<keep me secret>`");
    {
        // Configure Guild and channel roles (audiotool)
        const GUILD_ID: GuildId = GuildId(317447437650231296);
        let guild = GUILD_STATES.get_guild_state(GUILD_ID).await;
        guild
            .add_command_channel(ChannelId(344959567538421761))
            .await;
        guild
            .add_command_channel(ChannelId(683877804420628480))
            .await;
        guild
            .add_command_channel(ChannelId(429076647120207882))
            .await;
        guild
            .set_default_output_channel(ChannelId(344959567538421761))
            .await;
    }

    {
        // Configure Guild and channel roles (own server)
        const GUILD_ID: GuildId = GuildId(880206012508938330);
        let guild = GUILD_STATES.get_guild_state(GUILD_ID).await;
        guild
            .add_command_channel(ChannelId(880206012508938333))
            .await;
        guild
            .set_default_output_channel(ChannelId(880206012508938333))
            .await;
    }

    if let Err(err) = validate_token(&token) {
        panic!("invalid token given: {}", err);
    }

    let keep_alive = tokio::spawn(async {
        println!("Starting keep alive loop");
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        while !STOPPED.load(Ordering::Relaxed) {
            println!("firing keep alive …");
            AUDIOTOOL_HTTP_CLIENT.keep_alive().await;
            // println!("firing keep alive done");
            for _ in 0..60 {
                if STOPPED.load(Ordering::Relaxed) {
                    return;
                }
                interval.tick().await;
            }
        }
    });

    // TODO check whether more intents are required
    let mut client = Client::builder(
        &token,
        GatewayIntents::GUILD_VOICE_STATES
            | GatewayIntents::MESSAGE_CONTENT
            | GatewayIntents::GUILDS
            | GatewayIntents::GUILD_MESSAGES,
    )
    .event_handler(GlobalHandler)
    .register_songbird()
    .await
    .expect("Err creating client");

    tokio::spawn(async move {
        let _ = client
            .start()
            .await
            .map_err(|why| println!("Client ended: {why:?}"));
    });

    let _ = tokio::signal::ctrl_c().await;
    println!("Received Ctrl-C, shutting down.");

    STOPPED.store(true, Ordering::Relaxed);
    keep_alive
        .await
        .expect("Failed to join audiotool http client keep alive");
}

struct GlobalHandler;
#[async_trait]
impl EventHandler for GlobalHandler {
    async fn channel_create(
        &self,
        _ctx: Context,
        _channel: &serenity::model::channel::GuildChannel,
    ) {
        println!("event received: channel_create");
    }

    async fn category_create(
        &self,
        _ctx: Context,
        _category: &serenity::model::channel::ChannelCategory,
    ) {
        println!("event received: category_create");
    }

    async fn category_delete(
        &self,
        _ctx: Context,
        _category: &serenity::model::channel::ChannelCategory,
    ) {
        println!("event received: category_delete");
    }

    async fn channel_delete(
        &self,
        _ctx: Context,
        _channel: &serenity::model::channel::GuildChannel,
    ) {
        println!("event received: channel_delete");
    }

    async fn channel_pins_update(
        &self,
        _ctx: Context,
        _pin: serenity::model::event::ChannelPinsUpdateEvent,
    ) {
        println!("event received: channel_pins_update");
    }

    async fn channel_update(&self, _ctx: Context, _old: Option<Channel>, _new: Channel) {
        println!("event received: channel_update");
    }

    async fn guild_ban_addition(
        &self,
        _ctx: Context,
        _guild_id: GuildId,
        _banned_user: serenity::model::prelude::User,
    ) {
        println!("event received: guild_ban_addition");
    }

    async fn guild_ban_removal(
        &self,
        _ctx: Context,
        _guild_id: GuildId,
        _unbanned_user: serenity::model::prelude::User,
    ) {
        println!("event received: guild_ban_removal");
    }

    async fn guild_create(
        &self,
        ctx: Context,
        guild: serenity::model::guild::Guild,
        _is_new: bool,
    ) {
        println!("event received: guild_create({}:{})", guild.name, guild.id);
        let guild = GUILD_STATES.get_guild_state(guild.id).await;
        guild.set_http(ctx.http.clone()).await;
        guild.print_default("Ready to party! 🎵🕺🎶").await;
    }

    async fn guild_delete(
        &self,
        _ctx: Context,
        _incomplete: serenity::model::guild::UnavailableGuild,
        _full: Option<Guild>,
    ) {
        println!("event received: guild_delete");
    }

    async fn guild_emojis_update(
        &self,
        _ctx: Context,
        _guild_id: GuildId,
        _current_state: std::collections::HashMap<
            serenity::model::id::EmojiId,
            serenity::model::guild::Emoji,
        >,
    ) {
        println!("event received: guild_emojis_update");
    }

    async fn guild_integrations_update(&self, _ctx: Context, _guild_id: GuildId) {
        println!("event received: guild_integrations_update");
    }

    async fn guild_member_addition(
        &self,
        _ctx: Context,
        _new_member: serenity::model::guild::Member,
    ) {
        println!("event received: guild_member_addition");
    }

    async fn guild_member_removal(
        &self,
        _ctx: Context,
        _guild_id: GuildId,
        _user: User,
        _member_data_if_available: Option<Member>,
    ) {
        println!("event received: guild_member_removal");
    }

    async fn guild_member_update(
        &self,
        _ctx: Context,
        _old_if_available: Option<Member>,
        _new: Member,
    ) {
        println!("event received: guild_member_update");
    }

    async fn guild_members_chunk(
        &self,
        _ctx: Context,
        _chunk: serenity::model::event::GuildMembersChunkEvent,
    ) {
        println!("event received: guild_members_chunk");
    }

    async fn guild_role_create(&self, _ctx: Context, _new: serenity::model::guild::Role) {
        println!("event received: guild_role_create");
    }

    async fn guild_role_delete(
        &self,
        _ctx: Context,
        _guild_id: GuildId,
        _removed_role_id: RoleId,
        _removed_role_data_if_available: Option<Role>,
    ) {
        println!("event received: guild_role_delete");
    }

    async fn guild_role_update(
        &self,
        _ctx: Context,
        _old_data_if_available: Option<Role>,
        _new: Role,
    ) {
        println!("event received: guild_role_update");
    }

    async fn guild_unavailable(&self, _ctx: Context, _guild_id: GuildId) {
        println!("event received: guild_unavailable");
    }

    async fn guild_update(
        &self,
        _ctx: Context,
        _old_data_if_available: Option<Guild>,
        _new_but_incomplete: PartialGuild,
    ) {
        println!("event received: guild_update");
    }

    async fn invite_create(&self, _ctx: Context, _data: serenity::model::event::InviteCreateEvent) {
        println!("event received: invite_create");
    }

    async fn invite_delete(&self, _ctx: Context, _data: serenity::model::event::InviteDeleteEvent) {
        println!("event received: invite_delete");
    }

    async fn message(&self, ctx: Context, new_message: Message) {
        // prevent feedback loops
        if new_message.is_own(&ctx) {
            return;
        }
        println!("event received: message");

        // make sure the message was properly addressed
        let command_line = {
            if new_message.content == PREFIX.trim() {
                Some("")
            } else {
                new_message.content.strip_prefix(PREFIX)
            }
        };

        if let Some(command_line) = command_line {
            // make sure the message has a guild attached
            if let Some(guild_id) = new_message.guild(&ctx).map(|guild| guild.id) {
                let guild = GUILD_STATES.get_guild_state(guild_id).await;
                guild
                    .handle_command_line(command_line, &ctx, &new_message)
                    .await;
            }
        }
    }

    async fn message_delete(
        &self,
        _ctx: Context,
        _channel_id: ChannelId,
        _deleted_message_id: serenity::model::id::MessageId,
        _guild_id: Option<GuildId>,
    ) {
        println!("event received: message_delete");
    }

    async fn message_delete_bulk(
        &self,
        _ctx: Context,
        _channel_id: ChannelId,
        _multiple_deleted_messages_ids: Vec<serenity::model::id::MessageId>,
        _guild_id: Option<GuildId>,
    ) {
        println!("event received: message_delete_bulk");
    }

    async fn message_update(
        &self,
        _ctx: Context,
        _old_if_available: Option<Message>,
        _new: Option<Message>,
        _event: MessageUpdateEvent,
    ) {
        println!("event received: message_update");
    }

    async fn reaction_add(&self, _ctx: Context, _add_reaction: serenity::model::channel::Reaction) {
        println!("event received: reaction_add");
    }

    async fn reaction_remove(
        &self,
        _ctx: Context,
        _removed_reaction: serenity::model::channel::Reaction,
    ) {
        println!("event received: reaction_remove");
    }

    async fn reaction_remove_all(
        &self,
        _ctx: Context,
        _channel_id: ChannelId,
        _removed_from_message_id: serenity::model::id::MessageId,
    ) {
        println!("event received: reaction_remove_all");
    }

    async fn presence_replace(&self, _ctx: Context, _: Vec<serenity::model::prelude::Presence>) {
        println!("event received: presence_replace");
    }

    async fn presence_update(&self, _ctx: Context, _new_data: serenity::model::gateway::Presence) {
        println!("event received: presence_update");
    }

    async fn ready(&self, _ctx: Context, _data_about_bot: Ready) {
        println!("event received: ready");
    }

    async fn resume(&self, _ctx: Context, _: serenity::model::event::ResumedEvent) {
        println!("event received: resume");
    }

    async fn shard_stage_update(
        &self,
        _ctx: Context,
        _: serenity::client::bridge::gateway::event::ShardStageUpdateEvent,
    ) {
        println!("event received: shard_stage_update");
    }

    // async fn typing_start(&self, _ctx: Context, _: serenity::model::event::TypingStartEvent) {
    //     println!("event received: typing_start");
    // }

    async fn unknown(&self, _ctx: Context, _name: String, _raw: serde_json::Value) {
        println!("event received: unknown");
    }

    async fn user_update(&self, _ctx: Context, _old_data: CurrentUser, _new: CurrentUser) {
        println!("event received: user_update");
    }

    async fn voice_server_update(
        &self,
        _ctx: Context,
        _: serenity::model::event::VoiceServerUpdateEvent,
    ) {
        println!("event received: voice_server_update");
    }

    async fn voice_state_update(&self, _ctx: Context, _old: Option<VoiceState>, _new: VoiceState) {
        println!("event received: voice_state_update");
    }

    async fn webhook_update(
        &self,
        _ctx: Context,
        _guild_id: GuildId,
        _belongs_to_channel_id: ChannelId,
    ) {
        println!("event received: webhook_update");
    }

    async fn stage_instance_create(
        &self,
        _ctx: Context,
        _stage_instance: serenity::model::channel::StageInstance,
    ) {
        println!("event received: stage_instance_create");
    }

    async fn stage_instance_update(
        &self,
        _ctx: Context,
        _stage_instance: serenity::model::channel::StageInstance,
    ) {
        println!("event received: stage_instance_update");
    }

    async fn stage_instance_delete(
        &self,
        _ctx: Context,
        _stage_instance: serenity::model::channel::StageInstance,
    ) {
        println!("event received: stage_instance_delete");
    }

    async fn thread_create(&self, _ctx: Context, _thread: serenity::model::channel::GuildChannel) {
        println!("event received: thread_create");
    }

    async fn thread_update(&self, _ctx: Context, _thread: serenity::model::channel::GuildChannel) {
        println!("event received: thread_update");
    }

    async fn thread_delete(
        &self,
        _ctx: Context,
        _thread: serenity::model::channel::PartialGuildChannel,
    ) {
        println!("event received: thread_delete");
    }

    async fn thread_list_sync(
        &self,
        _ctx: Context,
        _thread_list_sync: serenity::model::event::ThreadListSyncEvent,
    ) {
        println!("event received: thread_list_sync");
    }

    async fn thread_member_update(
        &self,
        _ctx: Context,
        _thread_member: serenity::model::guild::ThreadMember,
    ) {
        println!("event received: thread_member_update");
    }

    async fn thread_members_update(
        &self,
        _ctx: Context,
        _thread_members_update: serenity::model::event::ThreadMembersUpdateEvent,
    ) {
        println!("event received: thread_members_update");
    }
}
