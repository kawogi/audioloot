use songbird::{error::JoinError, tracks::TrackError};

use crate::{help::HelpTopic, message::MessageChannel, track::selection::{TrackIndex, TrackIndexRange, TrackIndexSelection}};
use std::{fmt, time::Duration};

pub const CMD_HELP: &str = "help";
pub const CMD_JOIN: &str = "join";
pub const CMD_LEAVE: &str = "leave";
pub const CMD_ENQUEUE: &str = "+";
pub const CMD_PAUSE: &str = "pause";
pub const CMD_RESUME: &str = "resume";
pub const CMD_STOP: &str = "stop";
pub const CMD_PLAY: &str = "play";
pub const CMD_PRINT: &str = "print";
pub const CMD_GOTO: &str = "goto";
pub const CMD_NEXT: &str = "next";
pub const CMD_PREV: &str = "prev";
pub const CMD_REMOVE: &str = "-";
pub const CMD_SEEK: &str = "seek";
pub const CMD_NOW: &str = "now";
pub const CMD_REVERSE: &str = "reverse";
pub const CMD_QUOTA: &str = "quota";
pub const CMD_MOVE: &str = "move";
pub const CMD_WHEN: &str = "when";


pub enum Command {
    Help(HelpTopic),
    Join,
    Leave,
    Enqueue(Vec<(String, Option<String>)>),
    Pause,
    Resume,
    Play,
    Stop,
    Print(TrackIndexSelection),
    Goto(TrackIndex),
    Next,
    Prev,
    Remove(TrackIndexSelection),
    Seek(Duration),
    Now,
    Reverse(TrackIndexSelection),
    Quota(Option<usize>),
    Move(TrackIndexSelection, TrackIndex),
    When(TrackIndex),
}

pub type CommandResult<T = ()> = Result<T, CommandError>;

#[derive(Debug)]
pub enum CommandError {
    Usage { message: String, topic: HelpTopic },
    Execution(String),
    UserVoiceChannelRequired,
    BotVoiceChannelRequired,
    Discord(String),
    NotInCommandChannel,
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO make this better
        write!(f, "{:?}", self)
    }
}

impl From<TrackError> for CommandError {
    fn from(err: TrackError) -> Self {
        Self::Discord(err.to_string())
    }
}

impl From<JoinError> for CommandError {
    fn from(err: JoinError) -> Self {
        Self::Discord(err.to_string())
    }
}

impl Command {

