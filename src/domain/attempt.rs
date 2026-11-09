//! One run through a quiz: order, answers, flags, verdicts and timing.
//!
//! Pure state, no browser APIs: time is passed in as milliseconds, so every
//! rule here is covered by native tests. The whole struct is serializable and
//! is stored as is, so a refreshed page continues where it left off.

use serde::{Deserialize, Serialize};

use super::quiz::{Quiz, Verdict};
use super::rng::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    /// Feedback and explanation right after each answer.
    Practice,
    /// Timer, answers can be changed, results at the end.
    Exam,
}

/// A question at one position of the attempt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Slot {
    /// Index into `quiz.questions`.
    pub question: usize,
    /// Display order: `choice_order[k]` is the index of the k-th shown choice.
    pub choice_order: Vec<usize>,
    /// Index into `question.choices` (not the display position).
    pub answer: Option<usize>,
    pub flagged: bool,
    /// Choices the student struck through.
    pub eliminated: Vec<usize>,
    pub verdict: Option<Verdict>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attempt {
    pub quiz_id: String,
    /// Fingerprint of the quiz content; a changed quiz cannot resume an old attempt.
    pub fingerprint: u64,
    pub mode: Mode,
    pub slots: Vec<Slot>,
    pub current: usize,
    pub started_at_ms: f64,
    pub deadline_ms: Option<f64>,
    pub finished_at_ms: Option<f64>,
    /// Built from a subset ("retry wrong and empty").
    #[serde(default)]
    pub retry: bool,
}

pub struct AttemptOptions {
    pub mode: Mode,
    pub shuffle_questions: bool,
    pub shuffle_choices: bool,
    /// Only used in exam mode.
    pub time_limit_minutes: Option<u32>,
    pub seed: u64,
    pub now_ms: f64,
    /// Indices into `quiz.questions`; `None` means all of them.
    pub subset: Option<Vec<usize>>,
}

impl Attempt {
    pub fn new(quiz: &Quiz, fingerprint: u64, opts: AttemptOptions) -> Self {
        let mut rng = Rng::new(opts.seed);
        let retry = opts.subset.is_some();
        let mut order: Vec<usize> = opts
            .subset
            .unwrap_or_else(|| (0..quiz.questions.len()).collect());
        if opts.shuffle_questions {
            rng.shuffle(&mut order);
        }

        let slots = order
            .into_iter()
            .map(|qi| {
                let q = &quiz.questions[qi];
                let mut choice_order: Vec<usize> = (0..q.choices.len()).collect();
                if opts.shuffle_choices && !q.lock_choices {
                    rng.shuffle(&mut choice_order);
                }
                Slot {
                    question: qi,
                    choice_order,
                    answer: None,
                    flagged: false,
                    eliminated: Vec::new(),
                    verdict: None,
                }
            })
            .collect();

        let deadline_ms = match (opts.mode, opts.time_limit_minutes) {
            (Mode::Exam, Some(m)) if m > 0 => Some(opts.now_ms + m as f64 * 60_000.0),
            _ => None,
        };

        Attempt {
            quiz_id: quiz.id.clone(),
            fingerprint,
            mode: opts.mode,
            slots,
            current: 0,
            started_at_ms: opts.now_ms,
            deadline_ms,
            finished_at_ms: None,
            retry,
        }
    }

