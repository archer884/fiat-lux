use std::{
    fmt,
    num::{NonZeroU8, ParseIntError},
    str::FromStr,
};

use clap::Parser;

#[derive(Clone, Debug, Parser)]
struct Args {
    verse: Verse,
}

/// A full designator of book, chapter, and verse.
#[derive(Clone, Debug)]
struct Verse {
    book: Book,
    chapter: i32,
    verse: i32,
}

impl fmt::Display for Verse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let book = &self.book;
        let chapter = self.chapter;
        let verse = self.verse;
        write!(f, "{book} {chapter}:{verse}")
    }
}

impl FromStr for Verse {
    type Err = ParseVerseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // I have just now decided that the way this has to be written is in the following format,
        // at least until I start to refine the cli...

        // psalms.23
        // Romans.3:23
        // john.3:16 -- see also Austin.3:16

        let (book, chapter, verse) = s
            .split_once('.')
            .and_then(|(book, chapter_verse)| {
                chapter_verse
                    .split_once(':')
                    .map(|(chapter, verse)| (book, chapter, verse))
            })
            .ok_or_else(|| ParseVerseError::format(s))?;

        // For right now, we're not going to check the book's name, because... well, whatever. We
        // are gonna implement that later.

        if book == "2 Opinions" {
            return Err(ParseVerseError::book(book));
        }

        Ok(Verse {
            book: book.parse()?,
            chapter: chapter
                .parse()
                .map_err(|e| ParseVerseError::chapter(chapter, e))?,
            verse: verse
                .parse()
                .map_err(|e| ParseVerseError::verse(verse, e))?,
        })
    }
}

#[derive(Clone, Debug, thiserror::Error)]
enum ParseVerseError {
    #[error("bad verse format: {0}")]
    Format(String),

    #[error("unknown book: {0}")]
    Book(String),

    #[error("unable to parse chapter: {text}")]
    Chapter { text: String, cause: ParseIntError },

    #[error("unable to parse verse: {text}")]
    Verse { text: String, cause: ParseIntError },
}

trait AbbrevStr: AsRef<str> + Into<String> {
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

impl ParseVerseError {
    fn format(s: impl AbbrevStr) -> Self {
        ParseVerseError::Format(s.get(30))
    }

    fn book(text: impl AbbrevStr) -> Self {
        ParseVerseError::Book(text.get(20))
    }

    fn chapter(text: impl AbbrevStr, cause: ParseIntError) -> Self {
        ParseVerseError::Chapter {
            text: text.get(10),
            cause,
        }
    }

