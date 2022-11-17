use std::{fmt, fmt::Write, str::FromStr};

use crate::{PREFIX, command::*};

pub const HELP_HELP: &str = CMD_HELP;
pub const HELP_JOIN: &str = CMD_JOIN;
pub const HELP_LEAVE: &str = CMD_LEAVE;
pub const HELP_ENQUEUE: &str = CMD_ENQUEUE;
pub const HELP_PAUSE: &str = CMD_PAUSE;
pub const HELP_RESUME: &str = CMD_RESUME;
pub const HELP_STOP: &str = CMD_STOP;
pub const HELP_PLAY: &str = CMD_PLAY;
pub const HELP_PRINT: &str = CMD_PRINT;
pub const HELP_GOTO: &str = CMD_GOTO;
pub const HELP_NEXT: &str = CMD_NEXT;
pub const HELP_PREV: &str = CMD_PREV;
pub const HELP_REMOVE: &str = CMD_REMOVE;
pub const HELP_SEEK: &str = CMD_SEEK;
pub const HELP_NOW: &str = CMD_NOW;
pub const HELP_REVERSE: &str = CMD_REVERSE;
pub const HELP_QUOTA: &str = CMD_QUOTA;
pub const HELP_MOVE: &str = CMD_MOVE;
pub const HELP_WHEN: &str = CMD_WHEN;

pub const HELP_TRACK_INDEX: &str = "track-index";
pub const HELP_TRACK_RANGE: &str = "track-range";
pub const HELP_TRACK_SET: &str = "track-set";


#[derive(Debug, Clone, Copy)]
pub enum HelpTopic {
    General,
    Help,
    Join,
    Leave,
    Enqueue,
    Pause,
    Resume,
    Stop,
    Play,
    Print,
    Goto,
    Next,
    Prev,
    Remove,
    Seek,
    Now,
    Reverse,
    Quota,
    Move,
    When,
    TrackIndex,
    TrackRange,
    TrackSet,
}

impl HelpTopic {

