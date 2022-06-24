mod book;
mod error;
mod location;
mod search;

use book::Book;
use clap::{Parser, Subcommand};
use error::{Error, NotFound, Entity};
use indexmap::IndexMap;
use location::Location;
use search::SplitWindows;

static ASV_DAT: &str = include_str!("../resource/asv.dat");
static KJV_DAT: &str = include_str!("../resource/kjv.dat");

type Result<T, E = Error> = std::result::Result<T, E>;
type Index<'a> = IndexMap<Book, BookIndex<'a>>;
type BookIndex<'a> = IndexMap<u16, ChapterIndex<'a>>;
type ChapterIndex<'a> = IndexMap<u16, &'a str>;

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
    Search(SearchArgs),
}

#[derive(Clone, Debug, Parser)]
struct SearchArgs {
    query: String,
    #[clap(short, long)]
    limit: Option<usize>,
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
        Command::Search(args) => search(args, text),
    }
    Ok(())
}

fn search(args: &SearchArgs, text: &str) {
    let splitter = SplitWindows::new();
    let query = args.query.to_ascii_uppercase();

    let mut text_by_distance: Vec<_> = text
        .lines()
        .filter_map(|line| {
            edit_distance(&query, &line[9..].to_ascii_uppercase(), &splitter)
                .map(|distance| (distance, line))
        })
        .collect();

    text_by_distance.sort_by_key(|x| x.0);

    let candidates = text_by_distance.into_iter().map(|(_, text)| text).take(args.limit.unwrap_or(10));
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

fn edit_distance(query: &str, text: &str, splitter: &SplitWindows) -> Option<usize> {
    if query.len() > text.len() {
        return None;
    }

    let query = query.as_bytes();
    let windows = splitter.windows(text, query.len());

    windows
        .filter_map(|window| {
            let window = window.as_bytes();
            if window[0] == query[0] {
                Some(get_distance(query, window))
            } else {
                None
            }
        })
        .min()
}

fn get_distance(a: &[u8], b: &[u8]) -> usize {
    a.iter()
        .copied()
        .zip(b.iter().copied())
        .filter(|(a, b)| a != b)
        .count()
}
