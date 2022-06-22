use std::{
    fmt, io,
    num::{NonZeroU8, ParseIntError},
    str::FromStr,
};

use clap::Parser;
use indexmap::IndexMap;
use serde::Deserialize;

#[derive(Clone, Debug, Parser)]
struct Args {
    location: Location,
}

/// A full designator of book, chapter, and verse.
#[derive(Clone, Copy, Debug)]
struct Location {
    book: Book,
    chapter: u8,
    verse: u16,
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let book = &self.book;
        let chapter = self.chapter;
        let verse = self.verse;
        write!(f, "{book} [{chapter}:{verse}]")
    }
}

impl FromStr for Location {
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

        Ok(Location {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
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
    const fn from_u8(u: u8) -> Self {
        match u - 1 {
            0 => Book::Genesis,
            1 => Book::Exodus,
            2 => Book::Leviticus,
            3 => Book::Numbers,
            4 => Book::Deuteronomy,
            5 => Book::Joshua,
            6 => Book::Judges,
            7 => Book::Ruth,
            8 => Book::Samuel1,
            9 => Book::Samuel2,
            10 => Book::Kings1,
            11 => Book::Kings2,
            12 => Book::Chronicles1,
            13 => Book::Chronicles2,
            14 => Book::Ezra,
            15 => Book::Nehemiah,
            16 => Book::Esther,
            17 => Book::Job,
            18 => Book::Psalms,
            19 => Book::Proverbs,
            20 => Book::Ecclesiastes,
            21 => Book::SongofSongs,
            22 => Book::Isaiah,
            23 => Book::Jeremiah,
            24 => Book::Lamentations,
            25 => Book::Ezekiel,
            26 => Book::Daniel,
            27 => Book::Hosea,
            28 => Book::Joel,
            29 => Book::Amos,
            30 => Book::Obadiah,
            31 => Book::Jonah,
            32 => Book::Micah,
            33 => Book::Nahum,
            34 => Book::Habakkuk,
            35 => Book::Zephaniah,
            36 => Book::Haggai,
            37 => Book::Zechariah,
            38 => Book::Malachi,
            39 => Book::Matthew,
            40 => Book::Mark,
            41 => Book::Luke,
            42 => Book::John,
            43 => Book::Acts,
            44 => Book::Romans,
            45 => Book::Corinthians1,
            46 => Book::Corinthians2,
            47 => Book::Galatians,
            48 => Book::Ephesians,
            49 => Book::Philippians,
            50 => Book::Colossians,
            51 => Book::Thessalonians1,
            52 => Book::Thessalonians2,
            53 => Book::Timothy1,
            54 => Book::Timothy2,
            55 => Book::Titus,
            56 => Book::Philemon,
            57 => Book::Hebrews,
            58 => Book::James,
            59 => Book::Peter1,
            60 => Book::Peter2,
            61 => Book::John1,
            62 => Book::John2,
            63 => Book::John3,
            64 => Book::Jude,
            65 => Book::Revelation,

            _ => panic!("invalid conversion"),
        }
    }

    // No idea what I did this for.
    const fn number(self) -> u8 {
        self as u8 + 1
    }

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
        let number = number.map(u8::from);

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

    if let Err(e) = run(&args) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn run(args: &Args) -> io::Result<()> {
    #[derive(Debug, Deserialize)]
    pub struct Packet {
        resultset: Resultset,
    }

    #[derive(Debug, Deserialize)]
    pub struct Resultset {
        row: Vec<Row>,
    }

    #[derive(Debug, Deserialize)]
    pub struct Row {
        field: Vec<Field>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    pub enum Field {
        Integer(i64),
        String(String),
    }

    macro_rules! into_integer {
        ($i:ty) => {
            impl From<Field> for $i {
                fn from(field: Field) -> Self {
                    match field {
                        Field::Integer(i) => i as $i,
                        Field::String(_) => panic!("invalid conversion"),
                    }
                }
            }
        };
    }

    into_integer!(u8);
    into_integer!(u16);

    impl From<Field> for String {
        fn from(field: Field) -> Self {
            match field {
                Field::Integer(_) => panic!("invalid conversion"),
                Field::String(s) => s,
            }
        }
    }

    let data = include_str!("../../resource/kjv.json");
    let packet: Packet = serde_json::from_str(data).unwrap();

    let Resultset { row } = packet.resultset;
    let verses: Vec<Verse> = row
        .into_iter()
        .map(|mut field| {
            let mut columns = field.field.drain(1..);
            Verse {
                book: Book::from_u8(columns.next().unwrap().into()),
                chapter: columns.next().unwrap().into(),
                verse: columns.next().unwrap().into(),
                text: columns.next().unwrap().into(),
            }
        })
        .collect();

    let index = build_index(&verses);
    let location = args.location;
    let text = load(&index, location)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "verse not found"))?;

    println!("{location} {text}");

    Ok(())
}

type Index<'a> = IndexMap<Book, IndexMap<u8, IndexMap<u16, &'a str>>>;

struct Verse {
    book: Book,
    chapter: u8,
    verse: u16,
    text: String,
}

fn load<'a>(index: &Index<'a>, location: Location) -> Option<&'a str> {
    index
        .get(&location.book)?
        .get(&location.chapter)?
        .get(&location.verse)
        .copied()
}

fn build_index(verses: &[Verse]) -> Index {
    let mut index: Index = IndexMap::new();
    for verse in verses {
        index
            .entry(verse.book)
            .or_default()
            .entry(verse.chapter)
            .or_default()
            .insert(verse.verse, &verse.text);
    }
    index
}

#[cfg(test)]
mod tests {
    #[test]
    fn first_numeric_nonnumeric_transition() {
        use super::first_numeric_nonnumeric_transition as test;
        assert_eq!(Some(2), test("1 Kings"));
        assert_eq!(Some(1), test("1Kings"));
        assert_eq!(Some(5), test("Kings1"));
        assert_eq!(Some(6), test("Kings 1"));
        assert_eq!(None, test("Exodus"));
    }
}