    pub fn message(self) -> String {
        match self {
            HelpTopic::General => {
                let mut help = String::new();
                writeln!(help, "All commands follow the syntax: `{}<command> [<args>]`", PREFIX).unwrap();
                writeln!(help, "{}", HelpTopic::Help.overview()).unwrap();

                writeln!(help, "**Playback control**").unwrap();
                writeln!(help, "{}", HelpTopic::Play.overview()).unwrap();
                writeln!(help, "{}", HelpTopic::Stop.overview()).unwrap();
                writeln!(help, "{}", HelpTopic::Next.overview()).unwrap();
                writeln!(help, "{}", HelpTopic::Prev.overview()).unwrap();
                writeln!(help, "{}", HelpTopic::Goto.overview()).unwrap();
                writeln!(help, "{}", HelpTopic::Pause.overview()).unwrap();
                writeln!(help, "{}", HelpTopic::Resume.overview()).unwrap();
                writeln!(help, "{}", HelpTopic::Seek.overview()).unwrap();

                writeln!(help, "**Queue management**").unwrap();
                writeln!(help, "{}", HelpTopic::Enqueue.overview()).unwrap();
                writeln!(help, "{}", HelpTopic::Remove.overview()).unwrap();
                writeln!(help, "{}", HelpTopic::Reverse.overview()).unwrap();
                writeln!(help, "{}", HelpTopic::Move.overview()).unwrap();
                writeln!(help, "{}", HelpTopic::Quota.overview()).unwrap();

                writeln!(help, "**Status info**").unwrap();
                writeln!(help, "{}", HelpTopic::Print.overview()).unwrap();
                writeln!(help, "{}", HelpTopic::Now.overview()).unwrap();
                writeln!(help, "{}", HelpTopic::When.overview()).unwrap();

                writeln!(help, "**Bot control**").unwrap();
                writeln!(help, "{}", HelpTopic::Join.overview()).unwrap();
                writeln!(help, "{}", HelpTopic::Leave.overview()).unwrap();

                writeln!(help, "**Other help topics**").unwrap();
                writeln!(help, "`{}{} {}` - {}", PREFIX, CMD_HELP, HELP_TRACK_INDEX, HelpTopic::TrackIndex.overview()).unwrap();
                writeln!(help, "`{}{} {}` - {}", PREFIX, CMD_HELP, HELP_TRACK_RANGE, HelpTopic::TrackRange.overview()).unwrap();
                writeln!(help, "`{}{} {}` - {}", PREFIX, CMD_HELP, HELP_TRACK_SET, HelpTopic::TrackSet.overview()).unwrap();

                help
            }
            HelpTopic::Help => {
                let mut help = String::new();
                writeln!(help, "{}", self.overview()).unwrap();
                writeln!(help, "e.g. `{}{} {}`", PREFIX, CMD_HELP, HELP_PLAY).unwrap();
                write!(help, "valid help topics are: ").unwrap();
                let topics = [Self::Help, Self::Join, Self::Leave, Self::Enqueue, Self::Pause, Self::Resume, Self::Stop, Self::Play, Self::Print, Self::Goto, Self::Next, Self::Prev, Self::Remove, Self::Seek, Self::Now, Self::Reverse, Self::Quota, Self::Move, Self::When, Self::TrackIndex, Self::TrackRange, Self::TrackSet];
                topics.iter().for_each(|topic| write!(help, "`{}`, ", topic).unwrap());
                write!(help, "(or leave empty for an overview of available commands)").unwrap();
                help
            }
            HelpTopic::Join => {
                let mut help = String::new();
                writeln!(help, "{}", self.overview()).unwrap();
                writeln!(help, "Most functions of the bot will only be available when the bot is in a voice chat. \
                    To make it join a voice channel you have to join the channel first and then issue this command. \
                    The bot will follow you and display a welcome message in the default text channel to indicate it is ready for playback.").unwrap();
                writeln!(help, "If the bot is already in a voice channel it will move over.").unwrap();
                writeln!(help, "Use the `{}` command to make it leave again.", CMD_LEAVE).unwrap();
                help
            }
            HelpTopic::Leave => {
                let mut help = String::new();
                writeln!(help, "{}", self.overview()).unwrap();
                writeln!(help, "The playback will stop, but the bot will remember the current playlist and playback position.").unwrap();
                writeln!(help, "Use the `{}` command to make it join again.", CMD_JOIN).unwrap();
                help
            }
            HelpTopic::Enqueue => {
                let mut help = String::new();
                writeln!(help, "{}", self.overview()).unwrap();
                writeln!(help, "There's a handy shortcut: you can use `+` instead of `{}`.", CMD_ENQUEUE).unwrap();
                writeln!(help, "The track-reference is usually an URL pointing to a single track from a provider (such as audiotool, youtube, soundcloud, ...).").unwrap();
                writeln!(help, "You can specify an optional comment behind the track which will be displayed in the queue and during playback.").unwrap();
                writeln!(help, "There are a few specials available:").unwrap();
                writeln!(help, "· `at:single-charts` - enqueues the current single charts from the audiotool website in reverse order").unwrap();
                writeln!(help, "· `https://www.audiotool.com/genre/<some-genre>/charts/<year>-<week>` - enqueues some genre charts from a given week in reverse order.").unwrap();
                writeln!(help, "· `https://www.audiotool.com/album/<some-album>/` - enqueues an entire album").unwrap();
                writeln!(help, "You can also enqueue multiple tracks or lists at once. Just specify one reference per line.").unwrap();
                writeln!(help, "Pro-tip: surround your URLs with backticks (\"`\") to prevent Discord from flooding the channel with auto-previews. Triple-backticks (\\`\\`\\``) are also supported which comes in handy for multi-line enqueues.").unwrap();
                help
            }
            HelpTopic::Pause => {
                let mut help = String::new();
                writeln!(help, "{}", self.overview()).unwrap();
                help
            }
            HelpTopic::Resume => {
                let mut help = String::new();
                writeln!(help, "{}", self.overview()).unwrap();
                help
            }
            HelpTopic::Stop => {
                let mut help = String::new();
                writeln!(help, "{}", self.overview()).unwrap();
                writeln!(help, "If the playback ended automatically, this command can be used to disable auto-play. Otherwise playback would start again as soon as another track is enqueued.").unwrap();
                help
            }
            HelpTopic::Play => {
                let mut help = String::new();
                writeln!(help, "{}", self.overview()).unwrap();
                writeln!(help, "If the queue runs out of tracks, playback will stop but as soon as another track becomes available the auto-play feature will resume the playback.").unwrap();
                help
            }
            HelpTopic::Print => {
                let mut help = String::new();
                writeln!(help, "{}", self.overview()).unwrap();
                writeln!(help, "By default this command displays 2 tracks before the current and 15 tracks after it (`-2..+15`).\
                    If the output becomes too long, the bot will split it into multiple messages.").unwrap();
                writeln!(help, "Some common use cases:").unwrap();
                writeln!(help, "· `{} all` or `{} ..` - prints the entire queue", CMD_PRINT, CMD_PRINT).unwrap();
                writeln!(help, "· `{} future` - prints all tracks that are yet to come", CMD_PRINT).unwrap();
                writeln!(help, "· `{} history` - prints all tracks that have been played so far", CMD_PRINT).unwrap();
                writeln!(help, "· `{} -..` - same as future but includes the current track", CMD_PRINT).unwrap();
                writeln!(help, "· `{} +1` - prints the next track", CMD_PRINT).unwrap();
                writeln!(help, "· `{} -1` - prints the previous track", CMD_PRINT).unwrap();
                writeln!(help, "see `{} {}` for more options", CMD_HELP, HELP_TRACK_SET).unwrap();
                help
            }
            HelpTopic::Goto => {
                let mut help = String::new();
                writeln!(help, "{}", self.overview()).unwrap();
                writeln!(help, "Some common use cases:").unwrap();
                writeln!(help, "· `{} +5` - skips 5 tracks", CMD_GOTO).unwrap();
                writeln!(help, "· `{} 1` - continues playback at the start of the queue", CMD_GOTO).unwrap();
                writeln!(help, "see `{} {}` for more options", CMD_HELP, HELP_TRACK_INDEX).unwrap();
                help
            }
            HelpTopic::Next => {
                let mut help = String::new();
                writeln!(help, "{}", self.overview()).unwrap();
                help
            }
            HelpTopic::Prev => {
                let mut help = String::new();
                writeln!(help, "{}", self.overview()).unwrap();
                help
            }
            HelpTopic::Remove => {
                let mut help = String::new();
                writeln!(help, "{}", self.overview()).unwrap();
                writeln!(help, "If the current track is being removed, the playback will continue on the track which is now at the same location as the deleted one.").unwrap();
                writeln!(help, "Some common use cases:").unwrap();
                writeln!(help, "`· {} -` - remove the current track and continue with the next", CMD_REMOVE).unwrap();
                writeln!(help, "`· {} all` or `{} ..` - to clear the entire queue", CMD_REMOVE, CMD_REMOVE).unwrap();
                writeln!(help, "`· {} future` - removes everything after the current track", CMD_REMOVE).unwrap();
                writeln!(help, "`· {} history` - removes everything before the current track", CMD_REMOVE).unwrap();
                writeln!(help, "`· {} +1` - removes the next track", CMD_REMOVE).unwrap();
                writeln!(help, "`· {} 10..14` - removes multiple consecutive tracks", CMD_REMOVE).unwrap();
                writeln!(help, "`· {} 4,6,12` - removes multiple tracks", CMD_REMOVE).unwrap();
                writeln!(help, "see `{} {}` for more options", CMD_HELP, HELP_TRACK_SET).unwrap();
                help
            }
            HelpTopic::Seek => {
                let mut help = String::new();
                writeln!(help, "{}", self.overview()).unwrap();
                writeln!(help, "You can specify the position either in seconds (e.g. `120`) or in minutes and seconds (e.g. `1:23`). \
                        If you seek past the end of the track, the playback will continue with the next track.").unwrap();
                writeln!(help, "Some tracks do not allow seeking - especially when their duration is unknown.").unwrap();
                help
            }
            HelpTopic::Now => {
                let mut help = String::new();
                writeln!(help, "{}", self.overview()).unwrap();
                help
            }
            HelpTopic::Reverse => {
                let mut help = String::new();
                writeln!(help, "{}", self.overview()).unwrap();
                writeln!(help, "If the current track is being moved, the playback will continue on the track which is now at the same location as the moved one.").unwrap();
                writeln!(help, "Some common use cases:").unwrap();
                writeln!(help, "`· {} 3,7` - swap track #3 with #7", CMD_REVERSE).unwrap();
                writeln!(help, "`· {} 1..10` - reverse order of tracks #1 through #10 (useful for un-reversing charts)", CMD_REVERSE).unwrap();
                writeln!(help, "`· {} now,9` - swaps the current track with track #9", CMD_REVERSE).unwrap();
                writeln!(help, "see `{} {}` for more options", CMD_HELP, HELP_TRACK_SET).unwrap();
                help
            }
            HelpTopic::Quota => {
                let mut help = String::new();
                writeln!(help, "{}", self.overview()).unwrap();
                writeln!(help, "The quota affects the order in which enqueued track will be played back. \
                        A quota of `2` for example will allow each user to have 2 tracks enqueued after the current one. \
                        If they enqueue more tracks they will be tagged as _deferred_. \
                        This means that other users who haven't reached their quota yet, will have their track enqueued before the _deferred_ ones.").unwrap();
                writeln!(help, "This is a fairness rule ensures that every user's tracks will be heard within a reasonable time even when joining the VC at a later point or when individuals enqueue way more tracks than others.").unwrap();
                writeln!(help, "Deferred tracks are tagged with a `?` in a `{}` output which means their position in the queue is not final.", CMD_PRINT).unwrap();
                writeln!(help, "Modifying the queue or navigating manually may break the rule temporarily. Whenever something changes the bot will check if more _deferred_ tracks can be made final. There's no way to convert a final track into a _deferred_ one.").unwrap();
                writeln!(help, "Some common use cases:").unwrap();
                writeln!(help, "· `{} 1` - allows only onve final track per user; maximum fairness and useful for a lot of participants", CMD_QUOTA).unwrap();
                writeln!(help, "· `{} 2` - allows two tracks per user before marking them as _deferred_", CMD_QUOTA).unwrap();
                writeln!(help, "· `{} off` or `{} 0` - disables the quota rule which makes all enqueued tracks final.", CMD_QUOTA, CMD_QUOTA).unwrap();
                writeln!(help, "To query the current quota setting, issue this command without any parameter.").unwrap();
                help
            }
            HelpTopic::Move => {
                let mut help = String::new();
                writeln!(help, "{}", self.overview()).unwrap();
                writeln!(help, "This is basically the same as removing tracks and re-inserting them at the given location. \
                        If the current tracks changes due to the modification, playback will continue at the new track in place.").unwrap();
                writeln!(help, "Some common use cases:").unwrap();
                writeln!(help, "· `{} 1 to 7` - moves track #1 to the location of the current track #7; thus moving tracks #2 through #6 up by one.", CMD_MOVE).unwrap();
                writeln!(help, "· `{} 5 to next` - moves track #5 after the current one, so that it will be played next.", CMD_MOVE).unwrap();
                writeln!(help, "· `{} 10 to now` - moves track #10 to the current slot, so that it will be played immediatelly.", CMD_MOVE).unwrap();
                writeln!(help, "· `{} 10..15 to next` - moves tracks #10 though #15 after the current one.", CMD_MOVE).unwrap();
                writeln!(help, "· `{} 7,3,9 to next` - moves tracks #3, #7 and #9 (yes, in that order; see comment below) after the current one.", CMD_MOVE).unwrap();
                writeln!(help, "Caution: when selecting multiple tracks, their original order will be preserved; the order of the track-selector is irrelevant.").unwrap();
                writeln!(help, "see `{} {}` for more options on selecting tracks", CMD_HELP, HELP_TRACK_SET).unwrap();
                writeln!(help, "see `{} {}` for more options on placing tracks", CMD_HELP, HELP_TRACK_INDEX).unwrap();
                help
            }
            HelpTopic::When => {
                let mut help = String::new();
                writeln!(help, "{}", self.overview()).unwrap();
                writeln!(help, "If the queue contains tracks which don't have a known length, the result is a meant to be a minimal duration.").unwrap();
                writeln!(help, "If quota is enabled, _deferred_ tracks are volatile and might be moved in either direction.").unwrap();
                writeln!(help, "Some common use cases:").unwrap();
                writeln!(help, "· `{} 7` - displays how long to wait for track #7 to begin", CMD_WHEN).unwrap();
                writeln!(help, "· `{} +1` or `{} next` - displays how long to wait for the next track to start. the `{}` command does the same more verbosely.", CMD_WHEN, CMD_WHEN, CMD_NOW).unwrap();
                writeln!(help, "· `{} end` - displays how long it will take to play the remaining queue.", CMD_WHEN).unwrap();
                writeln!(help, "see `{} {}` for more options", CMD_HELP, HELP_TRACK_INDEX).unwrap();
                help
            }
            HelpTopic::TrackIndex => {
                let mut help = String::new();
                writeln!(help, "{}", self.overview()).unwrap();
                writeln!(help, "Track indices are either absolute or relative to the current playback position. A relative position always contains a sign (`-` or `+`) as prefix.").unwrap();
                writeln!(help, "Some common use cases:").unwrap();
                writeln!(help, "· `5` - track #5").unwrap();
                writeln!(help, "· `1` or `start` - track #1").unwrap();
                writeln!(help, "· `end` - the slot after the last enqueued track; useful for `{} end` or `{} end`", CMD_WHEN, CMD_GOTO).unwrap();
                writeln!(help, "· `+1` or `next` - next track after the current").unwrap();
                writeln!(help, "· `-1` or `prev` - previous track before the current").unwrap();
                writeln!(help, "· `+0`, `-0`, `+`, `-`, `0`, `now` - the current track").unwrap();
                writeln!(help, "Hint: Some commands allow to address the slot after the last track in the queue. This makes `4` a valid index even though the queue contains only 3 entries.").unwrap();
                help
            }
            HelpTopic::TrackRange => {
                let mut help = String::new();
                writeln!(help, "{}", self.overview()).unwrap();
                writeln!(help, "Track ranges have a start and an end separated by two dots `..`.  \
                        If not omitted, they can be any valid track index. \
                        If the start index is omitted, the range will start at the beginning of the queue. \
                        If the end index is omitted the range will end at the end of the queue.").unwrap();
                writeln!(help, "See `{} {}` for an explanation of track indices", CMD_HELP, HELP_TRACK_INDEX).unwrap();
                writeln!(help, "Some common use cases:").unwrap();
                writeln!(help, "· `4..9` - track #4 through #9").unwrap();
                writeln!(help, "· `+1..+4` - next 4 tracks after the current").unwrap();
                writeln!(help, "· `all` or `..` or `1..` - all tracks").unwrap();
                writeln!(help, "· `now`, `-..-`, `-` - only the current track").unwrap();
                writeln!(help, "· `future`, `+1..`, `next..` - everything after the current track").unwrap();
                writeln!(help, "· `history`, `..-1`, `..prev` - everything before the current track").unwrap();
                writeln!(help, "Hint: if (after resolving relative and named indices) the range is wrongly ordered (end comes before start), the boundaries will be switched.").unwrap();
                help
            }
            HelpTopic::TrackSet => {
                let mut help = String::new();
                writeln!(help, "{}", self.overview()).unwrap();
                writeln!(help, "They consist of an arbitrary number of indices or ranges whiy are comma-separated.").unwrap();
                writeln!(help, "See `{} {}` for an explanation of track ranges", CMD_HELP, HELP_TRACK_RANGE).unwrap();
                writeln!(help, "See `{} {}` for an explanation of track indices", CMD_HELP, HELP_TRACK_INDEX).unwrap();
                writeln!(help, "Some common use cases:").unwrap();
                writeln!(help, "· `1` - only track #1").unwrap();
                writeln!(help, "· `-` - only the current track").unwrap();
                writeln!(help, "· `4,6,9` - tracks #4, #6 and #9").unwrap();
                writeln!(help, "· `all` or `..` or `1..` or `start..end` - all tracks").unwrap();
                writeln!(help, "· `now` or `-..-` - only the current track").unwrap();
                writeln!(help, "· `future` or `next..` - everything after the current track").unwrap();
                writeln!(help, "· `history` or `..prev` - everything before the current track").unwrap();
                writeln!(help, "· `other` or `..prev,next..` - everything except for the current track; useful for the `{}` command to clear the queue without touching the current track", CMD_REMOVE).unwrap();
                writeln!(help, "Hint: duplicate indices and overlapping ranges will only be used once. For most command the order of the given tracks doesn't make a difference as all tracks will retain their original order.").unwrap();
                help
            }
        }
    }

