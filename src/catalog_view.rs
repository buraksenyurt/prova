//! What the home screen shows from the catalog: each exam's status for this
//! student, and filtering, searching and sorting of the list.
//!
//! Pure logic (history and the unfinished attempt are passed in), so it is
//! covered by native tests like the domain.

use serde::{Deserialize, Serialize};

use crate::convert::slug;
use crate::domain::{HistoryEntry, topic_trends};
use crate::source::catalog::{CatalogEntry, Level};

/// Below this the exam counts as "needs work".
pub const PASS_PERCENT: f64 = 70.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExamStatus {
    NotStarted,
    InProgress {
        answered: usize,
        total: usize,
    },
    /// At least one finished full attempt; `last` is its score.
    Completed {
        last: f64,
    },
    /// Only "retry wrong ones" runs so far, no full attempt to score.
    Practiced,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExamCard {
    pub entry: CatalogEntry,
    pub status: ExamStatus,
    pub attempts: usize,
    pub best: Option<f64>,
    pub last_at: Option<f64>,
    /// Course identifier used by tabs and `?course=` links.
    pub course_key: String,
    /// Position in the catalog, the default order.
    pub order: usize,
}

/// The unfinished attempt stored in the browser (only one exists at a time).
#[derive(Debug, Clone, PartialEq)]
pub struct InProgress {
    pub quiz_id: String,
    pub answered: usize,
    pub total: usize,
}

pub fn course_key(entry: &CatalogEntry) -> String {
    entry
        .course
        .as_deref()
        .map(slug)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "other".into())
}

pub fn build_cards(
    entries: &[CatalogEntry],
    history: impl Fn(&str) -> Vec<HistoryEntry>,
    in_progress: Option<&InProgress>,
) -> Vec<ExamCard> {
    entries
        .iter()
        .enumerate()
        .map(|(order, entry)| {
            let h = history(&entry.id);
            let last_full = h.iter().find(|e| !e.retry);
            let status = match in_progress.filter(|p| p.quiz_id == entry.id) {
                Some(p) => ExamStatus::InProgress {
                    answered: p.answered,
                    total: p.total,
                },
                None => match (last_full, h.is_empty()) {
                    (Some(e), _) => ExamStatus::Completed { last: e.percent() },
                    (None, true) => ExamStatus::NotStarted,
                    (None, false) => ExamStatus::Practiced,
                },
            };
            ExamCard {
                course_key: course_key(entry),
                entry: entry.clone(),
                status,
                attempts: h.len(),
                best: h
                    .iter()
                    .filter(|e| !e.retry)
                    .map(HistoryEntry::percent)
                    .reduce(f64::max),
                last_at: h.first().map(|e| e.finished_at_ms),
                order,
            }
        })
        .collect()
}

/// Courses in catalog order with their exam counts: `(key, name, count)`.
pub fn courses(cards: &[ExamCard]) -> Vec<(String, String, usize)> {
    let mut out: Vec<(String, String, usize)> = Vec::new();
    for c in cards {
        match out.iter_mut().find(|(k, _, _)| *k == c.course_key) {
            Some(x) => x.2 += 1,
            None => out.push((
                c.course_key.clone(),
                c.entry.course.clone().unwrap_or_default(),
                1,
            )),
        }
    }
    out
}

/// All tags in catalog order, without duplicates.
pub fn tags(cards: &[ExamCard]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for t in cards.iter().flat_map(|c| c.entry.tags.iter()) {
        if !out.iter().any(|x| x == t) {
            out.push(t.clone());
        }
    }
    out
}