    pub fn len(&self) -> usize {
        self.slots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    pub fn is_finished(&self) -> bool {
        self.finished_at_ms.is_some()
    }

    pub fn current_slot(&self) -> &Slot {
        &self.slots[self.current]
    }

    /// Answer can no longer change: the attempt is over, or practice mode
    /// already showed the result of this question.
    pub fn is_locked(&self, pos: usize) -> bool {
        self.is_finished() || (self.mode == Mode::Practice && self.slots[pos].answer.is_some())
    }

    /// Returns true if the answer changed.
    pub fn select(&mut self, pos: usize, choice: usize) -> bool {
        if self.is_locked(pos) || choice >= self.slots[pos].choice_order.len() {
            return false;
        }
        let slot = &mut self.slots[pos];
        if slot.answer == Some(choice) {
            return false;
        }
        slot.answer = Some(choice);
        slot.eliminated.retain(|&c| c != choice);
        true
    }

    /// Exam mode only: leave the question empty again.
    pub fn clear(&mut self, pos: usize) {
        if !self.is_locked(pos) {
            self.slots[pos].answer = None;
        }
    }

    pub fn toggle_flag(&mut self, pos: usize) {
        if !self.is_finished() {
            self.slots[pos].flagged = !self.slots[pos].flagged;
        }
    }

    /// Strike a choice through (or undo). The selected answer cannot be struck.
    pub fn toggle_eliminated(&mut self, pos: usize, choice: usize) {
        if self.is_locked(pos) || self.slots[pos].answer == Some(choice) {
            return;
        }
        let e = &mut self.slots[pos].eliminated;
        match e.iter().position(|&c| c == choice) {
            Some(i) => {
                e.remove(i);
            }
            None => e.push(choice),
        }
    }

    pub fn go_to(&mut self, pos: usize) {
        if pos < self.slots.len() {
            self.current = pos;
        }
    }

    pub fn next(&mut self) {
        self.go_to(self.current + 1);
    }

    pub fn prev(&mut self) {
        if self.current > 0 {
            self.current -= 1;
        }
    }

    pub fn set_verdict(&mut self, pos: usize, verdict: Verdict) {
        if let Some(slot) = self.slots.get_mut(pos) {
            slot.verdict = Some(verdict);
        }
    }

    pub fn finish(&mut self, now_ms: f64) {
        if self.finished_at_ms.is_none() {
            // A timed-out exam ends at its deadline, even if noticed later.
            let end = self.deadline_ms.map_or(now_ms, |d| now_ms.min(d));
            self.finished_at_ms = Some(end);
        }
    }

    pub fn answered(&self) -> usize {
        self.slots.iter().filter(|s| s.answer.is_some()).count()
    }

    pub fn unanswered(&self) -> usize {
        self.len() - self.answered()
    }

    /// Positions that still need a verdict.
    pub fn ungraded(&self) -> Vec<usize> {
        (0..self.len())
            .filter(|&p| self.slots[p].verdict.is_none())
            .collect()
    }

    pub fn remaining_ms(&self, now_ms: f64) -> Option<f64> {
        self.deadline_ms.map(|d| (d - now_ms).max(0.0))
    }

    pub fn is_timed_out(&self, now_ms: f64) -> bool {
        self.remaining_ms(now_ms) == Some(0.0)
    }

    pub fn elapsed_ms(&self, now_ms: f64) -> f64 {
        (self.finished_at_ms.unwrap_or(now_ms) - self.started_at_ms).max(0.0)
    }

    /// `None` while the question has no verdict yet.
    pub fn outcome(&self, quiz: &Quiz, pos: usize) -> Option<Outcome> {
        let slot = &self.slots[pos];
        let verdict = slot.verdict.as_ref()?;
        Some(match slot.answer {
            None => Outcome::Empty,
            Some(a) if quiz.questions[slot.question].choices[a].id == verdict.correct_choice => {
                Outcome::Correct
            }
            Some(_) => Outcome::Wrong,
        })
    }

    /// Questions (indices into `quiz.questions`) answered wrong or left empty.
    pub fn missed_questions(&self, quiz: &Quiz) -> Vec<usize> {
        (0..self.len())
            .filter(|&p| matches!(self.outcome(quiz, p), Some(Outcome::Wrong | Outcome::Empty)))
            .map(|p| self.slots[p].question)
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Correct,
    Wrong,
    Empty,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::domain::quiz::{Choice, ChoiceId, Question, QuestionId, QuizSettings};

    pub fn quiz(n: usize) -> Quiz {
        Quiz {
            id: "t".into(),
            title: "T".into(),
            description: None,
            assets_base_url: None,
            settings: QuizSettings::default(),
            questions: (0..n)
                .map(|i| Question {
                    id: QuestionId(format!("q{i}")),
                    topics: vec![if i % 2 == 0 { "even" } else { "odd" }.into()],
                    stem: format!("Q{i}"),
                    choices: ["a", "b", "c", "d"]
                        .iter()
                        .map(|c| Choice {
                            id: ChoiceId((*c).into()),
                            text: c.to_uppercase(),
                        })
                        .collect(),
                    lock_choices: i == 1,
                })
                .collect(),
        }
    }

    pub fn opts(mode: Mode) -> AttemptOptions {
        AttemptOptions {
            mode,
            shuffle_questions: false,
            shuffle_choices: false,
            time_limit_minutes: None,
            seed: 1,
            now_ms: 1000.0,
            subset: None,
        }
    }

    pub fn verdict(c: &str) -> Verdict {
        Verdict {
            correct_choice: ChoiceId(c.into()),
            explanation: None,
        }
    }

    #[test]
    fn practice_locks_after_first_answer() {
        let q = quiz(3);
        let mut a = Attempt::new(&q, 0, opts(Mode::Practice));
        assert!(a.select(0, 2));
        assert!(!a.select(0, 1));
        assert_eq!(a.slots[0].answer, Some(2));
    }

    #[test]
    fn exam_allows_changes_until_finished() {
        let q = quiz(3);
        let mut a = Attempt::new(&q, 0, opts(Mode::Exam));
        assert!(a.select(0, 2));
        assert!(a.select(0, 1));
        a.clear(0);
        assert_eq!(a.slots[0].answer, None);
        a.select(0, 3);
        a.finish(5000.0);
        assert!(!a.select(0, 0));
        assert_eq!(a.slots[0].answer, Some(3));
    }

    #[test]
    fn shuffle_keeps_locked_choices() {
        let q = quiz(6);
        let mut o = opts(Mode::Exam);
        o.shuffle_questions = true;
        o.shuffle_choices = true;
        o.seed = 42;
        let a = Attempt::new(&q, 0, o);
        let locked = a.slots.iter().find(|s| s.question == 1).unwrap();
        assert_eq!(locked.choice_order, vec![0, 1, 2, 3]);
        let mut seen: Vec<usize> = a.slots.iter().map(|s| s.question).collect();
        seen.sort();
        assert_eq!(seen, (0..6).collect::<Vec<_>>());
    }

    #[test]
    fn deadline_only_in_exam_mode() {
        let q = quiz(1);
        let mut o = opts(Mode::Practice);
        o.time_limit_minutes = Some(10);
        assert_eq!(Attempt::new(&q, 0, o).deadline_ms, None);
        let mut o = opts(Mode::Exam);
        o.time_limit_minutes = Some(10);
        let mut a = Attempt::new(&q, 0, o);
        assert_eq!(a.remaining_ms(1000.0 + 60_000.0), Some(540_000.0));
        assert!(a.is_timed_out(1000.0 + 700_000.0));
        a.finish(1000.0 + 700_000.0);
        assert_eq!(a.finished_at_ms, Some(1000.0 + 600_000.0));
    }

    #[test]
    fn elimination_rules() {
        let q = quiz(1);
        let mut a = Attempt::new(&q, 0, opts(Mode::Exam));
        a.toggle_eliminated(0, 1);
        a.toggle_eliminated(0, 2);
        assert_eq!(a.slots[0].eliminated, vec![1, 2]);
        a.select(0, 1);
        assert_eq!(a.slots[0].eliminated, vec![2]);
        a.toggle_eliminated(0, 1); // selected answer cannot be struck
        assert_eq!(a.slots[0].eliminated, vec![2]);
        a.toggle_eliminated(0, 2);
        assert!(a.slots[0].eliminated.is_empty());
    }

    #[test]
    fn outcomes_and_missed() {
        let q = quiz(3);
        let mut a = Attempt::new(&q, 0, opts(Mode::Exam));
        a.select(0, 0); // a
        a.select(1, 1); // b
        assert_eq!(a.outcome(&q, 0), None);
        for p in 0..3 {
            a.set_verdict(p, verdict("a"));
        }
        assert_eq!(a.outcome(&q, 0), Some(Outcome::Correct));
        assert_eq!(a.outcome(&q, 1), Some(Outcome::Wrong));
        assert_eq!(a.outcome(&q, 2), Some(Outcome::Empty));
        assert_eq!(a.missed_questions(&q), vec![1, 2]);
    }

    #[test]
    fn subset_retry() {
        let q = quiz(5);
        let mut o = opts(Mode::Practice);
        o.subset = Some(vec![4, 1]);
        let a = Attempt::new(&q, 0, o);
        assert_eq!(
            a.slots.iter().map(|s| s.question).collect::<Vec<_>>(),
            vec![4, 1]
        );
    }

    #[test]
    fn navigation_stays_in_range() {
        let q = quiz(2);
        let mut a = Attempt::new(&q, 0, opts(Mode::Exam));
        a.prev();
        assert_eq!(a.current, 0);
        a.next();
        a.next();
        assert_eq!(a.current, 1);
        a.go_to(9);
        assert_eq!(a.current, 1);
    }

    #[test]
    fn serde_roundtrip() {
        let q = quiz(2);
        let mut a = Attempt::new(&q, 99, opts(Mode::Exam));
        a.select(1, 3);
        a.toggle_flag(0);
        let json = serde_json::to_string(&a).unwrap();
        let back: Attempt = serde_json::from_str(&json).unwrap();
        assert_eq!(a, back);
    }
}
