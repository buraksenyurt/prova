//! Where a quiz comes from.
//!
//! A file the student picked, a URL (`?quiz=...`), the built-in sample, or
//! the copy saved in the browser. JSON quiz files and exam Markdown are both
//! accepted; Markdown is converted to JSON first (see `convert`). A later `ApiSource` only has to
//! implement [`QuizSource`]; the UI talks to the trait, not to a source.

mod browser;
pub mod catalog;

use std::future::Future;
use std::pin::Pin;

pub use browser::{FileSource, TextSource, UrlSource, fetch_text};
pub use catalog::{CatalogEntry, CatalogSource};

use crate::convert::{self, ConvertError, ConvertOptions, Warning};
use crate::dto::{self, Parsed, SchemaError};

/// Futures in the browser are single-threaded, so no `Send` is required.
pub type LocalFuture<T> = Pin<Box<dyn Future<Output = T>>>;

/// Shown on the intro screen and stored with the saved copy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Origin {
    File(String),
    Url(String),
    Sample,
    /// An exam shipped with the app (`exams/catalog.json`).
    Catalog {
        /// Catalog id; becomes the quiz id so history matches the catalog.
        id: String,
        title: String,
        url: String,
    },
    /// The copy kept in `localStorage`; carries the original origin's label.
    Saved(String),
}

impl Origin {
    pub fn label(&self) -> String {
        match self {
            Origin::File(name) => name.clone(),
            Origin::Url(url) => url.clone(),
            Origin::Sample => "demo.json".into(),
            Origin::Catalog { title, .. } => title.clone(),
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
    /// The quiz as JSON (converted if the source was Markdown); kept so the
    /// quiz can be restored after a refresh or downloaded.
    pub raw: String,
    pub from_markdown: bool,
    pub warnings: Vec<Warning>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoadError {
    Read(String),
    Network(String),
    Http(u16),
    Schema(SchemaError),
    Convert(ConvertError),
}

impl From<SchemaError> for LoadError {
    fn from(e: SchemaError) -> Self {
        LoadError::Schema(e)
    }
}

pub trait QuizSource {
    fn load(&self) -> LocalFuture<Result<LoadedQuiz, LoadError>>;
}

/// Shared by every source that ends up with text: JSON, or exam Markdown.
pub fn from_text(raw: String, origin: Origin) -> Result<LoadedQuiz, LoadError> {
    // A UTF-8 BOM (Windows editors) is not valid JSON.
    let raw = raw
        .strip_prefix('\u{feff}')
        .map(str::to_owned)
        .unwrap_or(raw);
    let (raw, from_markdown, warnings) = if raw.trim_start().starts_with('{') {
        (raw, false, Vec::new())
    } else {
        let converted = convert::exam_markdown_to_json(&raw, &markdown_options(&origin))
            .map_err(LoadError::Convert)?;
        (converted.json, true, converted.warnings)
    };
    let parsed = dto::parse(&raw)?;
    Ok(LoadedQuiz {
        parsed,
        origin,
        fingerprint: dto::fingerprint(&raw),
        raw,
        from_markdown,
        warnings,
    })
}

/// Images next to a Markdown file on the web resolve against its folder.
fn markdown_options(origin: &Origin) -> ConvertOptions {
    let mut opts = ConvertOptions::default();
    match origin {
        Origin::Url(url) | Origin::Catalog { url, .. } => {
            let path = url.split(['?', '#']).next().unwrap_or(url);
            if let Some(i) = path.rfind('/') {
                opts.assets_base_url = Some(path[..=i].to_string());
            }
            if let Origin::Catalog { id, .. } = origin {
                opts.id = Some(id.clone());
            }
        }
        Origin::File(name) => {
            let stem = name.rsplit_once('.').map_or(name.as_str(), |(s, _)| s);
            opts.default_id = Some(stem.to_string()).filter(|s| !s.is_empty());
        }
        _ => {}
    }
    opts
}

/// `github.com/<u>/<r>/blob/<branch>/<path>` → `raw.githubusercontent.com/<u>/<r>/<branch>/<path>`
/// so students can paste the link they see in the browser.
pub fn raw_github_url(url: &str) -> String {
    let rest = url
        .strip_prefix("https://github.com/")
        .or_else(|| url.strip_prefix("http://github.com/"));
    if let Some(rest) = rest {
        let parts: Vec<&str> = rest.splitn(4, '/').collect();
        if parts.len() == 4 && parts[2] == "blob" {
            let path = parts[3].split(['?', '#']).next().unwrap_or(parts[3]);
            return format!(
                "https://raw.githubusercontent.com/{}/{}/{}",
                parts[0], parts[1], path
            );
        }
    }
    url.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn github_blob_links_become_raw() {
        assert_eq!(
            raw_github_url("https://github.com/u/r/blob/main/dir/Exam00.md"),
            "https://raw.githubusercontent.com/u/r/main/dir/Exam00.md"
        );
        assert_eq!(raw_github_url("https://x.io/a.json"), "https://x.io/a.json");
    }

    #[test]
    fn markdown_is_converted_with_base_url() {
        let md = "# T\n## Soru 1\n![a](img/a.png)\n- A) a\n- B) b\n## Cevap Anahtarı\n| 1 | B |\n";
        let l = from_text(
            md.into(),
            Origin::Url("https://x.io/exams/e.md?raw=1".into()),
        )
        .unwrap();
        assert!(l.from_markdown);
        assert!(l.warnings.is_empty());
        assert_eq!(
            l.parsed.quiz.assets_base_url.as_deref(),
            Some("https://x.io/exams/")
        );
        assert!(l.raw.trim_start().starts_with('{'));
        // The stored JSON loads again as JSON.
        let again = from_text(l.raw.clone(), Origin::Saved("e.md".into())).unwrap();
        assert!(!again.from_markdown);
        assert_eq!(again.fingerprint, l.fingerprint);
    }

    #[test]
    fn file_name_becomes_id() {
        let md = "## Soru 1\nS\n- A) a\n- B) b\n";
        let l = from_text(md.into(), Origin::File("Exam 00.md".into())).unwrap();
        assert_eq!(l.parsed.quiz.id, "exam-00");
    }
}