    fn overview(self) -> String {
        match self {
            HelpTopic::Help => format!("`{}{} [<command>]` - shows a help page for the given command or topic", PREFIX, CMD_HELP),
            HelpTopic::Join => format!("`{}{}` - makes the bot follow you into a voice channel", PREFIX, CMD_JOIN),
            HelpTopic::Leave => format!("`{}{}` - makes the bot leave the voice channel", PREFIX, CMD_LEAVE),
            HelpTopic::Enqueue => format!("`{}{} <track-reference>` - adds tracks or entire playlists to the playback queue", PREFIX, CMD_ENQUEUE),
            HelpTopic::Pause => format!("`{}{}` - pauses the playback of the current track; use `{}` to resume the playback", PREFIX, CMD_PAUSE, CMD_RESUME),
            HelpTopic::Resume => format!("`{}{}` - resumes the playback of a paused track", PREFIX, CMD_RESUME),
            HelpTopic::Stop => format!("`{}{}` - stops the playback; use `{}` to restart the stopped track", PREFIX, CMD_STOP, CMD_PLAY),
            HelpTopic::Play => format!("`{}{}` - starts playback of the queue at the current position or restarts the current track", PREFIX, CMD_PLAY),
            HelpTopic::Print => format!("`{}{}` - displays the current playback queue", PREFIX, CMD_PRINT),
            HelpTopic::Goto => format!("`{}{} <track-index>` - continues playback at the given track number", PREFIX, CMD_GOTO),
            HelpTopic::Next => format!("`{}{}` - skips the current track", PREFIX, CMD_NEXT),
            HelpTopic::Prev => format!("`{}{}` - go back to the previous track", PREFIX, CMD_PREV),
            HelpTopic::Remove => format!("`{}{} <track-set>` - removes one or more tracks from the playback queue", PREFIX, CMD_REMOVE),
            HelpTopic::Seek => format!("`{}{} <position>` - seeks into the current track", PREFIX, CMD_SEEK),
            HelpTopic::Now => format!("`{}{}` - displays the current track with its playback position", PREFIX, CMD_NOW),
            HelpTopic::Reverse => format!("`{}{} <track-set>` - reverses or swaps two or more tracks", PREFIX, CMD_REVERSE),
            HelpTopic::Quota => format!("`{}{} [<quota>]` - limits the number of tracks a single user can enqueue", PREFIX, CMD_QUOTA),
            HelpTopic::Move => format!("`{}{} <track-set> to <track_index>` - moves one or multiple tracks to a new location", PREFIX, CMD_MOVE),
            HelpTopic::When => format!("`{}{} <track-index>` - tells how long to wait until the given track will be played", PREFIX, CMD_WHEN),
            HelpTopic::TrackIndex => "`<n>`|`+<n>`|`-<n>`|`start`|`now`|`end`|`next`|`prev` - a track-index allows to specify a single track within the queue".to_string(),
            HelpTopic::TrackRange => "`[<from>]..[<to>]`|`all`|`history`|`future`|`now`|`other` - a track-range can be used to specify one or more consecutive tracks".to_string(),
            HelpTopic::TrackSet => "`<range1>,<range2>,…`|`other` - a track-set is an arbitrary selection of tracks".to_string(),
            _ => String::new(),
        }
    }
}

