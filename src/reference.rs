mod biblia;

pub use biblia::Biblia;

use crate::book::Book;
use crate::location::Location;
use crate::text::{Chapter, Text};
use crate::Translation;
use clap::ValueEnum;
use std::fmt;

pub trait Reference {
    fn url(&self, location: &dyn ReferenceLocator, translation: Translation) -> String;
}

pub trait ReferenceLocator {
    fn book(&self) -> Book;
    fn chapter(&self) -> u16;
    fn verse(&self) -> Option<u16>;
}

impl ReferenceLocator for Location {
    fn book(&self) -> Book {
        self.book
    }

    fn chapter(&self) -> u16 {
        self.chapter
    }

    fn verse(&self) -> Option<u16> {
        Some(self.verse)
    }
}

impl ReferenceLocator for Chapter {
    fn book(&self) -> Book {
        self.book
    }

    fn chapter(&self) -> u16 {
        self.chapter
    }

    fn verse(&self) -> Option<u16> {
        None
    }
}

impl ReferenceLocator for &Text {
    fn book(&self) -> Book {
        self.book
    }

    fn chapter(&self) -> u16 {
        self.chapter
    }

    fn verse(&self) -> Option<u16> {
        Some(self.verse)
    }
}

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum ReferenceProvider {
    #[default]
    Biblia,
}

impl ReferenceProvider {
    fn short_name(&self) -> &'static str {
        match self {
            Self::Biblia => "biblia",
        }
    }
}

impl fmt::Display for ReferenceProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.short_name())
    }
}

impl ReferenceProvider {
    pub fn get(&self) -> Box<dyn Reference> {
        match self {
            ReferenceProvider::Biblia => Box::new(Biblia),
        }
    }
}
