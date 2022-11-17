use std::{fmt, sync::Arc};

use serenity::{http::Http, model::{channel::Message, id::ChannelId}};

#[derive(Default, Clone)]
pub struct MessageChannel{
    channel: Option<ChannelId>,
    http: Option<Arc<Http>>,
}

impl MessageChannel {

    pub fn new(channel: ChannelId, http: Arc<Http>) -> Self {
        Self {
            channel: Some(channel),
            http: Some(http),
        }
    }

    pub fn set_channel(&mut self, channel: ChannelId) {
        self.channel = Some(channel);
    }

    pub fn set_http(&mut self, http: Arc<Http>) {
        self.http = Some(http);
    }

    pub async fn print(&self, message: impl fmt::Display) {
        self.print_raw(message).await
    }

    async fn print_raw(&self, message: impl fmt::Display) {
        if let Self{ channel: Some(channel), http: Some(ref http)} = *self {

            let message = message.to_string();
            let mut lines: Vec::<&str> = message.lines().collect();
            let mut msg = lines.remove(0).to_string();
            for line in lines {
                if msg.len() + 1 + line.len() >= 1950 { // real limit is 2000
                    Self::check_msg(channel.say(&http, msg.clone()).await);
                    msg.clear();
                }
                if !msg.is_empty() {
                    msg.push('\n');
                }
                msg.push_str(line);
            }

            if !msg.is_empty() {
                Self::check_msg(channel.say(&http, msg).await);
            }
        }
    }

    /// Checks that a message successfully sent; if not, then logs why to stdout.
    fn check_msg(result: serenity::Result<Message>) {
        if let Err(why) = result {
            println!("Error sending message: {:?}", why);
        }
    }

}