impl FromStr for HelpTopic {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "" => Ok(Self::General),
            HELP_HELP => Ok(Self::Help),
            HELP_JOIN => Ok(Self::Join),
            HELP_LEAVE => Ok(Self::Leave),
            HELP_ENQUEUE => Ok(Self::Enqueue),
            HELP_PAUSE => Ok(Self::Pause),
            HELP_RESUME => Ok(Self::Resume),
            HELP_STOP => Ok(Self::Stop),
            HELP_PLAY => Ok(Self::Play),
            HELP_PRINT => Ok(Self::Print),
            HELP_GOTO => Ok(Self::Goto),
            HELP_NEXT => Ok(Self::Next),
            HELP_PREV => Ok(Self::Prev),
            HELP_REMOVE => Ok(Self::Remove),
            HELP_SEEK => Ok(Self::Seek),
            HELP_NOW => Ok(Self::Now),
            HELP_REVERSE => Ok(Self::Reverse),
            HELP_QUOTA => Ok(Self::Quota),
            HELP_MOVE => Ok(Self::Move),
            HELP_WHEN => Ok(Self::When),
            HELP_TRACK_INDEX => Ok(Self::TrackIndex),
            HELP_TRACK_RANGE => Ok(Self::TrackRange),
            HELP_TRACK_SET => Ok(Self::TrackSet),
            _ => Err(()),
        }
    }
}

