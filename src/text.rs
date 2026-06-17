use crate::{book::Book, search::SearchFields};
use tantivy::{
    TantivyDocument as Document,
    schema::{Facet, Value},
};

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

// A note regarding Eq and Ord:
//
// We implement these by hand because the actual text content of a verse isn't as important, for
// the purposes of comparison, as the chapter and verse. We want a given chapter and verse from
// translation A to be considered as equal to a given chapter and verse from translation B even
// though the CONTENT of the two verses won't be equal.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::book::Book;

    fn text(book: Book, chapter: u16, verse: u16, content: &str) -> Text {
        Text {
            book,
            chapter,
            verse,
            content: content.into(),
        }
    }

    #[test]
    fn equal_when_content_differs() {
        // Two texts at the same location but with different content (e.g. from
        // different translations) should compare equal.
        let a = text(Book::John, 3, 16, "For God so loved...");
        let b = text(Book::John, 3, 16, "For God so loved the world...");
        assert_eq!(a, b);
    }

    #[test]
    fn not_equal_when_location_differs() {
        let a = text(Book::John, 3, 16, "same content");
        let b = text(Book::John, 3, 17, "same content");
        assert_ne!(a, b);
    }

    #[test]
    fn ordering_by_book_then_chapter_then_verse() {
        let mut verses = [
            text(Book::John, 3, 17, ""),
            text(Book::Genesis, 1, 1, ""),
            text(Book::John, 3, 16, ""),
            text(Book::John, 1, 1, ""),
        ];
        verses.sort();
        assert_eq!(verses[0], text(Book::Genesis, 1, 1, ""));
        assert_eq!(verses[1], text(Book::John, 1, 1, ""));
        assert_eq!(verses[2], text(Book::John, 3, 16, ""));
        assert_eq!(verses[3], text(Book::John, 3, 17, ""));
    }
}
