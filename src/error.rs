use std::fmt;

use crate::{book::Book, location::Location};

pub trait AbbrevStr: AsRef<str> + Into<String> {
    fn get(self, limit: usize) -> String {
        let full = self.as_ref();

        if full.len() > limit {
            full[..limit].to_string() + "..."
        } else {
            self.into()
        }
    }
}

impl<T: AsRef<str> + Into<String>> AbbrevStr for T {}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    NotFound(NotFound),
}

#[derive(Debug, thiserror::Error)]
pub struct NotFound {
    pub entity: Entity,
    pub book: Book,
    pub location: Option<Location>,
}

impl fmt::Display for NotFound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entity = self.entity;
        let book = self.book;
        match self.location {
            Some(location) => write!(f, "{entity} not found: {book} {location}"),
            None => write!(f, "{entity} not found: {book}"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Entity {
    Book,
    Chapter,
    Verse,
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Entity::Book => f.write_str("book"),
            Entity::Chapter => f.write_str("chapter"),
            Entity::Verse => f.write_str("verse"),
        }
    }
}
