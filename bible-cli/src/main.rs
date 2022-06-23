use std::{
    fmt,
    num::{NonZeroU8, ParseIntError},
    str::FromStr,
};

use clap::{Parser, Subcommand};
use indexmap::IndexMap;

static ASV_DAT: &str = include_str!("../../resource/asv.dat");
static KJV_DAT: &str = include_str!("../../resource/kjv.dat");

type Result<T, E = Error> = std::result::Result<T, E>;

type Index<'a> = IndexMap<Book, BookIndex<'a>>;

type BookIndex<'a> = IndexMap<u16, ChapterIndex<'a>>;

type ChapterIndex<'a> = IndexMap<u16, &'a str>;

fn edit_distance(query: &str, text: &str) -> Option<usize> {
    if query.len() > text.len() {
        return None;
    }

    let query = query.as_bytes();
    let text = text.as_bytes();

    text.windows(query.len())
        .filter(|&window| window.starts_with(&query[..1]))
        .map(|window| get_distance(query, window))
        .min()
}

fn get_distance(a: &[u8], b: &[u8]) -> usize {
    a.iter()
        .copied()
        .zip(b.iter().copied())
        .filter(|(a, b)| a != b)
        .count()
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    NotFound(NotFound),
}

#[derive(Debug, thiserror::Error)]
struct NotFound {
    entity: Entity,
    book: Book,
    location: Option<Location>,
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
enum Entity {
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

#[derive(Clone, Debug, Parser)]
#[clap(subcommand_negates_reqs(true))]
struct Args {
    #[clap(required = true)]
    book: Option<Book>,
    location: Option<Location>,

    #[clap(flatten)]
    translation: Translations,

    #[clap(subcommand)]
    command: Option<Command>,
}

#[derive(Clone, Debug, Subcommand)]
enum Command {
    #[clap(alias = "s")]
    Search { query: String },
}

#[derive(Clone, Debug, Parser)]
#[clap(group(clap::ArgGroup::new("translation").required(false)))]
struct Translations {
    /// King James Version
    #[clap(long, group = "translation")]
    kjv: bool,

    /// American Standard Version
    #[clap(long, group = "translation")]
    asv: bool,
}

/// A full designator of book, chapter, and verse.
#[derive(Clone, Copy, Debug)]
struct Location {
    chapter: u16,
    verse: Option<u16>,
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let chapter = self.chapter;
        match self.verse {
            Some(verse) => write!(f, "[{chapter}:{verse}]"),
            None => write!(f, "[{chapter}]"),
        }
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

        let (chapter, verse) = s.split_once(':').unwrap_or((s, ""));

        // For right now, we're not going to check the book's name, because... well, whatever. We
        // are gonna implement that later.

        let chapter = chapter
            .parse()
            .map_err(|e| ParseVerseError::chapter(chapter, e))?;

