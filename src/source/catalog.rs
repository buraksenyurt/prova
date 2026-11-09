//! Exams shipped with the app.
//!
//! `exams/catalog.json` lists them; each entry points to an exam file (exam
//! Markdown or quiz JSON) next to it. Trunk copies the folder into the build,
//! so adding an exam is a new file plus a catalog entry, no code change. Later
//! the catalog URL can point to a service instead.

use serde::Deserialize;

use super::{LoadError, LoadedQuiz, LocalFuture, Origin, QuizSource, fetch_text, from_text};
use crate::dto::SchemaError;

/// Relative to the page, so it works under any base path (Pages, release zip).
pub const CATALOG_URL: &str = "exams/catalog.json";

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct CatalogEntry {
    /// Must equal the quiz id inside the file (history is stored per id).
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub course: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    /// Path relative to the catalog file.
    pub file: String,
    #[serde(default)]
    pub questions: Option<usize>,
}

#[derive(Deserialize)]
struct CatalogFile {
    exams: Vec<CatalogEntry>,
}

pub fn parse_catalog(text: &str) -> Result<Vec<CatalogEntry>, SchemaError> {
    Ok(serde_json::from_str::<CatalogFile>(text)?.exams)
}

impl CatalogEntry {
    /// URL of the exam file, resolved against the catalog URL.
    pub fn url(&self, catalog_url: &str) -> String {
        if self.file.contains("://") || self.file.starts_with('/') {
            return self.file.clone();
        }
        let dir = catalog_url.rfind('/').map_or("", |i| &catalog_url[..=i]);
        format!("{dir}{}", self.file.trim_start_matches("./"))
    }
}

pub async fn fetch_catalog(catalog_url: &str) -> Result<Vec<CatalogEntry>, LoadError> {
    let text = fetch_text(catalog_url).await?;
    parse_catalog(&text).map_err(LoadError::Schema)
}

pub struct CatalogSource {
    pub entry: CatalogEntry,
    pub catalog_url: String,
}

impl QuizSource for CatalogSource {
    fn load(&self) -> LocalFuture<Result<LoadedQuiz, LoadError>> {
        let url = self.entry.url(&self.catalog_url);
        let title = self.entry.title.clone();
        Box::pin(async move {
            let text = fetch_text(&url).await?;
            from_text(text, Origin::Catalog { title, url })
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::convert::{ConvertOptions, exam_markdown_to_json};
    use crate::dto;

    #[test]
    fn entry_urls() {
        let e = CatalogEntry {
            id: "x".into(),
            title: "X".into(),
            course: None,
            description: None,
            file: "./a.md".into(),
            questions: None,
        };
        assert_eq!(e.url("exams/catalog.json"), "exams/a.md");
        assert_eq!(e.url("catalog.json"), "a.md");
    }

    /// Every shipped exam must load without warnings, with answers, and with
    /// the id and question count the catalog promises. Runs in CI on each push.
    #[test]
    fn shipped_exams_are_valid() {
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/exams/");
        let catalog =
            parse_catalog(&std::fs::read_to_string(format!("{root}catalog.json")).unwrap())
                .unwrap();
        assert!(!catalog.is_empty());
        let mut ids = HashSet::new();
        for entry in &catalog {
            assert!(ids.insert(entry.id.clone()), "duplicate id {}", entry.id);
            let text = std::fs::read_to_string(format!("{root}{}", entry.file))
                .unwrap_or_else(|e| panic!("{}: {e}", entry.file));
            let json = if entry.file.ends_with(".json") {
                text
            } else {
                let opts = ConvertOptions {
                    assets_base_url: Some("exams/".into()),
                    ..Default::default()
                };
                let c = exam_markdown_to_json(&text, &opts)
                    .unwrap_or_else(|e| panic!("{}: {e:?}", entry.file));
                assert!(c.warnings.is_empty(), "{}: {:?}", entry.file, c.warnings);
                c.json
            };
            let parsed = dto::parse(&json).unwrap_or_else(|e| panic!("{}: {e:?}", entry.file));
            assert_eq!(
                parsed.quiz.id, entry.id,
                "{}: id must match the catalog",
                entry.file
            );
            let key = parsed
                .key
                .unwrap_or_else(|| panic!("{}: no answers", entry.file));
            assert_eq!(key.len(), parsed.quiz.questions.len());
            if let Some(n) = entry.questions {
                assert_eq!(
                    parsed.quiz.questions.len(),
                    n,
                    "{}: question count",
                    entry.file
                );
            }
            // Images referenced with relative paths must exist in the folder.
            for q in &parsed.quiz.questions {
                for (i, _) in q.stem.match_indices("](./") {
                    let rest = &q.stem[i + 4..];
                    let path = &rest[..rest.find(')').unwrap()];
                    assert!(
                        std::path::Path::new(&format!("{root}{path}")).exists(),
                        "{}: missing {path}",
                        entry.file
                    );
                }
            }
        }
    }
}
