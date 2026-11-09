//! The test itself: questions and their choices.
//!
//! The correct answers are deliberately **not** part of [`Question`]. They live
//! in [`AnswerKey`], which only the local grader sees. In later phases the key
//! will not reach the browser at all and a service will grade the answers; the
//! rest of the app does not need to change for that.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct QuestionId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ChoiceId(pub String);

#[derive(Debug, Clone, PartialEq)]
pub struct Quiz {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    /// Base URL for relative image paths in the Markdown content.
    pub assets_base_url: Option<String>,
    pub settings: QuizSettings,
    pub questions: Vec<Question>,
}

/// Defaults suggested by the author; the student can change them before starting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct QuizSettings {
    pub shuffle_questions: bool,
    pub shuffle_choices: bool,
    pub time_limit_minutes: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Question {
    pub id: QuestionId,
    pub topics: Vec<String>,
    /// Markdown.
    pub stem: String,
    pub choices: Vec<Choice>,
    /// Keep the authored order even when choices are shuffled
    /// ("All of the above", "Only I and II" ...).
    pub lock_choices: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Choice {
    pub id: ChoiceId,
    /// Markdown.
    pub text: String,
}

impl Quiz {
    /// All topics in order of first appearance.
    pub fn topics(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for t in self.questions.iter().flat_map(|q| q.topics.iter()) {
            if !out.contains(t) {
                out.push(t.clone());
            }
        }
        out
    }
}

impl Question {
    pub fn choice_index(&self, id: &ChoiceId) -> Option<usize> {
        self.choices.iter().position(|c| &c.id == id)
    }
}

/// Correct answer and explanation of each question.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AnswerKey {
    entries: BTreeMap<QuestionId, KeyEntry>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct KeyEntry {
    pub correct: ChoiceId,
    /// Markdown.
    pub explanation: Option<String>,
}

impl AnswerKey {
    pub fn insert(&mut self, question: QuestionId, entry: KeyEntry) {
        self.entries.insert(question, entry);
    }

    pub fn get(&self, question: &QuestionId) -> Option<&KeyEntry> {
        self.entries.get(question)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// What a grader says about one question.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Verdict {
    pub correct_choice: ChoiceId,
    pub explanation: Option<String>,
}
