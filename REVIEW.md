# Code Review

Ordered by severity.

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
