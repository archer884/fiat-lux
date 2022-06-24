use std::io;

pub trait AbbrevStr: AsRef<str> + Into<String> {
    fn get(self, limit: usize) -> String {
        let full = self.as_ref();

        if full.len() > limit {
            full[..limit].to_string() + "..."
        } else {
            self.into()
        }
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
