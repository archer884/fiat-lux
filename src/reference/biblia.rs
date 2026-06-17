use super::{Reference, ReferenceLocator};
use crate::{Translation, book::Book};

pub struct Biblia;

impl Reference for Biblia {
    fn url(&self, location: &dyn ReferenceLocator, translation: Translation) -> String {
        let book_slug = book_slug(location.book());
        match location.verse() {
            None => format!(
                "https://biblia.com/bible/{}/{}/{}",
                translation,
                book_slug,
                location.chapter()
            ),
            Some(verse) => format!(
                "https://biblia.com/bible/{}/{}/{}/{}",
                translation,
                book_slug,
                location.chapter(),
                verse
            ),
        }
    }
}

fn book_slug(book: Book) -> &'static str {
    match book {
        Book::Genesis => "genesis",
        Book::Exodus => "exodus",
        Book::Leviticus => "leviticus",
        Book::Numbers => "numbers",
        Book::Deuteronomy => "deuteronomy",
        Book::Joshua => "joshua",
        Book::Judges => "judges",
        Book::Ruth => "ruth",
        Book::Samuel1 => "1samuel",
        Book::Samuel2 => "2samuel",
        Book::Kings1 => "1kings",
        Book::Kings2 => "2kings",
        Book::Chronicles1 => "1chronicles",
        Book::Chronicles2 => "2chronicles",
        Book::Ezra => "ezra",
        Book::Nehemiah => "nehemiah",
        Book::Esther => "esther",
        Book::Job => "job",
        Book::Psalms => "psalms",
        Book::Proverbs => "proverbs",
        Book::Ecclesiastes => "ecclesiastes",
        Book::SongofSongs => "songofsongs",
        Book::Isaiah => "isaiah",
        Book::Jeremiah => "jeremiah",
        Book::Lamentations => "lamentations",
        Book::Ezekiel => "ezekiel",
        Book::Daniel => "daniel",
        Book::Hosea => "hosea",
        Book::Joel => "joel",
        Book::Amos => "amos",
        Book::Obadiah => "obadiah",
        Book::Jonah => "jonah",
        Book::Micah => "micah",
        Book::Nahum => "nahum",
        Book::Habakkuk => "habakkuk",
        Book::Zephaniah => "zephaniah",
        Book::Haggai => "haggai",
        Book::Zechariah => "zechariah",
        Book::Malachi => "malachi",
        Book::Matthew => "matthew",
        Book::Mark => "mark",
        Book::Luke => "luke",
        Book::John => "john",
        Book::Acts => "acts",
        Book::Romans => "romans",
        Book::Corinthians1 => "1corinthians",
        Book::Corinthians2 => "2corinthians",
        Book::Galatians => "galatians",
        Book::Ephesians => "ephesians",
        Book::Philippians => "philippians",
        Book::Colossians => "colossians",
        Book::Thessalonians1 => "1thessalonians",
        Book::Thessalonians2 => "2thessalonians",
        Book::Timothy1 => "1timothy",
        Book::Timothy2 => "2timothy",
        Book::Titus => "titus",
        Book::Philemon => "philemon",
        Book::Hebrews => "hebrews",
        Book::James => "james",
        Book::Peter1 => "1peter",
        Book::Peter2 => "2peter",
        Book::John1 => "1john",
        Book::John2 => "2john",
        Book::John3 => "3john",
        Book::Jude => "jude",
        Book::Revelation => "revelation",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Translation;
    use crate::book::Book;
    use crate::location::Location;
    use crate::text::Chapter;

    #[test]
    fn url_for_single_verse() {
        let loc = Location {
            book: Book::John,
            chapter: 3,
            verse: 16,
        };
        assert_eq!(
            Biblia.url(&loc, Translation::Kjv),
            "https://biblia.com/bible/KJV/john/3/16"
        );
    }

    #[test]
    fn url_for_chapter() {
        let ch = Chapter {
            book: Book::John,
            chapter: 3,
        };
        assert_eq!(
            Biblia.url(&ch, Translation::Kjv),
            "https://biblia.com/bible/KJV/john/3"
        );
    }

    #[test]
    fn url_asv_translation() {
        let loc = Location {
            book: Book::Genesis,
            chapter: 1,
            verse: 1,
        };
        assert_eq!(
            Biblia.url(&loc, Translation::Asv),
            "https://biblia.com/bible/ASV/genesis/1/1"
        );
    }

    #[test]
    fn url_numbered_book_slug() {
        let loc = Location {
            book: Book::Kings1,
            chapter: 19,
            verse: 12,
        };
        assert_eq!(
            Biblia.url(&loc, Translation::Kjv),
            "https://biblia.com/bible/KJV/1kings/19/12"
        );
    }
}
