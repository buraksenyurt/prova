//! Finished attempts of a quiz and how each topic develops over them.

use serde::{Deserialize, Serialize};

use super::attempt::{Attempt, Mode};
use super::summary::Summary;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub finished_at_ms: f64,
    pub mode: Mode,
    pub correct: usize,
    pub total: usize,
    pub duration_ms: f64,
    /// "Retry wrong and empty" run: only a subset, not comparable to full runs.
    #[serde(default)]
    pub retry: bool,
    /// Entries saved before topic tracking existed have none.
    #[serde(default)]
    pub topics: Vec<TopicResult>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TopicResult {
    pub topic: String,
    pub correct: usize,
    pub total: usize,
}

impl TopicResult {
    pub fn percent(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.correct as f64 * 100.0 / self.total as f64
        }
    }
}

impl HistoryEntry {
    pub fn from_summary(attempt: &Attempt, summary: &Summary, finished_at_ms: f64) -> Self {
        HistoryEntry {
            finished_at_ms,
            mode: attempt.mode,
            correct: summary.correct,
            total: summary.total,
            duration_ms: summary.duration_ms,
            retry: attempt.retry,
            topics: summary
                .topics
                .iter()
                .map(|t| TopicResult {
                    topic: t.topic.clone(),
                    correct: t.correct,
                    total: t.total,
                })
                .collect(),
        }
    }

    pub fn percent(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.correct as f64 * 100.0 / self.total as f64
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TopicTrend {
    pub topic: String,
    /// Percentages, oldest first.
    pub points: Vec<f64>,
    /// Latest minus the one before; `None` with a single point.
    pub change: Option<f64>,
}

impl TopicTrend {
    pub fn latest(&self) -> f64 {
        self.points.last().copied().unwrap_or(0.0)
    }
}

/// Per-topic percentages over the last `max_points` full attempts.
/// `history` is newest first (as stored). Weakest topics come first.
pub fn topic_trends(history: &[HistoryEntry], max_points: usize) -> Vec<TopicTrend> {
    let full: Vec<&HistoryEntry> = history
        .iter()
        .filter(|h| !h.retry && !h.topics.is_empty())
        .take(max_points)
        .collect();

    let mut trends: Vec<TopicTrend> = Vec::new();
    for entry in full.iter().rev() {
        for t in &entry.topics {
            match trends.iter_mut().find(|x| x.topic == t.topic) {
                Some(x) => x.points.push(t.percent()),
                None => trends.push(TopicTrend {
                    topic: t.topic.clone(),
                    points: vec![t.percent()],
                    change: None,
                }),
            }
        }
    }
    for t in &mut trends {
        let n = t.points.len();
        if n >= 2 {
            t.change = Some(t.points[n - 1] - t.points[n - 2]);
        }
    }
    trends.sort_by(|a, b| a.latest().total_cmp(&b.latest()));
    trends
}

/// The most recent full attempt finished before `before_ms`, to compare with.
pub fn previous_full(history: &[HistoryEntry], before_ms: f64) -> Option<&HistoryEntry> {
    history
        .iter()
        .find(|h| !h.retry && h.finished_at_ms < before_ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(at: f64, retry: bool, topics: &[(&str, usize, usize)]) -> HistoryEntry {
        HistoryEntry {
            finished_at_ms: at,
            mode: Mode::Exam,
            correct: 0,
            total: 0,
            duration_ms: 0.0,
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

    #[test]
    fn trends_skip_retries_and_order_oldest_first() {
        // newest first, as stored
        let h = vec![
            entry(4.0, false, &[("RAG", 3, 4), ("Test", 1, 2)]),
            entry(3.0, true, &[("RAG", 0, 1)]),
            entry(2.0, false, &[("RAG", 1, 4)]),
            entry(1.0, false, &[("Test", 2, 2)]),
        ];
        let t = topic_trends(&h, 5);
        assert_eq!(t[0].topic, "Test"); // weakest latest (50%) first
        assert_eq!(t[0].points, vec![100.0, 50.0]);
        assert_eq!(t[0].change, Some(-50.0));
        assert_eq!(t[1].points, vec![25.0, 75.0]);
        assert_eq!(t[1].change, Some(50.0));

        let limited = topic_trends(&h, 1);
        assert!(
            limited
                .iter()
                .all(|x| x.points.len() == 1 && x.change.is_none())
        );
    }

    #[test]
    fn old_entries_without_topics_still_parse() {
        let json =
            r#"[{"finished_at_ms":1.0,"mode":"Exam","correct":1,"total":2,"duration_ms":5.0}]"#;
        let h: Vec<HistoryEntry> = serde_json::from_str(json).unwrap();
        assert!(!h[0].retry && h[0].topics.is_empty());
        assert!(topic_trends(&h, 5).is_empty());
    }

    #[test]
    fn previous_full_attempt() {
        let h = vec![
            entry(3.0, false, &[]),
            entry(2.0, true, &[]),
            entry(1.0, false, &[]),
        ];
        assert_eq!(previous_full(&h, 3.0).unwrap().finished_at_ms, 1.0);
        assert!(previous_full(&h, 1.0).is_none());
    }
}
