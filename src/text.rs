use crate::{book::Book, search::SearchFields};
use tantivy::{TantivyDocument as Document, schema::{Facet, Value}};

#[derive(Clone, Debug)]
pub struct Text {
    pub book: Book,
    pub chapter: u16,
    pub verse: u16,
    pub content: String,
}

impl Text {
    pub fn from_document(document: Document, fields: &SearchFields) -> Self {
        let encoded = document
            .get_first(fields.location)
            .and_then(|x| x.as_facet())
            .unwrap();

        // Parse the location back out of the facet via the public `to_path`
        // API rather than reaching into the encoded string and splitting on
        // NULs ourselves.
        let facet = Facet::from_encoded(encoded.as_bytes().to_vec()).unwrap();
        let path = facet.to_path();

        let book = path[0].parse::<u8>().unwrap().into();
        let chapter = path[1].parse().unwrap();
        let verse = path[2].parse().unwrap();

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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Chapter {
    pub book: Book,
    pub chapter: u16,
}
