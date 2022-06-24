mod book;
mod error;
mod location;

use book::Book;
use clap::{Parser, Subcommand};
use error::{Entity, Error, NotFound};
use indexmap::IndexMap;
use location::Location;
use tantivy::{
    collector::TopDocs, query::QueryParser, schema::Schema, Index,
    IndexWriter, ReloadPolicy,
};

static ASV_DAT: &str = include_str!("../resource/asv.dat");
static KJV_DAT: &str = include_str!("../resource/kjv.dat");

type Result<T, E = Error> = std::result::Result<T, E>;
type FullIndex<'a> = IndexMap<Book, BookIndex<'a>>;
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
}

fn search(args: &SearchArgs, text: &str) -> Result<()> {
    let schema = build_schema();
    let location = schema.get_field("location").unwrap();
    let content = schema.get_field("content").unwrap();

    let index = Index::create_in_ram(schema.clone());
    write_index(text, &schema, &mut index.writer(0x3200000)?)?;

    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommit)
        .try_into()?;
    let searcher = reader.searcher();
    let query_parser = QueryParser::for_index(&index, vec![content]);
    let query = query_parser.parse_query(&args.query)?;
    let candidates = searcher.search(&query, &TopDocs::with_limit(args.limit.unwrap_or(10)))?;

    for (_score, address) in candidates {
        let retrieved = searcher.doc(address)?;
        let location = retrieved.get_first(location).unwrap().as_u64().unwrap();
        let content = retrieved.get_first(content).unwrap().as_text().unwrap();

        let (book, location) = decompose_id(location);

        println!("{book} {location}\n{content}");
    }

    Ok(())
}

fn decompose_id(id: u64) -> (Book, Location) {
    // Just the one or two most significant digits matter for the book id.
    // 01001001

    let chapter = (id % 1_000_000 / 1000) as u16;
    let verse = (id % 1000) as u16;

    (
        ((id / 1_000_000) as u8).into(),
        Location {
            chapter,
            verse: Some(verse),
        },
    )
}

fn write_index(text: &str, schema: &Schema, writer: &mut IndexWriter) -> tantivy::Result<()> {
    use tantivy::doc;

    let location = schema.get_field("location").unwrap();
    let content = schema.get_field("content").unwrap();

    for (id, text) in parse_verses_with_id(text) {
        writer.add_document(doc!(
            location => id,
            content => text,
        ))?;
    }
    
    writer.commit()?;
    Ok(())
}

fn build_schema() -> Schema {
    use tantivy::schema;
    let mut builder = Schema::builder();
    builder.add_u64_field("location", schema::STORED);
    builder.add_text_field("content", schema::TEXT | schema::STORED);
    builder.build()
}

fn parse_verses_with_id(text: &str) -> impl Iterator<Item = (u64, &str)> {
    text.lines()
        .filter_map(|line| line[..8].parse::<u64>().ok().map(|id| (id, &line[9..])))
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

fn build_index(text: &str) -> FullIndex {
    let mut index: FullIndex = IndexMap::new();

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
