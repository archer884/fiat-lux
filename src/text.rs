use tantivy::{schema::Value, TantivyDocument as Document};

use crate::{book::Book, search::SearchFields};

#[derive(Clone, Debug)]
pub struct Text {
    pub book: Book,
    pub chapter: u16,
    pub verse: u16,
    pub content: String,
}

impl Text {
    pub fn from_document(document: Document, fields: &SearchFields) -> Self {
        let location = document
            .get_first(fields.location)
            .and_then(|x| Some(x.as_facet()?.to_string()))
            .unwrap();

        let mut segments = location.trim_start_matches('/').split('/');

        let book = segments.next().unwrap().parse::<u8>().unwrap().into();
        let chapter = segments.next().unwrap().parse().unwrap();
        let verse = segments.next().unwrap().parse().unwrap();

        let content = document
            .get_first(fields.content)
            .unwrap()
            .as_str()
            .unwrap()
            .into();

        Self {
            book,
            chapter,
            verse,
            content,
        }
    }

    pub fn chapter(&self) -> Chapter {
        Chapter {
            book: self.book,
            chapter: self.chapter,
        }
    }
}

impl Eq for Text {}

impl PartialEq for Text {
    fn eq(&self, other: &Self) -> bool {
        self.book == other.book && self.chapter == other.chapter && self.verse == other.verse
    }
}

impl Ord for Text {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.book
            .cmp(&other.book)
            .then(self.chapter.cmp(&other.chapter))
            .then(self.verse.cmp(&other.verse))
    }
}

impl PartialOrd for Text {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Chapter {
    pub book: Book,
    pub chapter: u16,
}
