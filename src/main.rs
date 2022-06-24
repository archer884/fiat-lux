mod book;
mod error;
mod location;

use std::{
    borrow::Cow,
    cmp::{Ord, Ordering},
    fmt::{self, Write},
    io,
    str::FromStr,
};

use book::Book;
use clap::{Parser, Subcommand};
use comfy_table::{Attribute, Cell, CellAlignment, ContentArrangement, Table, TableComponent};
use directories::ProjectDirs;
use error::{AbbrevStr, Error};
use location::{Location, PartialLocation};
use tantivy::{
    collector::TopDocs,
    directory::MmapDirectory,
    query::{BooleanQuery, QueryParser, TermQuery},
    schema::{Facet, Field, IndexRecordOption, Schema},
    Document, Index, IndexWriter, ReloadPolicy, Term,
};

static ASV_DAT: &str = include_str!("../resource/asv.dat");
static KJV_DAT: &str = include_str!("../resource/kjv.dat");

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Clone, Debug, Parser)]
#[clap(subcommand_negates_reqs(true))]
struct Args {
    #[clap(required = true)]
    book: Option<Book>,
    location: Option<PartialLocation>,

    #[clap(flatten)]
    translation: TranslationArgs,

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

#[derive(Clone, Copy, Debug, Parser)]
#[clap(group(clap::ArgGroup::new("translation").required(false)))]
struct TranslationArgs {
    /// King James Version
    #[clap(long, group = "translation")]
    kjv: bool,

    /// American Standard Version
    #[clap(long, group = "translation")]
    asv: bool,
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
enum Translation {
    Kjv = 1,
    Asv = 2,
}

impl Translation {
    fn text(self) -> &'static str {
        match self {
            Translation::Kjv => KJV_DAT,
            Translation::Asv => ASV_DAT,
        }
    }

    fn facet(self) -> Facet {
        Facet::from(&format!("/{self}"))
    }
}

impl FromStr for Translation {
    type Err = ParseTranslationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_uppercase().as_str() {
            "KJV" => Ok(Translation::Kjv),
            "ASV" => Ok(Translation::Asv),
            _ => Err(ParseTranslationError::new(s)),
        }
    }
}

impl From<TranslationArgs> for Translation {
    fn from(args: TranslationArgs) -> Self {
        if args.asv {
            Translation::Asv
        } else {
            Translation::Kjv
        }
    }
}

impl fmt::Display for Translation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Translation::Kjv => f.write_str("KJV"),
            Translation::Asv => f.write_str("ASV"),
        }
    }
}

#[derive(Clone, Debug, thiserror::Error)]
#[error("unknown translation '{text}'")]
struct ParseTranslationError {
    text: String,
}

impl ParseTranslationError {
    fn new(text: impl AbbrevStr) -> Self {
        Self { text: text.get(7) }
    }
}

#[derive(Clone, Debug)]
struct Text {
    translation: Translation,
    book: Book,
    chapter: u16,
    verse: u16,
    content: String,
}

