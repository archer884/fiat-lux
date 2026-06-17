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
    path::Path,
    str::FromStr,
};

use book::Book;
use clap::{Parser, Subcommand};
use comfy_table::{Attribute, Cell, CellAlignment, ContentArrangement, Table};
use directories::ProjectDirs;
use error::{AbbrevStr, Error};
use indexmap::IndexMap;
use location::{Location, PartialLocation};
use reference::Reference;
use reference::ReferenceProvider;
use search::SearchFields;
use tantivy::{
    Index, IndexWriter, ReloadPolicy, Term,
    collector::TopDocs,
    directory::MmapDirectory,
    query::{BooleanQuery, QueryParser, TermQuery},
    schema::{Facet, IndexRecordOption, Schema},
};
use text::Text;

static ASV_DAT: &str = include_str!("../resource/asv.dat");
static KJV_DAT: &str = include_str!("../resource/kjv.dat");

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Clone, Debug, Parser)]
#[command(subcommand_negates_reqs(true))]
struct Args {
    #[arg(required = true)]
    book: Option<Book>,
    location: Option<PartialLocation>,

    #[command(flatten)]
    translation: TranslationArgs,

    #[arg(long, env = "FIAT_LUX_REFERENCE", default_value_t)]
    reference: ReferenceProvider,

    /// Memory budget for the index writer during index creation (e.g. "500MB", "2GB").
    /// Only applies when the index is being built; subsequent runs just open the existing index.
    #[arg(long, env = "FIAT_LUX_INDEX_MEMORY")]
    index_memory: Option<ByteSize>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Clone, Debug, Subcommand)]
enum Command {
    #[clap(alias = "s")]
    Search(SearchArgs),

    /// Build the search index.
    CreateIndex {
        /// Overwrite an existing index.
        #[arg(long)]
        force: bool,
    },

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

/// A memory size parsed from a human-friendly string like `"500MB"` or `"2GB"`.
#[derive(Clone, Copy, Debug)]
struct ByteSize(usize);

impl FromStr for ByteSize {
    type Err = ParseByteSizeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let split = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
        let (num, suffix) = s.split_at(split);
        let num: usize = num.parse().map_err(|_| ParseByteSizeError)?;
        let multiplier = match suffix.trim().to_ascii_uppercase().as_str() {
            "" | "B" => 1,
            "K" | "KB" => 1024,
            "M" | "MB" => 1024 * 1024,
            "G" | "GB" => 1024 * 1024 * 1024,
            _ => return Err(ParseByteSizeError),
        };
        num.checked_mul(multiplier)
            .map(ByteSize)
            .ok_or(ParseByteSizeError)
    }
}

#[derive(Clone, Debug, thiserror::Error)]
#[error("expected a number with an optional KB/MB/GB suffix, e.g. '500MB' or '2GB'")]
struct ParseByteSizeError;

fn default_arena_size() -> usize {
    let cpus = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    // 100MB per thread (tantivy caps at 8 threads), floored at 500MB.
    (cpus * 100 * 1024 * 1024).max(500 * 1024 * 1024)
}

