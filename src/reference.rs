mod biblia;

pub use biblia::Biblia;

use crate::location::Location;
use crate::Translation;
use clap::ValueEnum;
use std::fmt;

pub trait Reference {
    fn url(&self, location: &Location, translation: Translation) -> String;
}

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum ReferenceProvider {
    #[default]
    Biblia,
}

impl ReferenceProvider {
    fn short_name(&self) -> &'static str {
        match self {
            Self::Biblia => "biblia",
        }
    }
}

impl fmt::Display for ReferenceProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.short_name())
    }
}

impl ReferenceProvider {
    pub fn get(&self) -> Box<dyn Reference> {
        match self {
            ReferenceProvider::Biblia => Box::new(Biblia),
        }
    }
}
