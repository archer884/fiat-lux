use std::io;

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub trait AbbrevStr: AsRef<str> + Into<String> {
    fn get(self, limit: usize) -> String {
        let full = self.as_ref();

        // Truncate by display width (terminal columns) rather than by byte count.
        // Slicing on a raw byte index risks landing inside a multibyte codepoint
        // and panicking, and columns are what we actually care about keeping
        // short for error messages anyway.
        if UnicodeWidthStr::width(full) <= limit {
            return self.into();
        }

        // Walk codepoint-by-codepoint, banking width until we overshoot the
        // budget; `char_indices` always yields safe slice boundaries.
        let mut width = 0;
        for (idx, ch) in full.char_indices() {
            width += UnicodeWidthChar::width(ch).unwrap_or(0);
            if width > limit {
                return full[..idx].to_string() + "...";
            }
        }

        // Defensive fallback: if per-character and whole-string width ever
        // disagree, prefer the full string over panicking.
        self.into()
    }
}

impl<T: AsRef<str> + Into<String>> AbbrevStr for T {}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] io::Error),

    #[error(transparent)]
    Tantivy(#[from] tantivy::error::TantivyError),

    #[error(transparent)]
    TantivyDir(#[from] tantivy::directory::error::OpenDirectoryError),

    #[error(transparent)]
    TantivyRead(#[from] tantivy::directory::error::OpenReadError),

    #[error(transparent)]
    TantivyQuery(#[from] tantivy::query::QueryParserError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abbrev_str_keeps_short_ascii_intact() {
        assert_eq!("genesis".to_string().get(20), "genesis");
    }

    #[test]
    fn abbrev_str_truncates_long_ascii_by_width() {
        assert_eq!("hello world".to_string().get(5), "hello...");
    }

    #[test]
    fn abbrev_str_budgets_by_columns_not_codepoints() {
        // Each CJK character occupies two terminal columns. With a width budget
        // of three, only the first character (width two) fits — byte-slicing
        // here would have panicked at byte 3 (mid-codepoint).
        assert_eq!("中文测试".to_string().get(3), "中...");
    }

    #[test]
    fn abbrev_str_handles_empty_input() {
        assert_eq!("".to_string().get(5), "");
    }
}