        if verse.is_empty() {
            Ok(Location {
                chapter,
                verse: None,
            })
        } else {
            let verse: u16 = verse
                .parse()
                .map_err(|e| ParseVerseError::verse(verse, e))?;
            Ok(Location {
                chapter,
                verse: Some(verse),
            })
        }
    }
}

#[derive(Clone, Debug, thiserror::Error)]
enum ParseVerseError {
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
    // Fun fact: setting Genesis as 1 causes this enum to be 1-based, which I am hoping will
    // enable the optimization where the None variant will simply appear as zero.
    Genesis = 1,    // Genesis
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
        match u {
            1 => Book::Genesis,
            2 => Book::Exodus,
            3 => Book::Leviticus,
            4 => Book::Numbers,
            5 => Book::Deuteronomy,
            6 => Book::Joshua,
            7 => Book::Judges,
            8 => Book::Ruth,
            9 => Book::Samuel1,
            10 => Book::Samuel2,
            11 => Book::Kings1,
            12 => Book::Kings2,
            13 => Book::Chronicles1,
            14 => Book::Chronicles2,
            15 => Book::Ezra,
            16 => Book::Nehemiah,
            17 => Book::Esther,
            18 => Book::Job,
            19 => Book::Psalms,
            20 => Book::Proverbs,
            21 => Book::Ecclesiastes,
            22 => Book::SongofSongs,
            23 => Book::Isaiah,
            24 => Book::Jeremiah,
            25 => Book::Lamentations,
            26 => Book::Ezekiel,
            27 => Book::Daniel,
            28 => Book::Hosea,
            29 => Book::Joel,
            30 => Book::Amos,
            31 => Book::Obadiah,
            32 => Book::Jonah,
            33 => Book::Micah,
            34 => Book::Nahum,
            35 => Book::Habakkuk,
            36 => Book::Zephaniah,
            37 => Book::Haggai,
            38 => Book::Zechariah,
            39 => Book::Malachi,
            40 => Book::Matthew,
            41 => Book::Mark,
            42 => Book::Luke,
            43 => Book::John,
            44 => Book::Acts,
            45 => Book::Romans,
            46 => Book::Corinthians1,
            47 => Book::Corinthians2,
            48 => Book::Galatians,
            49 => Book::Ephesians,
            50 => Book::Philippians,
            51 => Book::Colossians,
            52 => Book::Thessalonians1,
            53 => Book::Thessalonians2,
            54 => Book::Timothy1,
            55 => Book::Timothy2,
            56 => Book::Titus,
            57 => Book::Philemon,
            58 => Book::Hebrews,
            59 => Book::James,
            60 => Book::Peter1,
            61 => Book::Peter2,
            62 => Book::John1,
            63 => Book::John2,
            64 => Book::John3,
            65 => Book::Jude,
            66 => Book::Revelation,

            _ => panic!("invalid conversion"),
        }
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

impl From<u8> for Book {
    fn from(u: u8) -> Self {
        Book::from_u8(u)
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

fn run(args: &Args) -> Result<()> {
    let text = if args.translation.asv {
        ASV_DAT
    } else {
        KJV_DAT
    };

    if let Some(command) = &args.command {
        return dispatch(command, text);
    }

    let book = args.book.expect("unreachable");
    let index = build_index(text);
    let book_index = index.get(&book).ok_or(Error::NotFound(NotFound {
        entity: Entity::Book,
        book,
        location: None,
    }))?;

    match args.location {
        Some(location) => load_and_print(book, location, book_index)?,
        None => print_book(book, book_index),
    }

    Ok(())
}

fn dispatch(command: &Command, text: &str) -> Result<()> {
    match command {
        // It is not obvious to me that a search should be performed against a given translation
        // rather than all translations, but we can revisit this later.
        Command::Search { query } => search(query, text),
    }
    Ok(())
}

fn search(query: &str, text: &str) {
    let query = query.to_ascii_uppercase();
    let mut text_by_distance: Vec<_> = text
        .lines()
        .filter_map(|line| {
            edit_distance(&query, &line[9..].to_ascii_uppercase()).map(|distance| (distance, line))
        })
        .collect();

    text_by_distance.sort_by_key(|x| x.0);

    let candidates = text_by_distance.into_iter().map(|(_, text)| text).take(10);
    format_candidates(candidates);
}

fn format_candidates<'a>(candidates: impl IntoIterator<Item = &'a str>) {
    // These candidates are the raw content of the dat file, meaning that each one includes a
    // unique identifier which may be decomposed into book, chapter, verse, etc.
    for candidate in candidates {
        let book = Book::from_u8(candidate[..2].parse().unwrap());
        let chapter: u16 = candidate[2..5].parse().unwrap();
        let verse: u16 = candidate[5..8].parse().unwrap();
        let text = &candidate[9..];
        println!("{book} [{chapter}:{verse}] {text}");
    }
}

fn print_book(book: Book, index: &BookIndex) {
    for (chapter, chapter_index) in index {
        println!("{book} {chapter}:");
        print_chapter(chapter_index);
    }
}

fn print_chapter(index: &ChapterIndex) {
    println!();
    for (&verse, &text) in index {
        println!("{verse} {text}");
    }
    println!();
}

fn load_and_print(book: Book, location: Location, index: &BookIndex) -> Result<()> {
    let chapter_index = index
        .get(&location.chapter)
        .ok_or(Error::NotFound(NotFound {
            entity: Entity::Chapter,
            book,
            location: Some(location),
        }))?;

    if let Some(verse) = location.verse {
        let &verse = chapter_index.get(&verse).ok_or(Error::NotFound(NotFound {
            entity: Entity::Verse,
            book,
            location: Some(location),
        }))?;
        println!("{book}\n{location} {verse}");
    } else {
        let chapter = location.chapter;
        println!("{book} {chapter}:");
        print_chapter(chapter_index);
    }

    Ok(())
}

fn build_index(text: &str) -> Index {
    let mut index: Index = IndexMap::new();

    for record in text.lines() {
        let book = Book::from_u8(record[..2].parse().unwrap());
        let chapter: u16 = record[2..5].parse().unwrap();
        let verse: u16 = record[5..8].parse().unwrap();
        let text = &record[9..];

        index
            .entry(book)
            .or_default()
            .entry(chapter)
            .or_default()
            .insert(verse, text);
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
