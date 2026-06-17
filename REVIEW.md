# Code Review

Ordered by severity.

## Bugs

**2. `Verse::from_str` doesn't validate range ordering** — `location.rs:107`

`"5-3"` parses to `Verse { start: 5, end: Some(3) }`, and `contains` then matches nothing (since `verse >= 5 && verse <= 3` is always false). The user gets silently empty output instead of an error. Should reject `end < start`.

## UX gaps

**3. No "no results" feedback** — `main.rs:172,190`

Both the single-verse and multi-verse paths call `format_texts` / print directly. If a search or passage lookup returns zero verses (e.g. `fiat-lux enoch 1` or a search with no hits), the program prints nothing and exits 0. A "no verses found" message would help.

## Robustness

**4. `text.rs::from_document` relies on tantivy's internal facet encoding** — `text.rs:19`

```rust
let mut segments = facet.split('\0');
```

This depends on tantivy serializing facet segments as NUL-separated bytes rather than using the public `Facet::to_path()` API, which returns `Vec<&str>` directly. A tantivy version bump could change this silently. Prefer `facet.to_path()` and index into the returned `Vec`.

**5. `parse_verses_with_id` indexes by byte offset** — `main.rs:479`

```rust
line[..8].parse::<u64>()... &line[9..]
```

Assumes every line is ≥9 bytes with a tab at index 8. Safe today because the `.dat` files are bundled at compile time, but a malformed line would panic. Low risk given controlled data, but a `.filter(|l| l.len() >= 9)` guard or `split_once('\t')` would be more robust.

**6. No index schema versioning** — `main.rs:404`

`initialize_search` checks `Index::exists` but not whether the on-disk schema matches `build_schema()`. If the schema ever changes, reopening a stale index will fail at query time with a confusing tantivy error rather than rebuilding. A version sentinel (e.g. a `meta.json` in the index dir) would make future migrations clean.

## Minor / style

**7. Duplicated width logic** — `main.rs:197-214`

The `#[cfg(feature = "pager")]` and `#[cfg(not(...))]` branches duplicate the `terminal_size` call. The only real difference is the pager-setup side effect. Could be collapsed into one `let (w, h) = ...` followed by a `#[cfg]`-gated side-effect block.

**8. Hidden side effect in a width-binding block** — `main.rs:198-206`

`pager::Pager::setup()` (which redirects stdout) happens inside `let width = { ... }`, making the side effect easy to miss. Worth lifting out as an explicit statement.

**9. Two separate `impl ReferenceProvider` blocks** — `reference.rs:70` and `reference.rs:84` could be merged.

**10. `Text` manual `Eq`/`Ord` is correct but slightly surprising** — `text.rs:48-69`. Two texts with matching book/chapter/verse but different content compare equal. This is intentional (content differs only across translations, never within one query) and is fine — just worth a one-line comment explaining *why* it's hand-rolled instead of derived.

## Strengths

- Clean module boundaries; domain types (`Book`, `Location`, `Verse`) are well-factored.
- The facet-based translation/location filtering in tantivy is a tidy design.
- `AbbrevStr` is a nice UX touch (modulo the byte-slice bug).
- `Verse::contains` correctly handles both single verses and ranges with `NonZero`.
- Error flow through `thiserror` + `Error` is idiomatic.

---

The highest-impact fix is **#1** (the panic) — it's directly reachable from CLI input.