fn main() {
    let args = Args::parse();
    let arena_size = args
        .index_memory
        .map(|b| b.0)
        .unwrap_or_else(default_arena_size);
    if let Err(e) = run(&args, arena_size) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn run(args: &Args, arena_size: usize) -> Result<()> {
    let reference = args.reference.get();

    if let Some(command) = &args.command {
        return dispatch(command, args.translation.into(), &*reference, arena_size);
    }

    let book = args.book.expect("unreachable");
    let (index, fields) = initialize_search(arena_size)?;
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
    // Bail out early with a clear message rather than printing an empty table.
    // This also avoids setting up the pager for a single line of output.
    if texts.is_empty() {
        println!("No verses found.");
        return;
    }

    let (width, _height) = terminal_size::terminal_size()
        .map(|(terminal_size::Width(w), terminal_size::Height(h))| (w, h))
        .unwrap_or((100, 20));

    // We need to group verses by book and chapter without scrambling their order. The accepted
    // means for accomplishing this appears to be either ordermap or indexmap. I'm going with the
    // latter because the documentation *says* that ordermap provides stronger ordering guarantees,
    // and I'm pretty sure I don't need them to be that strong. (I'm not removing any items.)

    let mut groups = IndexMap::new();
    for text in texts {
        groups
            .entry(text.chapter())
            .and_modify(|x: &mut Vec<_>| x.push(text))
            .or_insert(vec![text]);
    }

    let mut table = Table::new();

    table.set_content_arrangement(ContentArrangement::DynamicFullWidth);
    table.load_preset(comfy_table::presets::NOTHING);
    table.set_width(width.min(100));

    // Now we have each verse grouped with other verses in the same book and chapter, and we can
    // easily print one link per group rather than attempting to infer link placement on the basis
    // of which chapter heading we read last.

    for (loc, verses) in groups {
        table.add_row(vec![
            Cell::new(""),
            Cell::new(format!("\n{} {}", loc.book, loc.chapter)).add_attribute(Attribute::Bold),
        ]);

        for verse in &verses {
            let Text { verse, content, .. } = verse;
            table.add_row(&[Cow::from(format!("{verse:4}")), Cow::from(content)]);
        }

        // The last thing we need to do is to append a reference here for the preceding verse or
        // verses.

        if let &[verse] = verses.as_slice() {
            table.add_row(vec![
                Cell::new(""),
                Cell::new(reference.url(&verse, translation)),
            ]);
        } else {
            table.add_row(vec![
                Cell::new(""),
                Cell::new(reference.url(&loc, translation)),
            ]);
        }
    }

    if let Some(col) = table.column_mut(0) {
        col.set_cell_alignment(CellAlignment::Right);
    }

    // Render before deciding on the pager so we can compare the *actual* output
    // height against the terminal, rather than guessing from the verse count.
    let output = table.to_string();

    #[cfg(feature = "pager")]
    if output.lines().count() > _height as usize {
        pager::Pager::with_pager("bat --style=plain --paging=always").setup();
    }

    println!("{output}");
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
    let documents = searcher
        .search(&query, &TopDocs::with_limit(10_000).order_by_score())?
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

fn dispatch(
    command: &Command,
    translation: Translation,
    reference: &dyn Reference,
    arena_size: usize,
) -> Result<()> {
    match command {
        // It is not obvious to me that a search should be performed against a given translation
        // rather than all translations, but we can revisit this later.
        Command::Search(args) => search(args, translation, reference, arena_size),

        Command::CreateIndex { force } => create_index_command(*force, arena_size),

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

fn search(
    args: &SearchArgs,
    translation: Translation,
    reference: &dyn Reference,
    arena_size: usize,
) -> Result<()> {
    let (index, fields) = initialize_search(arena_size)?;

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
            &TopDocs::with_limit(args.limit.unwrap_or(10)).order_by_score(),
        )?
        .into_iter()
        .filter_map(|(_, address)| searcher.doc(address).ok())
        .map(|document| Text::from_document(document, &fields))
        .collect();

    texts.sort();
    format_texts(&texts, reference, translation);

    Ok(())
}

fn create_index_command(force: bool, arena_size: usize) -> Result<()> {
    let dirs = ProjectDirs::from("org", "Hack Commons", "Bible-App")
        .ok_or_else(|| io::Error::other("unable to initialize project directory"))?;

    let index_path = dirs.data_dir().join("bible_idx");
    if !index_path.exists() {
        std::fs::create_dir_all(&index_path)?;
    }

    let index_exists = tantivy::Index::exists(&MmapDirectory::open(&index_path)?)?;

    if index_exists && !force {
        println!("Index already exists. Use --force to rebuild.");
        return Ok(());
    }

    if index_exists {
        std::fs::remove_dir_all(&index_path)?;
        std::fs::create_dir_all(&index_path)?;
    }

    let (schema, fields) = build_schema();
    let fingerprint = schema_fingerprint(&schema);

    let sw = chronograf::start();
    create_index(&index_path, schema, &fields, fingerprint, arena_size)?;
    let elapsed = sw.finish();

    println!("Index created in {:.2}s", elapsed.as_secs_f64());

    Ok(())
}

/// Computes a fingerprint of the schema so that *any* change to the field
/// layout (name, type, or options) automatically invalidates stale indexes.
///
/// This is derived from the `Schema` object itself rather than a
/// hand-maintained version constant, so there is nothing to remember to bump
/// when editing `build_schema`. Note that a non-schema change to the document
/// encoding in `write_index` (e.g. facet path format) won't be caught here;
/// if you make such a change, bump the salt below.
const INDEX_FORMAT_SALT: u64 = 0;

fn schema_fingerprint(schema: &Schema) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    INDEX_FORMAT_SALT.hash(&mut hasher);
    for (_, entry) in schema.fields() {
        entry.name().hash(&mut hasher);
        // `FieldType` doesn't implement `Hash`, but its `Debug` output captures
        // the variant and all of its options, which is what we care about.
        format!("{:?}", entry.field_type()).hash(&mut hasher);
    }
    hasher.finish()
}

fn initialize_search(arena_size: usize) -> tantivy::Result<(Index, SearchFields)> {
    // We want to store our data someplace sane, so we're gonna use the directories library to
    // decide where all this data goes.
    let dirs = ProjectDirs::from("org", "Hack Commons", "Bible-App")
        .ok_or_else(|| io::Error::other("unable to initialize project directory"))?;

    // We'll need to ensure the directory exists. That's easy, but I'm not sure how to know if
    // there is an existing index in an existing directory. That seems important.

    let index_path = dirs.data_dir().join("bible_idx");
    if !index_path.exists() {
        std::fs::create_dir_all(&index_path)?;
    }

    let (schema, fields) = build_schema();
    let index_dir = MmapDirectory::open(&index_path)?;
    let sentinel = index_path.join("schema_version");
    let fingerprint = schema_fingerprint(&schema);

    if !tantivy::Index::exists(&index_dir)? {
        // No index on disk yet — build everything from scratch.
        return create_index(&index_path, schema, &fields, fingerprint, arena_size);
    }

    // An index exists. Rebuild unless the sentinel matches the current schema
    // fingerprint. A missing sentinel means the index predates versioning and
    // can't be trusted to be compatible; a mismatch means the schema has changed.
    let compatible = std::fs::read_to_string(&sentinel)
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        == Some(fingerprint);

    if !compatible {
        std::fs::remove_dir_all(&index_path)?;
        std::fs::create_dir_all(&index_path)?;
        return create_index(&index_path, schema, &fields, fingerprint, arena_size);
    }

    Ok((tantivy::Index::open(index_dir)?, fields))
}

fn create_index(
    index_path: &Path,
    schema: Schema,
    fields: &SearchFields,
    fingerprint: u64,
    arena_size: usize,
) -> tantivy::Result<(Index, SearchFields)> {
    let index = Index::create_in_dir(index_path, schema)?;

    // A single writer for both translations means a single commit and fewer
    // segments than two separate writer sessions.
    let mut writer = index.writer(arena_size)?;
    write_index(Translation::Kjv, fields, &mut writer)?;
    write_index(Translation::Asv, fields, &mut writer)?;
    writer.commit()?;

    // Stamp the index with the current schema fingerprint so future runs can
    // tell whether the on-disk format is still compatible.
    write_version(&index_path.join("schema_version"), fingerprint)?;

    Ok((index, *fields))
}

fn write_version(path: &Path, fingerprint: u64) -> tantivy::Result<()> {
    std::fs::write(path, fingerprint.to_string())?;
    Ok(())
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
    text.lines().filter_map(|line| {
        let id = line.get(..8)?.parse::<u64>().ok()?;
        Some((id, line.get(9..)?))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_size_units() {
        assert_eq!(100, ByteSize::from_str("100").unwrap().0);
        assert_eq!(100 * 1024, ByteSize::from_str("100KB").unwrap().0);
        assert_eq!(500 * 1024 * 1024, ByteSize::from_str("500MB").unwrap().0);
        assert_eq!(2 * 1024 * 1024 * 1024, ByteSize::from_str("2GB").unwrap().0);
    }

    #[test]
    fn byte_size_case_insensitive() {
        assert_eq!(
            ByteSize::from_str("500mb").unwrap().0,
            ByteSize::from_str("500MB").unwrap().0
        );
        assert_eq!(
            ByteSize::from_str("2g").unwrap().0,
            ByteSize::from_str("2GB").unwrap().0
        );
        assert_eq!(
            ByteSize::from_str("100k").unwrap().0,
            ByteSize::from_str("100KB").unwrap().0
        );
    }

    #[test]
    fn byte_size_invalid() {
        assert!(ByteSize::from_str("abc").is_err());
        assert!(ByteSize::from_str("").is_err());
        assert!(ByteSize::from_str("500TB").is_err());
        assert!(ByteSize::from_str("MB").is_err());
    }

    #[test]
    fn parse_verses_valid_lines() {
        let data = "01001001\tIn the beginning God created the heaven and the earth.\n\
                    01001002\tAnd the earth was without form, and void.";
        let verses: Vec<_> = parse_verses_with_id(data).collect();
        assert_eq!(verses.len(), 2);
        assert_eq!(verses[0].0, 1_001_001);
        assert_eq!(
            verses[0].1,
            "In the beginning God created the heaven and the earth."
        );
        assert_eq!(verses[1].0, 1_001_002);
    }

    #[test]
    fn parse_verses_skips_malformed() {
        // Lines that are too short or lack a tab are silently skipped.
        let data = "01001001\tValid line\n\
                    short\n\
                    \n\
                    01001004\tAlso valid";
        let verses: Vec<_> = parse_verses_with_id(data).collect();
        assert_eq!(verses.len(), 2);
        assert_eq!(verses[0].0, 1_001_001);
        assert_eq!(verses[1].0, 1_001_004);
    }
}
