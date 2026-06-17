# AGENTS.md

Guidance for AI coding agents working on `fiat-lux`.

## What this is

`fiat-lux` ("let there be light") is an offline, terminal-accessible Bible
application written in idiomatic Rust (edition 2024). It can display passages
from public-domain translations (KJV, ASV) and perform full-text search over
them, with generated links to biblia.com for online reference.

## Commands

```sh
cargo run -- <book> <chapter>[:verse[-verse]]   # display passage, e.g. john 3:16
cargo run -- search "<query>"                   # full-text search (alias: s)
cargo run -- --asv <book> <loc>                 # select translation
cargo run -- <book> <loc> --limit N             # search result limit
cargo test                                      # run unit tests (only a few, in book.rs)
cargo clippy --all-targets --all-features       # lint
cargo fmt --check                               # formatting check
cargo build --release                           # release build (LTO + codegen-units=1)
```

Feature flags (in `Cargo.toml`):
- `pager` (default, on) — pipes long output through `bat` via the `pager` crate.
  Disable with `--no-default-features` for environments without a pager.

`.cargo/config.toml` sets `lld` as the linker and `target-cpu=native`; keep
these in mind if builds misbehave on a different machine.

## Module map (`src/`)

- `main.rs` — clap CLI (`Args`, `Command`), `Translation` enum (KJV=1, ASV=2),
  tantivy index bootstrap (`initialize_search`, `build_schema`, `write_index`),
  query construction (`search_by_book_and_location`, `search`), and output
  formatting (`format_texts`). Also contains the hidden `Austin` Easter egg.
- `book.rs` — 1-based `Book` enum (Genesis=1 .. Revelation=66) with
  `from_u8`, `FromStr` (handles `1 Kings`, `Kings1`, `1Kings`), and a
  `first_numeric_nonnumeric_transition` helper (tested).
- `location.rs` — `Location { book, chapter, verse }`,
  `PartialLocation { chapter, verse: Option<Verse> }`, and `Verse`
  (`start`/`end` range, `NonZero`-based). Parses `3:16` and `3:16-18`.
- `reference.rs` + `reference/biblia.rs` — `Reference` trait,
  `ReferenceLocator` (implemented for `Location`, `Chapter`, `&Text`),
  `ReferenceProvider` clap `ValueEnum` (currently only `Biblia`).
- `text.rs` — `Text` record with manual `Eq`/`Ord` (orders by book, chapter,
  verse) and `Chapter` key used for grouping output.
- `search.rs` — `SearchFields { translation, location, content }` tantivy
  field bundle.
- `error.rs` — top-level `Error` enum (io, tantivy variants) and the
  `AbbrevStr` trait used to truncate user input in error messages.

## Data encoding

Bible text lives in `resource/{asv,kjv}.dat` and is `include_str!`'d at
compile time. Each line is an 8-digit `u64` ID followed by a tab and the verse
text. IDs encode the location as `BBCCCVVV`:

- `id / 1_000_000` → book (1–66, matches `Book` discriminant)
- `id % 1_000_000 / 1000` → chapter
- `id % 1000` → verse

See `Location::from_id` and `parse_verses_with_id`.

## Tantivy index

- Schema (`build_schema`): `translation` (facet), `location` (facet), `content`
  (text, stored).
- Location facets are stored as `/{book}/{chapter}/{verse}`; translation facets
  as `/{KJV|ASV}`. Queries intersect the translation facet with either a
  location facet (for passage lookup) or a parsed content query (for search).
- The index is created on first run under
  `ProjectDirs::from("org", "Hack Commons", "Bible-App").data_dir()/bible_idx`
  and reopened thereafter. To force a rebuild, delete that directory. Index
  writers use a 500 MB arena.
- Schema versioning: a `schema_version` sentinel file alongside the index
  stores a fingerprint derived from the `Schema` object itself (see
  `schema_fingerprint` in `main.rs`). Any change to `build_schema` — adding,
  removing, renaming, or reconfiguring a field — automatically changes the
  fingerprint and triggers a rebuild on the next run; there is no manual
  version constant to bump. A missing sentinel (index predates versioning) or
  a mismatch also triggers a rebuild. For non-schema format changes (e.g.
  facet path encoding in `write_index`), bump `INDEX_FORMAT_SALT`.
- Note: `text.rs::from_document` parses the location facet back out by
  round-tripping the encoded string through `Facet::from_encoded` and calling
  `to_path()`, rather than depending on the internal NUL separator.

## Conventions

- Idiomatic Rust; the author writes fairly chatty code comments — match that
  tone in any code you add where it aids understanding.
- Errors flow through `error::Error`; user-facing strings are truncated via
  `AbbrevStr::get(N)` to keep messages short.
- `Book` is `#[repr(u8)]` and 1-based; `0` is implicitly "no book" and is
  relied on when `Args.book` is `Option<Book>`.
- Tests are sparse (one in `book.rs`). When adding parsing logic, prefer
  adding focused unit tests alongside.
- Do not commit changes unless explicitly asked. Do not update the
  `Cargo.lock` unless dependencies actually changed.

## Known gotchas

- The `Austin` subcommand is a deliberate Stone Cold Steve Austin / John 3:16
  parody — leave it alone unless asked.
- `SearchArgs` runs against a single translation (whichever is selected); the
  author has a TODO in `dispatch` noting this may want to change.
- Terminal width is capped at 100 columns in both single- and multi-verse
  output paths.