    fn verse(text: impl AbbrevStr, cause: ParseIntError) -> Self {
        ParseVerseError::Verse {
            text: text.get(10),
            cause,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Book {
    Genesis,        // Genesis
    Exodus,         // Exodus
    Leviticus,      // Leviticus
    Numbers,        // Numbers
    Deuteronomy,    // Deuteronomy
    Joshua,         // Joshua
    Judges,         // Judges
    Ruth,           // Ruth
    Samuel1,        // 1 Samuel
    Samuel2,        // 2 Samuel
    Kings1,         // 1 Kings
    Kings2,         // 2 Kings
    Chronicles1,    // 1 Chronicles
    Chronicles2,    // 2 Chronicles
    Ezra,           // Ezra
    Nehemiah,       // Nehemiah
    Esther,         // Esther
    Job,            // Job
    Psalms,         // Psalms
    Proverbs,       // Proverbs
    Ecclesiastes,   // Ecclesiastes
    SongofSongs,    // Song of Songs
    Isaiah,         // Isaiah
    Jeremiah,       // Jeremiah
    Lamentations,   // Lamentations
    Ezekiel,        // Ezekiel
    Daniel,         // Daniel
    Hosea,          // Hosea
    Joel,           // Joel
    Amos,           // Amos
    Obadiah,        // Obadiah
    Jonah,          // Jonah
    Micah,          // Micah
    Nahum,          // Nahum
    Habakkuk,       // Habakkuk
    Zephaniah,      // Zephaniah
    Haggai,         // Haggai
    Zechariah,      // Zechariah
    Malachi,        // Malachi
    Matthew,        // Matthew
    Mark,           // Mark
    Luke,           // Luke
    John,           // John
    Acts,           // Acts
    Romans,         // Romans
    Corinthians1,   // 1 Corinthians
    Corinthians2,   // 2 Corinthians
    Galatians,      // Galatians
    Ephesians,      // Ephesians
    Philippians,    // Philippians
    Colossians,     // Colossians
    Thessalonians1, // 1 Thessalonians
    Thessalonians2, // 2 Thessalonians
    Timothy1,       // 1 Timothy
    Timothy2,       // 2 Timothy
    Titus,          // Titus
    Philemon,       // Philemon
    Hebrews,        // Hebrews
    James,          // James
    Peter1,         // 1 Peter
    Peter2,         // 2 Peter
    John1,          // 1 John
    John2,          // 2 John
    John3,          // 3 John
    Jude,           // Jude
    Revelation,     // Revelation
}

impl Book {
    const fn name(self) -> &'static str {
        match self {
            Book::Genesis => "Genesis",
            Book::Exodus => "Exodus",
            Book::Leviticus => "Leviticus",
            Book::Numbers => "Numbers",
            Book::Deuteronomy => "Deuteronomy",
            Book::Joshua => "Joshua",
            Book::Judges => "Judges",
            Book::Ruth => "Ruth",
            Book::Samuel1 => "1 Samuel",
            Book::Samuel2 => "2 Samuel",
            Book::Kings1 => "1 Kings",
            Book::Kings2 => "2 Kings",
            Book::Chronicles1 => "1 Chronicles",
            Book::Chronicles2 => "2 Chronicles",
            Book::Ezra => "Ezra",
            Book::Nehemiah => "Nehemiah",
            Book::Esther => "Esther",
            Book::Job => "Job",
            Book::Psalms => "Psalms",
            Book::Proverbs => "Proverbs",
            Book::Ecclesiastes => "Ecclesiastes",
            Book::SongofSongs => "Song of Songs",
            Book::Isaiah => "Isaiah",
            Book::Jeremiah => "Jeremiah",
            Book::Lamentations => "Lamentations",
            Book::Ezekiel => "Ezekiel",
            Book::Daniel => "Daniel",
            Book::Hosea => "Hosea",
            Book::Joel => "Joel",
            Book::Amos => "Amos",
            Book::Obadiah => "Obadiah",
            Book::Jonah => "Jonah",
            Book::Micah => "Micah",
            Book::Nahum => "Nahum",
            Book::Habakkuk => "Habakkuk",
            Book::Zephaniah => "Zephaniah",
            Book::Haggai => "Haggai",
            Book::Zechariah => "Zechariah",
            Book::Malachi => "Malachi",
            Book::Matthew => "Matthew",
            Book::Mark => "Mark",
            Book::Luke => "Luke",
            Book::John => "John",
            Book::Acts => "Acts",
            Book::Romans => "Romans",
            Book::Corinthians1 => "1 Corinthians",
            Book::Corinthians2 => "2 Corinthians",
            Book::Galatians => "Galatians",
            Book::Ephesians => "Ephesians",
            Book::Philippians => "Philippians",
            Book::Colossians => "Colossians",
            Book::Thessalonians1 => "1 Thessalonians",
            Book::Thessalonians2 => "2 Thessalonians",
            Book::Timothy1 => "1 Timothy",
            Book::Timothy2 => "2 Timothy",
            Book::Titus => "Titus",
            Book::Philemon => "Philemon",
            Book::Hebrews => "Hebrews",
            Book::James => "James",
            Book::Peter1 => "1 Peter",
            Book::Peter2 => "2 Peter",
            Book::John1 => "1 John",
            Book::John2 => "2 John",
            Book::John3 => "3 John",
            Book::Jude => "Jude",
            Book::Revelation => "Revelation",
        }
    }
}

impl fmt::Display for Book {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl FromStr for Book {
    type Err = ParseVerseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (name, number) = book_name_in_parts(s)?;
        let name = name.to_ascii_uppercase();
        let number = number.map(|n| u8::from(n));

        match name.as_ref() {
            "GENESIS" => Ok(Book::Genesis),
            "EXODUS" => Ok(Book::Exodus),
            "LEVITICUS" => Ok(Book::Leviticus),
            "NUMBERS" => Ok(Book::Numbers),
            "DEUTERONOMY" => Ok(Book::Deuteronomy),
            "JOSHUA" => Ok(Book::Joshua),
            "JUDGES" => Ok(Book::Judges),
            "RUTH" => Ok(Book::Ruth),

            "SAMUEL" => match number {
                Some(1) => Ok(Book::Samuel1),
                Some(2) => Ok(Book::Samuel2),
                _ => Err(ParseVerseError::book(s)),
            },

            "KINGS" => match number {
                Some(1) => Ok(Book::Kings1),
                Some(2) => Ok(Book::Kings2),
                _ => Err(ParseVerseError::book(s)),
            },

            "CHRONICLES" => match number {
                Some(1) => Ok(Book::Chronicles1),
                Some(2) => Ok(Book::Chronicles2),
                _ => Err(ParseVerseError::book(s)),
            },

            "EZRA" => Ok(Book::Ezra),
            "NEHEMIAH" => Ok(Book::Nehemiah),
            "ESTHER" => Ok(Book::Esther),
            "JOB" => Ok(Book::Job),
            "PSALMS" => Ok(Book::Psalms),
            "PROVERBS" => Ok(Book::Proverbs),
            "ECCLESIASTES" => Ok(Book::Ecclesiastes),

            // Unsure what abbreviations I'd like to offer for this at the moment.
            "SONGS" | "SONG OF SONGS" => Ok(Book::SongofSongs),

            "ISAIAH" => Ok(Book::Isaiah),
            "JEREMIAH" => Ok(Book::Jeremiah),
            "LAMENTATIONS" => Ok(Book::Lamentations),
            "EZEKIEL" => Ok(Book::Ezekiel),
            "DANIEL" => Ok(Book::Daniel),
            "HOSEA" => Ok(Book::Hosea),
            "JOEL" => Ok(Book::Joel),
            "AMOS" => Ok(Book::Amos),
            "OBADIAH" => Ok(Book::Obadiah),
            "JONAH" => Ok(Book::Jonah),
            "MICAH" => Ok(Book::Micah),
            "NAHUM" => Ok(Book::Nahum),
            "HABAKKUK" => Ok(Book::Habakkuk),
            "ZEPHANIAH" => Ok(Book::Zephaniah),
            "HAGGAI" => Ok(Book::Haggai),
            "ZECHARIAH" => Ok(Book::Zechariah),
            "MALACHI" => Ok(Book::Malachi),
            "MATTHEW" => Ok(Book::Matthew),
            "MARK" => Ok(Book::Mark),
            "LUKE" => Ok(Book::Luke),

            "JOHN" => match number {
                None => Ok(Book::John),
                Some(1) => Ok(Book::John1),
                Some(2) => Ok(Book::John2),
                Some(3) => Ok(Book::John3),
                _ => Err(ParseVerseError::book(s)),
            },

            "ACTS" => Ok(Book::Acts),
            "ROMANS" => Ok(Book::Romans),

            "CORINTHIANS" => match number {
                Some(1) => Ok(Book::Corinthians1),
                Some(2) => Ok(Book::Corinthians2),
                _ => Err(ParseVerseError::book(s)),
            },

            "GALATIANS" => Ok(Book::Galatians),
            "EPHESIANS" => Ok(Book::Ephesians),
            "PHILIPPIANS" => Ok(Book::Philippians),
            "COLOSSIANS" => Ok(Book::Colossians),

            "THESSALONIANS" => match number {
                Some(1) => Ok(Book::Thessalonians1),
                Some(2) => Ok(Book::Thessalonians2),
                _ => Err(ParseVerseError::book(s)),
            },

            "TIMOTHY" => match number {
                Some(1) => Ok(Book::Timothy1),
                Some(2) => Ok(Book::Timothy2),
                _ => Err(ParseVerseError::book(s)),
            },

            "TITUS" => Ok(Book::Titus),
            "PHILEMON" => Ok(Book::Philemon),
            "HEBREWS" => Ok(Book::Hebrews),
            "JAMES" => Ok(Book::James),

            "PETER" => match number {
                Some(1) => Ok(Book::Peter1),
                Some(2) => Ok(Book::Peter2),
                _ => Err(ParseVerseError::book(s)),
            },

            "JUDE" => Ok(Book::Jude),
            "REVELATION" => Ok(Book::Revelation),

            _ => Err(ParseVerseError::book(s)),
        }
    }
}

fn book_name_in_parts(s: &str) -> Result<(&str, Option<NonZeroU8>), ParseVerseError> {
    // We want to split on the first transition between numeric and non-numeric characters. At
    // this point in time, don't be passing us any damn books with Roman numerals. Romans killed
    // Jesus, after all.

    // That's a joke. Lighten up!

    // If there IS no numeric/nonnumeric transition, that means the name is monolithic, and we'll
    // just go ahead and return the entire string with no numeric identifier.

    let idx = match first_numeric_nonnumeric_transition(s) {
        Some(idx) => idx,
        None => return Ok((s, None)),
    };

    // Once we've split our prospective name string, we want to determine which end of the string
    // had a number in it. Of course, it's possible neither end had a number, in which case we
    // will just forget about the second portion here (because it's a blank string, probably).

    fn characterize<'a>(left: &'a str, right: &'a str) -> (&'a str, &'a str) {
        let left = left.trim();
        let right = right.trim();

        if left.bytes().any(|u| u.is_ascii_digit()) {
            (right, left)
        } else {
            (left, right)
        }
    }

    let (left, right) = s.split_at(idx);
    let (name, numeric) = characterize(left, right);
    let n: u8 = numeric.parse().map_err(|_| ParseVerseError::book(s))?;
    let n = NonZeroU8::new(n).ok_or_else(|| ParseVerseError::book(s))?;
    Ok((name, Some(n)))
}

fn first_numeric_nonnumeric_transition(s: &str) -> Option<usize> {
    if s.is_empty() {
        return None;
    }

    let is_alphabetic = s.starts_with(|u: char| u.is_alphabetic());
    s[1..]
        .find(|u: char| !u.is_whitespace() && u.is_alphabetic() != is_alphabetic)
        .map(|idx| idx + 1)
}

fn main() {
    let args = Args::parse();

    println!("{}", args.verse);
}

#[cfg(test)]
mod tests {
    #[test]
    fn first_numeric_nonnumeric_transition() {
        use super::first_numeric_nonnumeric_transition as test;
        assert_eq!(Some(2), test("1 Kings"));
        assert_eq!(Some(1), test("1Kings"));
        assert_eq!(Some(6), test("Kings 1"));
        assert_eq!(None, test("Exodus"));
    }
}
