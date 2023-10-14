use std::{collections::HashSet, fmt, ops, str::FromStr};

#[derive(Clone, Copy)]
pub enum TrackIndex {
    Start(isize),
    Current(isize),
    End(isize),
}

#[derive(Clone, Copy)]
pub enum IndexResolve {
    TooSmall(isize),
    Ok(usize),
    End(usize),
    TooBig(usize),
}

impl FromStr for TrackIndex {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let result = match s.trim() {
            "" => return Err("input was empty".to_owned()),
            "start" => Self::Start(0),
            "end" => Self::End(0),
            "now" => Self::Current(0),
            "next" => Self::Current(1),
            "prev" => Self::Current(-1),
            s => {
                let result = if let Some(relative) = s.strip_prefix('+') {
                    let relative = relative.trim();
                    if relative.is_empty() {
                        Ok(Self::Current(0))
                    } else {
                        relative.parse::<isize>().map(Self::Current)
                    }
                } else if let Some(relative) = s.strip_prefix('-') {
                    let relative = relative.trim();
                    if relative.is_empty() {
                        Ok(Self::Current(0))
                    } else {
                        #[allow(clippy::cast_possible_wrap)]
                        relative
                            .parse::<usize>()
                            .map(|i| Self::Current(-(i as isize)))
                    }
                } else {
                    s.parse::<isize>().map(|i| {
                        if i == 0 {
                            Self::Current(0)
                        } else {
                            Self::Start(i - 1)
                        }
                    })
                };
                result.map_err(|err| err.to_string())?
            }
        };
        Ok(result)
    }
}

impl TrackIndex {
    pub fn resolve_raw(self, current: usize, track_count: usize) -> isize {
        #[allow(clippy::cast_possible_wrap)]
        match self {
            TrackIndex::Start(i) => i,
            TrackIndex::Current(i) => current as isize + i,
            TrackIndex::End(i) => track_count as isize + i,
        }
    }

    pub fn resolve(self, current: usize, track_count: usize) -> IndexResolve {
        let raw_index = self.resolve_raw(current, track_count);

        if raw_index < 0 {
            IndexResolve::TooSmall(raw_index)
        } else {
            #[allow(clippy::cast_sign_loss)]
            let raw_index = raw_index as usize;
            match (raw_index).cmp(&track_count) {
                std::cmp::Ordering::Less => IndexResolve::Ok(raw_index),
                std::cmp::Ordering::Equal => IndexResolve::End(raw_index),
                std::cmp::Ordering::Greater => IndexResolve::TooBig(raw_index),
            }
        }
    }
}

impl fmt::Debug for TrackIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Start(i) => write!(f, "{}", i + 1),
            Self::Current(i) => match i.signum() {
                -1 => write!(f, "-{}", -i),
                1 => write!(f, "+{i}"),
                _ => write!(f, "-"),
            },
            Self::End(i) => match i.signum() {
                -1 => write!(f, "end-{}", -i),
                1 => write!(f, "end+{i}"),
                _ => write!(f, "end"),
            },
        }
    }
}
/*
fn main() {
    println!("{:?}", TrackIndex::parse_str(""));
    println!("{:?}", TrackIndex::parse_str(" - "));
    println!("{:?}", TrackIndex::parse_str(" + "));
    println!("{:?}", TrackIndex::parse_str(" "));
    println!("{:?}", TrackIndex::parse_str(" 1 "));
    println!("{:?}", TrackIndex::parse_str(" +2"));
    println!("{:?}", TrackIndex::parse_str("-4 "));
    println!("{:?}", TrackIndex::parse_str(" + 2"));
    println!("{:?}", TrackIndex::parse_str("- 4 "));
    println!("{:?}", TrackIndex::parse_str("0"));
    println!("{:?}", TrackIndex::parse_str("+0"));
    println!("{:?}", TrackIndex::parse_str("-0"));
    println!("{:?}", TrackIndex::parse_str("s"));
}
*/

