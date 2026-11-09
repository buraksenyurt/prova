//! Pure domain model: no browser APIs, fully testable with `cargo test`.

pub mod attempt;
pub mod quiz;
pub mod rng;
pub mod summary;

pub use attempt::{Attempt, AttemptOptions, Mode, Outcome, Slot};
pub use quiz::{
    AnswerKey, Choice, ChoiceId, KeyEntry, Question, QuestionId, Quiz, QuizSettings, Verdict,
};
pub use summary::{Summary, TopicStat, summarize};
