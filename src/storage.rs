//! `localStorage` helpers. Every call is best effort: private windows or
//! blocked storage must not break the app, they only lose persistence.

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use web_sys::Storage;

use crate::domain::{Attempt, Mode};

pub const LANG: &str = "prova:lang";
pub const THEME: &str = "prova:theme";
const LAST_QUIZ: &str = "prova:quiz";
const ATTEMPT: &str = "prova:attempt";
const HISTORY_PREFIX: &str = "prova:history:";
const HISTORY_LIMIT: usize = 10;

fn storage() -> Option<Storage> {
    web_sys::window()?.local_storage().ok()?
}

pub fn get(key: &str) -> Option<String> {
    storage()?.get_item(key).ok()?
}

pub fn set(key: &str, value: &str) {
    if let Some(s) = storage() {
        let _ = s.set_item(key, value);
    }
}

pub fn remove(key: &str) {
    if let Some(s) = storage() {
        let _ = s.remove_item(key);
    }
}

fn load<T: DeserializeOwned>(key: &str) -> Option<T> {
    serde_json::from_str(&get(key)?).ok()
}

fn save<T: Serialize>(key: &str, value: &T) {
    if let Ok(json) = serde_json::to_string(value) {
        set(key, &json);
    }
}

/// The last loaded quiz, so a refreshed page can restore it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedQuiz {
    pub raw: String,
    pub label: String,
}

pub fn load_quiz() -> Option<SavedQuiz> {
    load(LAST_QUIZ)
}

pub fn save_quiz(q: &SavedQuiz) {
    save(LAST_QUIZ, q);
}

pub fn load_attempt() -> Option<Attempt> {
    load(ATTEMPT)
}

pub fn save_attempt(a: Option<&Attempt>) {
    match a {
        Some(a) => save(ATTEMPT, a),
        None => remove(ATTEMPT),
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub finished_at_ms: f64,
    pub mode: Mode,
    pub correct: usize,
    pub total: usize,
    pub duration_ms: f64,
}

/// Newest first.
pub fn history(quiz_id: &str) -> Vec<HistoryEntry> {
    load(&format!("{HISTORY_PREFIX}{quiz_id}")).unwrap_or_default()
}

pub fn push_history(quiz_id: &str, entry: HistoryEntry) {
    let mut list = history(quiz_id);
    list.insert(0, entry);
    list.truncate(HISTORY_LIMIT);
    save(&format!("{HISTORY_PREFIX}{quiz_id}"), &list);
}