pub fn has_levels(cards: &[ExamCard]) -> bool {
    cards.iter().any(|c| c.entry.level().is_some())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum StatusFilter {
    #[default]
    All,
    NotStarted,
    InProgress,
    Completed,
    BelowPass,
}

impl StatusFilter {
    pub const ALL: [StatusFilter; 5] = [
        StatusFilter::All,
        StatusFilter::NotStarted,
        StatusFilter::InProgress,
        StatusFilter::Completed,
        StatusFilter::BelowPass,
    ];

    fn accepts(self, status: ExamStatus) -> bool {
        match self {
            StatusFilter::All => true,
            StatusFilter::NotStarted => status == ExamStatus::NotStarted,
            StatusFilter::InProgress => matches!(status, ExamStatus::InProgress { .. }),
            StatusFilter::Completed => matches!(status, ExamStatus::Completed { .. }),
            StatusFilter::BelowPass => {
                matches!(status, ExamStatus::Completed { last } if last < PASS_PERCENT)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SortKey {
    #[default]
    Recommended,
    Title,
    Questions,
}

impl SortKey {
    pub const ALL: [SortKey; 3] = [SortKey::Recommended, SortKey::Title, SortKey::Questions];
}

/// The student's selection. Everything except the search text is remembered.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ViewState {
    /// `None`: all courses.
    pub course: Option<String>,
    #[serde(skip)]
    pub query: String,
    pub status: StatusFilter,
    pub tag: Option<String>,
    #[serde(default)]
    pub level: Option<String>,
    pub sort: SortKey,
}

impl ViewState {
    /// True when anything narrows the list (the course tab does not count).
    pub fn is_filtered(&self) -> bool {
        !self.query.trim().is_empty()
            || self.status != StatusFilter::All
            || self.tag.is_some()
            || self.level.is_some()
    }

    pub fn clear_filters(&mut self) {
        self.query.clear();
        self.status = StatusFilter::All;
        self.tag = None;
        self.level = None;
    }

    /// Drops selections that no longer exist in the catalog.
    pub fn sanitize(&mut self, cards: &[ExamCard]) {
        if let Some(c) = &self.course
            && !cards.iter().any(|x| &x.course_key == c)
        {
            self.course = None;
        }
        if let Some(t) = &self.tag
            && !cards.iter().any(|x| x.entry.tags.contains(t))
        {
            self.tag = None;
        }
        if self.level.as_deref().and_then(Level::from_key).is_none() {
            self.level = None;
        }
    }
}

/// Lowercase without diacritics, so "cozum" finds "Çözüm" and "DI" finds "di".
pub fn fold(s: &str) -> String {
    s.chars()
        .flat_map(|c| match c {
            'İ' | 'I' | 'ı' => vec!['i'],
            'Ç' | 'ç' => vec!['c'],
            'Ğ' | 'ğ' => vec!['g'],
            'Ö' | 'ö' => vec!['o'],
            'Ş' | 'ş' => vec!['s'],
            'Ü' | 'ü' => vec!['u'],
            'Â' | 'â' => vec!['a'],
            'Î' | 'î' => vec!['i'],
            'Û' | 'û' => vec!['u'],
            c => c.to_lowercase().collect(),
        })
        .collect()
}

fn matches_query(card: &ExamCard, query: &str) -> bool {
    let words: Vec<String> = query.split_whitespace().map(fold).collect();
    if words.is_empty() {
        return true;
    }
    let e = &card.entry;
    let haystack = fold(&format!(
        "{} {} {} {} {}",
        e.title,
        e.description.as_deref().unwrap_or(""),
        e.course.as_deref().unwrap_or(""),
        e.tags.join(" "),
        e.id
    ));
    words.iter().all(|w| haystack.contains(w.as_str()))
}

/// Topics below the pass mark in the latest full attempt of any exam.
pub fn weak_topics(histories: &[Vec<HistoryEntry>]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for h in histories {
        for t in topic_trends(h, 1) {
            if t.latest() < PASS_PERCENT && !out.contains(&t.topic) {
                out.push(t.topic);
            }
        }
    }
    out
}

/// "What should I solve next?": unfinished first, then new exams, then those
/// below the pass mark (weakest first), then the rest. Within a group, exams
/// covering more of the student's weak topics come first.
fn recommended_rank(card: &ExamCard, weak: &[String]) -> (u8, i64, i64) {
    let (group, score) = match card.status {
        ExamStatus::InProgress { .. } => (0, 0.0),
        ExamStatus::NotStarted => (1, 0.0),
        ExamStatus::Practiced => (2, 0.0),
        ExamStatus::Completed { last } if last < PASS_PERCENT => (2, last),
        ExamStatus::Completed { last } => (3, last),
    };
    let overlap = card.entry.tags.iter().filter(|t| weak.contains(t)).count() as i64;
    (group, -overlap, score.round() as i64)
}

pub fn apply(cards: &[ExamCard], state: &ViewState, weak: &[String]) -> Vec<ExamCard> {
    let level = state.level.as_deref().and_then(Level::from_key);
    let mut out: Vec<ExamCard> = cards
        .iter()
        .filter(|c| state.course.as_ref().is_none_or(|k| &c.course_key == k))
        .filter(|c| state.status.accepts(c.status))
        .filter(|c| state.tag.as_ref().is_none_or(|t| c.entry.tags.contains(t)))
        .filter(|c| level.is_none_or(|l| c.entry.level() == Some(l)))
        .filter(|c| matches_query(c, &state.query))
        .cloned()
        .collect();
    match state.sort {
        SortKey::Recommended => {
            out.sort_by_key(|c| (recommended_rank(c, weak), c.order));
        }
        SortKey::Title => out.sort_by_key(|a| fold(&a.entry.title)),
        SortKey::Questions => out.sort_by_key(|c| (c.entry.questions.unwrap_or(0), c.order)),
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Mode, TopicResult};

    fn entry(
        id: &str,
        course: &str,
        title: &str,
        tags: &[&str],
        level: Option<&str>,
        q: usize,
    ) -> CatalogEntry {
        CatalogEntry {
            id: id.into(),
            title: title.into(),
            course: Some(course.into()),
            description: Some(format!("{title} açıklaması")),
            file: format!("{id}.md"),
            questions: Some(q),
            level: level.map(Into::into),
            minutes: None,
            tags: tags.iter().map(|t| (*t).into()).collect(),
        }
    }

    fn run(
        at: f64,
        correct: usize,
        total: usize,
        retry: bool,
        topics: &[(&str, usize, usize)],
    ) -> HistoryEntry {
        HistoryEntry {
            finished_at_ms: at,
            mode: Mode::Exam,
            correct,
            total,
            duration_ms: 1.0,
            retry,
            topics: topics
                .iter()
                .map(|(t, c, n)| TopicResult {
                    topic: (*t).into(),
                    correct: *c,
                    total: *n,
                })
                .collect(),
        }
    }

    fn catalog() -> Vec<CatalogEntry> {
        vec![
            entry(
                "ai-1",
                "Yapay Zeka Destekli Yazılım Geliştirme",
                "Final Soruları",
                &["RAG", "MCP"],
                Some("advanced"),
                27,
            ),
            entry(
                "ai-2",
                "Yapay Zeka Destekli Yazılım Geliştirme",
                "Senaryo Soruları",
                &["RAG"],
                Some("intermediate"),
                10,
            ),
            entry(
                "oop-0",
                "OOP in Enterprise",
                "OOP Temelleri",
                &["C#"],
                Some("beginner"),
                12,
            ),
            entry(
                "oop-3",
                "OOP in Enterprise",
                "Dependency Injection",
                &["C#", "DI"],
                None,
                5,
            ),
        ]
    }

    fn cards() -> Vec<ExamCard> {
        let history = |id: &str| match id {
            "ai-1" => vec![run(3.0, 6, 10, false, &[("RAG", 1, 4), ("MCP", 3, 3)])],
            "oop-0" => vec![
                run(5.0, 2, 2, true, &[]),
                run(4.0, 11, 12, false, &[("C#", 11, 12)]),
            ],
            "oop-3" => vec![run(2.0, 1, 1, true, &[])],
            _ => vec![],
        };
        let wip = InProgress {
            quiz_id: "ai-2".into(),
            answered: 4,
            total: 10,
        };
        build_cards(&catalog(), history, Some(&wip))
    }

    fn ids(v: &[ExamCard]) -> Vec<&str> {
        v.iter().map(|c| c.entry.id.as_str()).collect()
    }

    #[test]
    fn statuses() {
        let c = cards();
        assert_eq!(c[0].status, ExamStatus::Completed { last: 60.0 });
        assert_eq!(
            c[1].status,
            ExamStatus::InProgress {
                answered: 4,
                total: 10
            }
        );
        // the latest full run counts, not the newer retry
        assert!(matches!(c[2].status, ExamStatus::Completed { last } if last > 91.0));
        assert_eq!((c[2].attempts, c[2].last_at), (2, Some(5.0)));
        assert_eq!(c[3].status, ExamStatus::Practiced);
        assert_eq!(c[3].best, None);
    }

    #[test]
    fn courses_and_tags() {
        let c = cards();
        let courses = courses(&c);
        assert_eq!(courses[0].0, "yapay-zeka-destekli-yazilim-gelistirme");
        assert_eq!(
            (courses[1].0.as_str(), courses[1].2),
            ("oop-in-enterprise", 2)
        );
        assert_eq!(tags(&c), vec!["RAG", "MCP", "C#", "DI"]);
        assert!(has_levels(&c));
    }

    #[test]
    fn filters() {
        let c = cards();
        let mut s = ViewState {
            course: Some("oop-in-enterprise".into()),
            ..Default::default()
        };
        assert_eq!(ids(&apply(&c, &s, &[])), vec!["oop-3", "oop-0"]);
        s.course = None;
        s.status = StatusFilter::BelowPass;
        assert_eq!(ids(&apply(&c, &s, &[])), vec!["ai-1"]);
        s.status = StatusFilter::All;
        s.tag = Some("RAG".into());
        s.level = Some("advanced".into());
        assert_eq!(ids(&apply(&c, &s, &[])), vec!["ai-1"]);
        assert!(s.is_filtered());
        s.clear_filters();
        assert!(!s.is_filtered());
    }

    #[test]
    fn search_folds_turkish_characters() {
        let c = cards();
        let s = |q: &str| ViewState {
            query: q.into(),
            ..Default::default()
        };
        assert_eq!(ids(&apply(&c, &s("senaryo"), &[])), vec!["ai-2"]);
        assert_eq!(ids(&apply(&c, &s("SORULARI final"), &[])), vec!["ai-1"]);
        assert_eq!(ids(&apply(&c, &s("yazilim di"), &[])).len(), 0);
        assert_eq!(ids(&apply(&c, &s("dependency"), &[])), vec!["oop-3"]);
        assert_eq!(fold("İŞLEM Çözüm"), "islem cozum");
    }

    #[test]
    fn recommended_order() {
        let c = cards();
        let weak = weak_topics(&[vec![run(
            3.0,
            6,
            10,
            false,
            &[("RAG", 1, 4), ("MCP", 3, 3)],
        )]]);
        assert_eq!(weak, vec!["RAG"]);
        let s = ViewState::default();
        // in progress, then not started (none here), then practiced / below pass, then passed
        assert_eq!(
            ids(&apply(&c, &s, &weak)),
            vec!["ai-2", "ai-1", "oop-3", "oop-0"]
        );
        let by_q = ViewState {
            sort: SortKey::Questions,
            ..Default::default()
        };
        assert_eq!(
            ids(&apply(&c, &by_q, &weak)),
            vec!["oop-3", "ai-2", "oop-0", "ai-1"]
        );
        let by_t = ViewState {
            sort: SortKey::Title,
            ..Default::default()
        };
        assert_eq!(ids(&apply(&c, &by_t, &weak))[0], "oop-3");
    }

    #[test]
    fn view_state_persistence_and_sanitize() {
        let c = cards();
        let mut s = ViewState {
            course: Some("gone".into()),
            query: "not saved".into(),
            tag: Some("RAG".into()),
            level: Some("expert".into()),
            ..Default::default()
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: ViewState = serde_json::from_str(&json).unwrap();
        assert!(back.query.is_empty());
        s.sanitize(&c);
        assert_eq!(
            (s.course, s.tag.as_deref(), s.level),
            (None, Some("RAG"), None)
        );
    }
}
