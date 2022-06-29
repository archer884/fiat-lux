use std::{fmt, num::ParseIntError, str::FromStr};

use crate::{book::Book, error::AbbrevStr};

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
    pub verse: Option<u16>,
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
        // I have just now decided that the way this has to be written is in the following format,
        // at least until I start to refine the cli...

        // psalms.23
        // Romans.3:23
        // john.3:16 -- see also Austin.3:16

        let (chapter, verse) = s.split_once(':').unwrap_or((s, ""));

        // For right now, we're not going to check the book's name, because... well, whatever. We
        // are gonna implement that later.

        let chapter = chapter
            .parse()
            .map_err(|e| ParseLocationError::chapter(chapter, e))?;

        if verse.is_empty() {
            Ok(PartialLocation {
                chapter,
                verse: None,
            })
        } else {
            let verse: u16 = verse
                .parse()
                .map_err(|e| ParseLocationError::verse(verse, e))?;
            Ok(PartialLocation {
                chapter,
                verse: Some(verse),
            })
        }
    }
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum ParseLocationError {
    #[error("unable to parse chapter: {text}")]
    Chapter { text: String, cause: ParseIntError },

    #[error("unable to parse verse: {text}")]
    Verse { text: String, cause: ParseIntError },
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
