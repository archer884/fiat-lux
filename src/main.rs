mod book;
mod error;
mod location;
mod reference;
mod search;
mod text;

use std::{
    borrow::Cow,
    fmt::{self, Write},
    io,
    str::FromStr,
};

use crate::reference::Reference;
use book::Book;
use clap::{Parser, Subcommand};
use comfy_table::{Attribute, Cell, CellAlignment, ContentArrangement, Table};
use directories::ProjectDirs;
use error::{AbbrevStr, Error};
use location::{Location, PartialLocation};
use reference::ReferenceProvider;
use search::SearchFields;
use tantivy::{
    collector::TopDocs,
    directory::MmapDirectory,
    query::{BooleanQuery, QueryParser, TermQuery},
    schema::{Facet, IndexRecordOption, Schema},
    Index, IndexWriter, ReloadPolicy, Term,
};
use text::{Chapter, Text};

static ASV_DAT: &str = include_str!("../resource/asv.dat");
static KJV_DAT: &str = include_str!("../resource/kjv.dat");

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Clone, Debug, Parser)]
#[command(subcommand_negates_reqs(true))]
struct Args {
    #[clap(required = true)]
    book: Option<Book>,
    location: Option<PartialLocation>,

    #[clap(flatten)]
    translation: TranslationArgs,

    #[clap(long, env = "FIAT_LUX_REFERENCE", default_value_t)]
    reference: ReferenceProvider,

    #[clap(subcommand)]
    command: Option<Command>,
}

#[derive(Clone, Debug, Subcommand)]
enum Command {
    #[clap(alias = "s")]
    Search(SearchArgs),

    #[clap(hide(true), alias = "Austin")]
    Austin { location: PartialLocation },
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

fn main() {
    let args = Args::parse();
    if let Err(e) = run(&args) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn run(args: &Args) -> Result<()> {
    let reference = args.reference.get();

    if let Some(command) = &args.command {
        return dispatch(command, args.translation.into(), &*reference);
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
            book,
            chapter,
            verse,
            content,
        } = texts.into_iter().next().unwrap();
        let width =
            terminal_size::terminal_size().map_or(100, |(terminal_size::Width(w), _)| w.min(100));
        let content = textwrap::fill(&content, usize::from(width));
        let location = Location {
            book,
            chapter,
            verse,
        };
        let url = reference.url(&location, args.translation.into());
        println!("{book} {chapter}:{verse}\n{content}\n{url}");
    } else {
        format_texts(&texts, &*reference, args.translation.into());
    }

    Ok(())
}

fn format_texts(texts: &[Text], reference: &dyn Reference, translation: Translation) {
    #[cfg(feature = "pager")]
    let width = {
        let (w, h) = terminal_size::terminal_size()
            .map(|(terminal_size::Width(w), terminal_size::Height(h))| (w, h))
            .unwrap_or((100, 20));
        if texts.len() > h as usize {
            pager::Pager::with_default_pager("bat").setup();
        }
        w
    };

    #[cfg(not(feature = "pager"))]
    let width = {
        let (w, _h) = terminal_size::terminal_size()
            .map(|(terminal_size::Width(w), terminal_size::Height(h))| (w, h))
            .unwrap_or((100, 20));
        w
    };

    let mut current: Option<Chapter> = None;
    let mut table = Table::new();
    let mut section_verse_count = 0;

    table.set_content_arrangement(ContentArrangement::DynamicFullWidth);
    table.load_preset(comfy_table::presets::NOTHING);
    table.set_width(width.min(100));

    for text in texts {
        if current.is_none() {
            let next = text.chapter();
            let Chapter { book, chapter } = next;
            current = Some(next);
            table.add_row(vec![
                Cell::new(""),
                Cell::new(format!("\n{book} {chapter}")).add_attribute(Attribute::Bold),
            ]);
        } else if !current
            .map(|chapter| chapter == text.chapter())
            .unwrap_or_default()
        {
            append_reference(
                reference,
                translation,
                &mut table,
                text,
                section_verse_count > 1,
            );

            let next = text.chapter();
            let Chapter { book, chapter } = next;
            current = Some(next);
            table.add_row(vec![
                Cell::new(""),
                Cell::new(format!("\n{book} {chapter}")).add_attribute(Attribute::Bold),
            ]);

            section_verse_count = 0;
        }

        let verse = text.verse;
        let content = &text.content;
        section_verse_count += 1;
        table.add_row(&[Cow::from(format!("{verse:4}")), Cow::from(content)]);
    }

    if let Some(text) = texts.last() {
        append_reference(
            reference,
            translation,
            &mut table,
            text,
            section_verse_count > 1,
        );
    }

    if let Some(col) = table.column_mut(0) {
        col.set_cell_alignment(CellAlignment::Right);
    }

    println!("{table}");
}

fn append_reference(
    reference: &dyn Reference,
    translation: Translation,
    table: &mut Table,
    text: &Text,
    has_multiple_verses: bool,
) {
    // In the event we've emitted multiple verses in this "section," we need to only emit
    // a chapter link -- NOT a full verse link.
    if has_multiple_verses {
        table.add_row(vec![
            Cell::new(""),
            Cell::new(reference.url(&text.chapter(), translation)),
        ]);
    } else {
        table.add_row(vec![
            Cell::new(""),
            Cell::new(reference.url(&text, translation)),
        ]);
    }
}

fn search_by_book_and_location(
    index: &Index,
    fields: &SearchFields,
    book: Book,
    location: Option<PartialLocation>,
    translation: Translation,
) -> tantivy::Result<Vec<Text>> {
    // In the original version, we created a facet of the form /a/b?/c?, where a and b were book
    // and chapter and c was the verse. In this version, we're going to go with /a/b? and leave
    // c out entirely, because I doubt very seriously that losing the verse from search is going
    // to do significant harm to performance AND because we'll be able to do ranges of verses more
    // easily this way.

    let mut buf = format!("/{}", book as u8);
    if let Some(location) = &location {
        write!(buf, "/{}", location.chapter).unwrap();
    }

    let location_facet = Facet::from(&buf);
    let translation_facet = Facet::from(&format!("/{translation}"));

    let location_query = TermQuery::new(
        Term::from_facet(fields.location, &location_facet),
        IndexRecordOption::Basic,
    );

    let translation_query = TermQuery::new(
        Term::from_facet(fields.translation, &translation_facet),
        IndexRecordOption::Basic,
    );

    let query =
        BooleanQuery::intersection(vec![Box::new(location_query), Box::new(translation_query)]);

    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::Manual)
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

