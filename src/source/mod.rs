//! Where a quiz comes from.
//!
//! Phase 1: a file the student picked, a URL (`?quiz=...`), the built-in
//! sample, or the copy saved in the browser. A later `ApiSource` only has to
//! implement [`QuizSource`]; the UI talks to the trait, not to a source.

mod browser;

use std::future::Future;
use std::pin::Pin;

pub use browser::{FileSource, TextSource, UrlSource};

use crate::dto::{self, Parsed, SchemaError};

/// Futures in the browser are single-threaded, so no `Send` is required.
pub type LocalFuture<T> = Pin<Box<dyn Future<Output = T>>>;

/// Shown on the intro screen and stored with the saved copy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Origin {
    File(String),
    Url(String),
    Sample,
    /// The copy kept in `localStorage`; carries the original origin's label.
    Saved(String),
}

impl Origin {
    pub fn label(&self) -> String {
        match self {
            Origin::File(name) => name.clone(),
            Origin::Url(url) => url.clone(),
            Origin::Sample => "demo.json".into(),
            Origin::Saved(label) => label.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoadedQuiz {
    pub parsed: Parsed,
    pub origin: Origin,
    /// Identifies this exact content (see `dto::fingerprint`).
    pub fingerprint: u64,
    /// The original text, kept so the quiz can be restored after a refresh.
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoadError {
    Read(String),
    Network(String),
    Http(u16),
    Schema(SchemaError),
}

impl From<SchemaError> for LoadError {
    fn from(e: SchemaError) -> Self {
        LoadError::Schema(e)
    }
}

pub trait QuizSource {
    fn load(&self) -> LocalFuture<Result<LoadedQuiz, LoadError>>;
}

/// Shared by every source that ends up with JSON text.
pub fn from_text(raw: String, origin: Origin) -> Result<LoadedQuiz, LoadError> {
    // A UTF-8 BOM (Windows editors) is not valid JSON.
    let raw = raw
        .strip_prefix('\u{feff}')
        .map(str::to_owned)
        .unwrap_or(raw);
    let parsed = dto::parse(&raw)?;
    Ok(LoadedQuiz {
        parsed,
        origin,
        fingerprint: dto::fingerprint(&raw),
        raw,
    })
}
