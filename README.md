# fiat-lux

Offline, terminal-accessible Bible.

Display passages and search the full text of public-domain Bible
translations (KJV, ASV) with generated links to biblia.com for online
reference.

## Installation

```sh
cargo install --path .
```

Or clone and build:

```sh
git clone https://github.com/archer884/fiat-lux.git
cd fiat-lux
cargo build --release
```

The binary will be at `target/release/fiat-lux`.

## Usage

```sh
fiat-lux john 3:16                  # display a single verse
fiat-lux john 3:16-18               # display a verse range
fiat-lux psalms 119                 # display a whole chapter
fiat-lux --asv john 3:16            # use the ASV translation (KJV is default)
fiat-lux search "light"             # full-text search (alias: s)
fiat-lux search "love" --limit 20   # show up to 20 results
fiat-lux create-index               # build the search index manually
fiat-lux create-index --force       # rebuild the index from scratch
fiat-lux --index-memory 2GB john 3  # override index writer memory budget
```

### Environment variables

- `FIAT_LUX_REFERENCE` — reference link provider (default: `biblia`)
- `FIAT_LUX_INDEX_MEMORY` — index writer memory budget (e.g. `2GB`)

## Translations

- **King James Version** (KJV) — default
- **American Standard Version** (ASV) — use `--asv`

## Features

- **Full-text search** powered by [tantivy](https://github.com/quickwit-oss/tantivy)
- **Passage lookup** by book, chapter, and verse range
- **Pager support** — long output is piped through
  [`bat`](https://github.com/sharkdp/bat) automatically
- **Book name parsing** — accepts `1 Kings`, `Kings1`, `1Kings`, and more
- **Verse ranges** — e.g. `3:16-18`
- **Offline** — all text is bundled in the binary at compile time

## Resources

Bible text (KJV, ASV) from https://github.com/scrollmapper/bible_databases