    // Now that we've got a set of verses to work with, IF the user has provided us a set of verses
    // to return, we want to filter out any verses NOT included in the user's specification. Since
    // the verses are already on a vec, I figure the easiest way to do this is just with a retain
    // call.

    if let Some(verse) = location.and_then(|x| x.verse) {
        texts.retain(|text| verse.contains(text.verse));
    }

    Ok(texts)
}

fn dispatch(command: &Command, translation: Translation, reference: &dyn Reference) -> Result<()> {
    match command {
        // It is not obvious to me that a search should be performed against a given translation
        // rather than all translations, but we can revisit this later.
        Command::Search(args) => search(args, translation, reference),

        // This code does not exist. Do not read this code.
        // Also don't watch this video:
        // https://www.youtube.com/watch?v=tjWPoQWdmjg
        Command::Austin { location } => {
            let expected = PartialLocation {
                chapter: 3,
                verse: Some(location::AUSTIN_VERSE),
            };

            if location == &expected {
                println!("Austin 3:16\nI just whipped your ass!");
            }

            Ok(())
        }
    }
}

fn search(args: &SearchArgs, translation: Translation, reference: &dyn Reference) -> Result<()> {
    let (index, fields) = initialize_search()?;

    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::Manual)
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
    format_texts(&texts, reference, translation);

    Ok(())
}

fn initialize_search() -> tantivy::Result<(Index, SearchFields)> {
    // We want to store our data someplace sane, so we're gonna use the directories library to
    // decide where all this data goes.
    let dirs = ProjectDirs::from("org", "Hack Commons", "Bible-App")
        .ok_or_else(|| io::Error::other("unable to initialize project directory"))?;

    // Well need to ensure the directory exists. That's easy, but I'm not sure how to know if
    // there is an existing index in an existing directory. That seems important.

    let index_path = dirs.data_dir().join("bible_idx");
    if !index_path.exists() {
        std::fs::create_dir_all(&index_path)?;
    }

    let (schema, fields) = build_schema();
    let index_dir = MmapDirectory::open(&index_path)?;

    if !tantivy::Index::exists(&index_dir)? {
        let index = Index::create_in_dir(index_path, schema)?;

        /// 500 megabytes
        const ARENA_SIZE: usize = 0x100000 * 500;
        write_index(Translation::Kjv, &fields, &mut index.writer(ARENA_SIZE)?)?;
        write_index(Translation::Asv, &fields, &mut index.writer(ARENA_SIZE)?)?;
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

fn build_schema() -> (Schema, SearchFields) {
    use tantivy::schema;

    let facet_options = schema::INDEXED | schema::STORED;
    let mut builder = Schema::builder();
    let fields = SearchFields {
        translation: builder.add_facet_field("translation", facet_options.clone()),
        location: builder.add_facet_field("location", facet_options),
        content: builder.add_text_field("content", schema::TEXT | schema::STORED),
    };

    (builder.build(), fields)
}

fn parse_verses_with_id(text: &str) -> impl Iterator<Item = (u64, &str)> {
    text.lines()
        .filter_map(|line| line[..8].parse::<u64>().ok().map(|id| (id, &line[9..])))
}