impl fmt::Display for HelpTopic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            HelpTopic::General => "",
            HelpTopic::Help => HELP_HELP,
            HelpTopic::Join => HELP_JOIN,
            HelpTopic::Leave => HELP_LEAVE,
            HelpTopic::Enqueue => HELP_ENQUEUE,
            HelpTopic::Pause => HELP_PAUSE,
            HelpTopic::Resume => HELP_RESUME,
            HelpTopic::Stop => HELP_STOP,
            HelpTopic::Play => HELP_PLAY,
            HelpTopic::Print => HELP_PRINT,
            HelpTopic::Goto => HELP_GOTO,
            HelpTopic::Next => HELP_NEXT,
            HelpTopic::Prev => HELP_PREV,
            HelpTopic::Remove => HELP_REMOVE,
            HelpTopic::Seek => HELP_SEEK,
            HelpTopic::Now => HELP_NOW,
            HelpTopic::Reverse => HELP_REVERSE,
            HelpTopic::Quota => HELP_QUOTA,
            HelpTopic::Move => HELP_MOVE,
            HelpTopic::When => HELP_WHEN,
            HelpTopic::TrackIndex => HELP_TRACK_INDEX,
            HelpTopic::TrackRange => HELP_TRACK_RANGE,
            HelpTopic::TrackSet => HELP_TRACK_SET,
        }.fmt(f)
    }
}