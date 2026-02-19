use super::Reference;
use crate::{book::Book, location::Location, Translation};

pub struct Biblia;

impl Reference for Biblia {
    fn url(&self, location: &Location, translation: Translation) -> String {
        let book_slug = book_slug(location.book);
        format!(
            "https://biblia.com/bible/{}/{}/{}/{}",
            translation, book_slug, location.chapter, location.verse
        )
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