pub enum TrackIndexRange {
    Single(TrackIndex),
    Range(TrackIndex, TrackIndex),
}

impl TrackIndexRange {
    pub fn resolve(&self, current: usize, track_count: usize) -> ops::Range<usize> {
        #[allow(clippy::range_plus_one)]
        match *self {
            TrackIndexRange::Single(index) => match index.resolve(current, track_count) {
                IndexResolve::Ok(index) => index..index + 1,
                IndexResolve::TooSmall(_) | IndexResolve::End(_) | IndexResolve::TooBig(_) => 0..0,
            },
            TrackIndexRange::Range(start_index, end_index) => {
                match (
                    start_index.resolve(current, track_count),
                    end_index.resolve(current, track_count),
                ) {
                    (IndexResolve::TooSmall(_), IndexResolve::Ok(end_index)) => 0..end_index + 1,
                    (IndexResolve::TooSmall(_), IndexResolve::End(_) | IndexResolve::TooBig(_)) => {
                        0..track_count
                    }
                    (IndexResolve::Ok(start_index), IndexResolve::Ok(end_index)) => {
                        start_index..end_index + 1
                    }
                    (
                        IndexResolve::Ok(start_index),
                        IndexResolve::End(_) | IndexResolve::TooBig(_),
                    ) => start_index..track_count,
                    (_, IndexResolve::TooSmall(_))
                    | (IndexResolve::End(_) | IndexResolve::TooBig(_), _) => 0..0,
                }
            }
        }
    }
}

impl FromStr for TrackIndexRange {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        let range = match s.trim() {
            "" => return Err("input was empty".to_owned()),
            "all" => Self::Range(TrackIndex::Start(0), TrackIndex::End(0)),
            "history" => Self::Range(TrackIndex::Start(0), TrackIndex::Current(-1)),
            "future" => Self::Range(TrackIndex::Current(1), TrackIndex::End(0)),
            "now" => Self::Single(TrackIndex::Current(0)),
            s => {
                let mut interval = s.splitn(2, "..");
                let start_str = interval.next().unwrap();
                let start_index = match start_str.trim() {
                    "" => TrackIndex::Start(0),
                    s => s
                        .parse::<TrackIndex>()
                        .map_err(|_err| format!("`{s}` is not a valid track index"))?,
                };

                if let Some(end_str) = interval.next() {
                    // interval
                    let end_index = match end_str.trim() {
                        "" => TrackIndex::End(0),
                        s => s
                            .parse::<TrackIndex>()
                            .map_err(|_err| format!("`{s}` is not a valid track index"))?,
                    };

                    Self::Range(start_index, end_index)
                } else {
                    // single number
                    Self::Single(start_index)
                }
            }
        };
        Ok(range)
    }
}

impl fmt::Debug for TrackIndexRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Single(i) => write!(f, "{i:?}"),
            Self::Range(start, end) => write!(f, "{start:?}..{end:?}"),
        }
    }
}

#[derive(Debug)]
pub struct TrackIndexSelection(pub Vec<TrackIndexRange>);

impl TrackIndexSelection {
    pub fn parse_str(s: &str) -> Result<Self, String> {
        let mut list = Vec::new();
        match s {
            "other" => {
                list.push(TrackIndexRange::Range(
                    TrackIndex::Start(0),
                    TrackIndex::Current(-1),
                ));
                list.push(TrackIndexRange::Range(
                    TrackIndex::Current(1),
                    TrackIndex::End(0),
                ));
            }
            _ => {
                for part in s.split(',').map(str::trim).filter(|part| !part.is_empty()) {
                    list.push(part.parse::<TrackIndexRange>()?);
                }
            }
        }

        Ok(Self(list))
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn collect(&self, current: usize, track_count: usize) -> HashSet<usize> {
        // TODO refactor this with a `flap_map`
        let mut result = HashSet::new();
        for range in &self.0 {
            for index in range.resolve(current, track_count) {
                result.insert(index);
            }
        }
        result
    }
}
