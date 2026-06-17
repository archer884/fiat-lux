use std::{
    fmt,
    num::{NonZero, ParseIntError},
    str::FromStr,
};

use crate::{book::Book, error::AbbrevStr};

pub const AUSTIN_VERSE: Verse = Verse {
    start: NonZero::new(16).unwrap(),
    end: None,
};

/// Book, chapter and verse
///
/// A location such as this can be used to search translations for a specific verse.
#[derive(Clone, Copy, Debug)]
pub struct Location {
    pub book: Book,
    pub chapter: u16,
    pub verse: u16,
}

impl Location {
    pub fn from_id<T: Into<u64>>(id: T) -> Self {
        let id = id.into();
        Self {
            book: ((id / 1_000_000) as u8).into(),
            chapter: (id % 1_000_000 / 1000) as u16,
            verse: (id % 1000) as u16,
        }
    }
}

/// Chapter and verse
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PartialLocation {
    pub chapter: u16,
    pub verse: Option<Verse>,
}

impl fmt::Display for PartialLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let chapter = self.chapter;
        match self.verse {
            Some(verse) => write!(f, "[{chapter}:{verse}]"),
            None => write!(f, "[{chapter}]"),
        }
    }
}

impl FromStr for PartialLocation {
    type Err = ParseLocationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (chapter, verse) = s.split_once(':').unwrap_or((s, ""));
        let chapter = chapter
            .parse()
            .map_err(|e| ParseLocationError::chapter(chapter, e))?;

        if verse.is_empty() {
            return Ok(PartialLocation {
                chapter,
                verse: None,
            });
        }

        // The original version of this function was only concerned with parsing a chapter and
        // verse, e.g. 3:16. It's not uncommon, however, for verses to be given as a range, e.g.
        // 127:4-5. To support that will require a little more effort on my part...
        Ok(PartialLocation {
            chapter,
            verse: Some(verse.parse()?),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Verse {
    start: NonZero<u16>,
    end: Option<NonZero<u16>>,
}

impl Verse {
    pub fn contains(&self, verse: u16) -> bool {
        let Some(verse) = NonZero::new(verse) else {
            return false;
        };

        if let Some(end) = self.end {
            verse >= self.start && verse <= end
        } else {
            verse == self.start
        }
    }
}

impl fmt::Display for Verse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.end {
            Some(end) => write!(f, "{}-{}", self.start, end),
            None => self.start.fmt(f),
        }
    }
}

impl FromStr for Verse {
    type Err = ParseLocationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once('-') {
            Some((start, end)) => {
                let start = start.parse().map_err(|e| ParseLocationError::verse(s, e))?;
                let end = end.parse().map_err(|e| ParseLocationError::verse(s, e))?;

                // Bible verses are always cited in ascending order; a reversed
                // range is invariably a typo, so reject it rather than silently
                // matching nothing.
                if end < start {
                    return Err(ParseLocationError::Range { start, end });
                }

                Ok(Verse {
                    start,
                    end: Some(end),
                })
            }
            None => Ok(Verse {
                start: s.parse().map_err(|e| ParseLocationError::verse(s, e))?,
                end: None,
            }),
        }
    }
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum ParseLocationError {
    #[error("unable to parse chapter: {text}")]
    Chapter { text: String, cause: ParseIntError },

    #[error("unable to parse verse: {text}")]
    Verse { text: String, cause: ParseIntError },

    #[error("verse range end {end} precedes start {start}")]
    Range {
        start: NonZero<u16>,
        end: NonZero<u16>,
    },
}

impl ParseLocationError {
    fn chapter(text: impl AbbrevStr, cause: ParseIntError) -> Self {
        ParseLocationError::Chapter {
            text: text.get(10),
            cause,
        }
    }

    fn verse(text: impl AbbrevStr, cause: ParseIntError) -> Self {
        ParseLocationError::Verse {
            text: text.get(10),
            cause,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verse_single() {
        let v: Verse = "16".parse().unwrap();
        assert_eq!(v.start, NonZero::new(16).unwrap());
        assert_eq!(v.end, None);
    }

    #[test]
    fn verse_ascending_range() {
        let v: Verse = "16-18".parse().unwrap();
        assert_eq!(v.start, NonZero::new(16).unwrap());
        assert_eq!(v.end, Some(NonZero::new(18).unwrap()));
    }

    #[test]
    fn verse_degenerate_range_is_allowed() {
        // Redundant but not wrong: a range whose endpoints coincide.
        let v: Verse = "16-16".parse().unwrap();
        assert_eq!(v.start, v.end.unwrap());
    }

    #[test]
    fn verse_reversed_range_is_rejected() {
        let err = "18-16".parse::<Verse>().unwrap_err();
        assert!(matches!(
            err,
            ParseLocationError::Range { start, end } if start.get() == 18 && end.get() == 16
        ));
    }
}