    pub async fn from_str(command_line: &str, reply_channel: &MessageChannel) -> Self {

        let mut parts = command_line.splitn(2, &[' ', '\n', '\t'][..]);
        let command = parts.next().unwrap();
        let args = parts.next()
                .map(|args| args.trim())
                .and_then(|args| (!args.is_empty()).then_some(args));
        println!("command {}({})", command, args.unwrap_or_default());
        match command {
            "" | CMD_HELP => {
                if let Some(topic) = args {
                    match topic.parse::<HelpTopic>() {
                        Ok(topic) => Command::Help(topic),
                        Err(_) => {
                            reply_channel.print(format!("Unknown help topic: {}", topic)).await;
                            Command::Help(HelpTopic::General)
                        }
                    }
                } else {
                    Command::Help(HelpTopic::General)
                }
            }
            CMD_JOIN => Command::Join,
            CMD_LEAVE => Command::Leave,
            CMD_ENQUEUE => {
                let mut tracks: Vec<(String, Option<String>)> = Vec::new();
                let args = args.unwrap_or_default();
                for line in args.lines() {
                    let mut parts = line.trim_matches(&[' ', '\t'][..]).splitn(2, &[' ', '\t'][..]);
                    let url = parts.next().unwrap().trim_matches(&[' ', '`', '\t'][..]);
                    if url.is_empty() {
                        continue;
                    }
                    let comment = parts.next().map(|comment| comment.trim_matches(&[' ', '`', '\t'][..]));
                    let comment = comment.and_then(|comment| if comment.is_empty() { None } else { Some(comment.to_owned()) });
                    tracks.push((url.to_owned(), comment));
                }

                Command::Enqueue(tracks)
            },
            CMD_PAUSE => Command::Pause,
            CMD_RESUME => Command::Resume,
            CMD_STOP => Command::Stop,
            CMD_PLAY => {
                if args.is_some() {
                    reply_channel.print(format!("play is meant to (re-)start the playback.\nIf you meant to add more tracks to the queue, use `{}` instead.", CMD_ENQUEUE)).await;
                    Command::Help(HelpTopic::Play)
                } else {
                    Command::Play
                }
            }
            CMD_PRINT => {
                match TrackIndexSelection::parse_str(args.unwrap_or_default()) {
                    Ok(tracks) => {
                        if tracks.is_empty() {
                            let start = TrackIndex::Current(-2);
                            let end = TrackIndex::Current(15);
                            Command::Print(TrackIndexSelection(vec![TrackIndexRange::Range(start, end)]))
                        } else {
                            Command::Print(tracks)
                        }
                    }
                    Err(err) => {
                        reply_channel.print(err).await;
                        Command::Help(HelpTopic::Print)
                    }
                }
            }
            CMD_GOTO => match args.map(|args| args.parse::<TrackIndex>()) {
                    Some(Ok(index)) => {
                        Command::Goto(index)
                    }
                    Some(Err(err)) => {
                        reply_channel.print(err).await;
                        Command::Help(HelpTopic::Goto)
                    }
                    None => {
                        reply_channel.print("this command requires a track index".to_string()).await;
                        Command::Help(HelpTopic::Goto)
                    }
            }
            CMD_NEXT => Command::Next,
            CMD_PREV => Command::Prev,
            CMD_REMOVE => {
                match TrackIndexSelection::parse_str(args.unwrap_or_default()) {
                    Ok(tracks) => {
                        if tracks.is_empty() {
                            reply_channel.print("please specify the tracks to remove".to_string()).await;
                            Command::Help(HelpTopic::Remove)
                        } else {
                            Command::Remove(tracks)
                        }
                    }
                    Err(err) => {
                        reply_channel.print(err).await;
                        Command::Help(HelpTopic::Remove)
                    }
                }
            }
            CMD_SEEK => {
                if let Some(time) = args {
                    let mut parts = time.splitn(2, ':');
                    match (parts.next().unwrap(), parts.next()) {
                        (seconds, None) => {
                            match seconds.parse::<u64>() {
                                Ok(seconds) => {
                                    Command::Seek(Duration::from_secs(seconds))
                                }
                                Err(_err) => {
                                    reply_channel.print("invalid seek position".to_string()).await;
                                    Command::Help(HelpTopic::Seek)
                                }
                            }
                        },
                        (minutes, Some(seconds)) => {
                            match (minutes.parse::<u64>(), seconds.parse::<u64>()) {
                                (Ok(minutes), Ok(seconds)) => {
                                    Command::Seek(Duration::from_secs(minutes * 60 + seconds))
                                }
                                _ => {
                                    reply_channel.print("invalid seek position".to_string()).await;
                                    Command::Help(HelpTopic::Seek)
                                }
                            }
                        },
                    }
                    
                } else {
                    reply_channel.print("please specify a seek position".to_string()).await;
                    Command::Help(HelpTopic::Seek)
                }
            },
            CMD_NOW => {
                Command::Now
            }
            CMD_REVERSE => {
                match TrackIndexSelection::parse_str(args.unwrap_or_default()) {
                    Ok(tracks) => {
                        if tracks.is_empty() {
                            reply_channel.print("please specify the tracks to reverse".to_string()).await;
                            Command::Help(HelpTopic::Reverse)
                        } else {
                            Command::Reverse(tracks)
                        }
                    }
                    Err(err) => {
                        reply_channel.print(err).await;
                        Command::Help(HelpTopic::Reverse)
                    }
                }
            }
            CMD_QUOTA => {
                if let Some(args) = args {
                    if args == "off" {
                        Command::Quota(Some(0))
                    } else if let Ok(quota) = args.parse::<usize>() {
                        Command::Quota(Some(quota))
                    } else {
                        reply_channel.print("please specify a valid quota".to_string()).await;
                        Command::Help(HelpTopic::Quota)
                    }
                } else {
                    Command::Quota(None)
                }
            }
            CMD_MOVE => {
                let mut src_dst = args.unwrap_or_default().splitn(2, "to");
                match TrackIndexSelection::parse_str(src_dst.next().unwrap_or_default()) {
                    Ok(tracks) => {
                        if tracks.is_empty() {
                            reply_channel.print("please specify the tracks to move".to_string()).await;
                            Command::Help(HelpTopic::Move)
                        } else {
                            match src_dst.next().map(|dst| dst.parse::<TrackIndex>()) {
                                Some(Ok(dst)) => {
                                    Command::Move(tracks, dst)
                                }
                                Some(Err(_err)) => {
                                    reply_channel.print("please specify a valid destination".to_string()).await;
                                    Command::Help(HelpTopic::Move)
                                }
                                None => {
                                    reply_channel.print("please specify the destination with `to`".to_string()).await;
                                    Command::Help(HelpTopic::Move)
                                }
                            }
                        }
                    }
                    Err(err) => {
                        reply_channel.print(err).await;
                        Command::Help(HelpTopic::Move)
                    }
                }

            }
            CMD_WHEN => {
                match args.map(|args| args.parse::<TrackIndex>()) {
                    Some(Ok(index)) => {
                        Command::When(index)
                    }
                    Some(Err(err)) => {
                        reply_channel.print(err).await;
                        Command::Help(HelpTopic::When)
                    }
                    None => {
                        reply_channel.print("this command requires a track index".to_string()).await;
                        Command::Help(HelpTopic::When)
                    }
                }
            },
            
            _ => {
                reply_channel.print(format!("Unknown command: {}", command)).await;
                Command::Help(HelpTopic::General)
            }
        }
    }

    pub fn requires_vc(&self) -> bool {
        match *self {
            Command::Help(_) | Command::Print(_) | Command::Now | Command::When(_) => false,

            Command::Join | Command::Leave | Command::Enqueue(_) | Command::Pause | Command::Resume |
            Command::Play | Command::Stop | Command::Goto(_) | Command::Next | Command::Prev |
            Command::Remove(_) | Command::Seek(_) | Command::Reverse(_) | Command::Quota(_) |
            Command::Move(_, _) => true,
        }
    }

}