impl Text {
    fn from_document(document: Document, fields: &SearchFields) -> Self {
        let translation = document
            .get_first(fields.translation)
            .unwrap()
            .as_facet()
            .unwrap()
            .to_string();
        let translation: Translation = translation.trim_start_matches('/').parse().unwrap();

        let location = document
            .get_first(fields.location)
            .unwrap()
            .as_facet()
            .unwrap()
            .to_string();
        let mut segments = location.trim_start_matches('/').split('/');

        let book = segments.next().unwrap().parse::<u8>().unwrap().into();
        let chapter = segments.next().unwrap().parse().unwrap();
        let verse = segments.next().unwrap().parse().unwrap();

        let content = document
            .get_first(fields.content)
            .unwrap()
            .as_text()
            .unwrap()
            .into();

        Self {
            book,
            chapter,
            verse,
            content,
            translation,
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
        match self.book.cmp(&other.book) {
            Ordering::Equal => match self.chapter.cmp(&other.chapter) {
                Ordering::Equal => self.verse.cmp(&other.verse),
                ordering => ordering,
            },
            ordering => ordering,
        }
    }
}

impl PartialOrd for Text {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

struct SearchFields {
    translation: Field,
    location: Field,
    content: Field,
}

impl SearchFields {
    fn from_schema(schema: &Schema) -> Self {
        Self {
            translation: schema.get_field("translation").unwrap(),
            location: schema.get_field("location").unwrap(),
            content: schema.get_field("content").unwrap(),
        }
    }
}

fn main() {
    let args = Args::parse();
    if let Err(e) = run(&args) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn run(args: &Args) -> Result<()> {
    if let Some(command) = &args.command {
        return dispatch(command, args.translation.into());
    }

    let book = args.book.expect("unreachable");
    let (index, fields) = initialize_search()?;
    let texts = search_by_book_and_location(
        &index,
        &fields,
        book,
        args.location,
        args.translation.into(),
    )?;

    if texts.len() == 1 {
        let Text {
            translation,
            book,
            chapter,
            verse,
            content,
        } = texts.into_iter().next().unwrap();
        println!("{book} {chapter}:{verse}\n{content}");
    } else {
        format_texts(&texts);
    }

    Ok(())
}

fn format_texts(texts: &[Text]) {
    let (w, h) = terminal_size::terminal_size()
        .map(|(terminal_size::Width(w), terminal_size::Height(h))| (w, h))
        .unwrap_or((80, 20));

    if texts.len() > h as usize {
        pager::Pager::with_default_pager("bat").setup();
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Chapter {
        book: Book,
        chapter: u16,
    }

    impl Text {
        fn chapter(&self) -> Chapter {
            Chapter {
                book: self.book,
                chapter: self.chapter,
            }
        }
    }

    let mut current: Option<Chapter> = None;
    let mut table = Table::new();

    table.set_content_arrangement(ContentArrangement::DynamicFullWidth);
    table.load_preset(comfy_table::presets::NOTHING);
    table.set_width(w.min(100));

    for text in texts {
        if current.is_none()
            || !current
                .map(|chapter| chapter == text.chapter())
                .unwrap_or_default()
        {
            let next = text.chapter();
            let Chapter { book, chapter } = next;
            current = Some(next);
            table.add_row(vec![
                Cell::new(""),
                Cell::new(format!("\n{book} {chapter}")).add_attribute(Attribute::Bold),
            ]);
        }

        let verse = text.verse;
        let content = &text.content;
        table.add_row(&[Cow::from(format!("{verse:4}")), Cow::from(content)]);
    }

    table
        .column_mut(0)
        .unwrap()
        .set_cell_alignment(CellAlignment::Right);

    println!("{table}");
}

fn search_by_book_and_location(
    index: &Index,
    fields: &SearchFields,
    book: Book,
    location: Option<PartialLocation>,
    translation: Translation,
) -> tantivy::Result<Vec<Text>> {
    let mut buf = format!("/{}", book as u8);
    if let Some(location) = &location {
        let chapter = location.chapter;
        write!(buf, "/{chapter}").unwrap();
        if let Some(verse) = location.verse {
            write!(buf, "/{verse}").unwrap()
        }
    }

    let location = TermQuery::new(
        Term::from_facet(fields.location, &Facet::from(&buf)),
        IndexRecordOption::Basic,
    );
    let translation = TermQuery::new(
        Term::from_facet(fields.translation, &Facet::from(&format!("/{translation}"))),
        IndexRecordOption::Basic,
    );
    let query = BooleanQuery::intersection(vec![Box::new(location), Box::new(translation)]);

    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommit)
        .try_into()?;
    let searcher = reader.searcher();
    // In this case, we don't actually want to limit the docs returned, and the number will be
    // small in most cases, but I have no idea what collector to use or how, so...
    let documents = searcher
        .search(&query, &TopDocs::with_limit(10_000))?
        .into_iter()
        .map(|(_, candidate)| searcher.doc(candidate));

    let mut texts = Vec::new();
    for document in documents {
        texts.push(Text::from_document(document?, fields));
    }
    texts.sort();
    Ok(texts)
}

fn dispatch(command: &Command, translation: Translation) -> Result<()> {
    match command {
        // It is not obvious to me that a search should be performed against a given translation
        // rather than all translations, but we can revisit this later.
        Command::Search(args) => search(args, translation),
    }
}

fn search(args: &SearchArgs, translation: Translation) -> Result<()> {
    let (index, fields) = initialize_search()?;

    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommit)
        .try_into()?;
    let searcher = reader.searcher();

    // This query parser constructs a query from the user's search string. We can break the search
    // string into multiple strings at some point to make the cli less annoying, maybe? But for now
    // the user provides a monolithic string.

    let query_parser = QueryParser::for_index(&index, vec![fields.content]);
    let query = query_parser.parse_query(&args.query)?;

    // That gives us one search term. We need to make a second term for the facet referencing the
    // correct translation.

    let translation_term = Term::from_facet(fields.translation, &translation.facet());
    let term_query = TermQuery::new(translation_term, IndexRecordOption::Basic);

    // Damned if I know the correct way to do this, but this seems to work, so....

    let combined_query = BooleanQuery::intersection(vec![query, Box::new(term_query)]);
    let mut texts: Vec<_> = searcher
        .search(
            &combined_query,
            &TopDocs::with_limit(args.limit.unwrap_or(10)),
        )?
        .into_iter()
        .filter_map(|(_, address)| searcher.doc(address).ok())
        .map(|document| Text::from_document(document, &fields))
        .collect();

    texts.sort();
    format_texts(&texts);

    Ok(())
}

fn initialize_search() -> tantivy::Result<(Index, SearchFields)> {
    // We want to store our data someplace sane, so we're gonna use the directories library to
    // decide where all this data goes.

    let dirs = ProjectDirs::from("org", "Hack Commons", "Bible-App").ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::Other,
            "unable to initialize project directory",
        )
    })?;

    // Well need to ensure the directory exists. That's easy, but I'm not sure how to know if
    // there is an existing index in an existing directory. That seems important.

    let index_path = dirs.data_dir().join("bible_idx");
    if !index_path.exists() {
        std::fs::create_dir_all(&index_path)?;
    }

    let schema = build_schema();
    let fields = SearchFields::from_schema(&schema);

    let index_dir = MmapDirectory::open(&index_path)?;
    if !tantivy::Index::exists(&index_dir)? {
        let index = Index::create_in_dir(index_path, schema)?;
        // Index using 50 megabytes of memory
        write_index(Translation::Kjv, &fields, &mut index.writer(0x3200000)?)?;
        write_index(Translation::Asv, &fields, &mut index.writer(0x3200000)?)?;
        Ok((index, fields))
    } else {
        Ok((tantivy::Index::open(index_dir)?, fields))
    }
}

fn write_index(
    translation: Translation,
    fields: &SearchFields,
    writer: &mut IndexWriter,
) -> tantivy::Result<()> {
    use tantivy::doc;

    for (id, text) in parse_verses_with_id(translation.text()) {
        let Location {
            book,
            chapter,
            verse,
        } = Location::from_id(id);

        let book = book as u8;
        let location = Facet::from(&format!("/{book}/{chapter}/{verse}"));
        let translation = Facet::from(&format!("/{translation}"));

        writer.add_document(doc!(
            fields.translation => translation,
            fields.location => location,
            fields.content => text,
        ))?;
    }

    writer.commit()?;
    Ok(())
}

fn build_schema() -> Schema {
    use tantivy::schema;

    let facet_options = schema::INDEXED | schema::STORED;

    let mut builder = Schema::builder();
    builder.add_facet_field("translation", facet_options.clone());
    builder.add_facet_field("location", facet_options);
    builder.add_text_field("content", schema::TEXT | schema::STORED);
    builder.build()
}

fn parse_verses_with_id(text: &str) -> impl Iterator<Item = (u64, &str)> {
    text.lines()
        .filter_map(|line| line[..8].parse::<u64>().ok().map(|id| (id, &line[9..])))
}
