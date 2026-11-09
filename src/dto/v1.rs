//! Quiz file format, version 1.
//!
//! ```json
//! {
//!   "schema_version": 1,
//!   "id": "exam-00",
//!   "title": "...",
//!   "description": "Markdown (optional)",
//!   "assets_base_url": "https://.../ (optional)",
//!   "settings": { "shuffle_questions": true, "shuffle_options": true, "time_limit_minutes": 40 },
//!   "questions": [
//!     {
//!       "id": "q01",
//!       "topics": ["test"],
//!       "stem": "Markdown",
//!       "options": [{ "id": "a", "text": "Markdown" }],
//!       "lock_options": false,
//!       "answer": "a",
//!       "explanation": "Markdown (optional)"
//!     }
//!   ]
//! }
//! ```
//! Unknown fields are ignored, so newer files still load in older apps.

use std::collections::HashSet;

use serde::Deserialize;

use super::{Parsed, SchemaError};
use crate::domain::{
    AnswerKey, Choice, ChoiceId, KeyEntry, Question, QuestionId, Quiz, QuizSettings,
};

#[derive(Deserialize)]
struct QuizFile {
    id: String,
    title: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    assets_base_url: Option<String>,
    #[serde(default)]
    settings: SettingsFile,
    questions: Vec<QuestionFile>,
}

#[derive(Deserialize, Default)]
struct SettingsFile {
    #[serde(default)]
    shuffle_questions: bool,
    #[serde(default)]
    shuffle_options: bool,
    #[serde(default)]
    time_limit_minutes: Option<u32>,
}

#[derive(Deserialize)]
struct QuestionFile {
    id: String,
    #[serde(default)]
    topics: Vec<String>,
    stem: String,
    options: Vec<OptionFile>,
    #[serde(default)]
    lock_options: bool,
    #[serde(default)]
    answer: Option<String>,
    #[serde(default)]
    explanation: Option<String>,
}

#[derive(Deserialize)]
struct OptionFile {
    id: String,
    text: String,
}

fn blank(s: &str) -> bool {
    s.trim().is_empty()
}

fn non_blank(s: Option<String>) -> Option<String> {
    s.filter(|s| !blank(s))
}

pub fn parse(text: &str) -> Result<Parsed, SchemaError> {
    let file: QuizFile = serde_json::from_str(text)?;

    if blank(&file.id) {
        return Err(SchemaError::EmptyField {
            question: None,
            field: "id",
        });
    }
    if blank(&file.title) {
        return Err(SchemaError::EmptyField {
            question: None,
            field: "title",
        });
    }
    if file.questions.is_empty() {
        return Err(SchemaError::NoQuestions);
    }

    let with_answer = file.questions.iter().filter(|q| q.answer.is_some()).count();
    if with_answer != 0 && with_answer != file.questions.len() {
        let missing = file.questions.iter().find(|q| q.answer.is_none()).unwrap();
        return Err(SchemaError::PartialAnswers {
            missing: missing.id.clone(),
        });
    }

    let mut ids = HashSet::new();
    let mut key = AnswerKey::default();
    let mut questions = Vec::with_capacity(file.questions.len());

    for q in file.questions {
        if blank(&q.id) {
            return Err(SchemaError::EmptyField {
                question: None,
                field: "questions[].id",
            });
        }
        if !ids.insert(q.id.clone()) {
            return Err(SchemaError::DuplicateQuestion(q.id));
        }
        if blank(&q.stem) {
            return Err(SchemaError::EmptyField {
                question: Some(q.id),
                field: "stem",
            });
        }
        if q.options.len() < 2 {
            return Err(SchemaError::TooFewChoices(q.id));
        }
        let mut choice_ids = HashSet::new();
        for o in &q.options {
            if blank(&o.id) || blank(&o.text) {
                return Err(SchemaError::EmptyField {
                    question: Some(q.id),
                    field: "options[]",
                });
            }
            if !choice_ids.insert(o.id.as_str()) {
                return Err(SchemaError::DuplicateChoice {
                    question: q.id.clone(),
                    choice: o.id.clone(),
                });
            }
        }

        let id = QuestionId(q.id.clone());
        if let Some(answer) = q.answer {
            if !choice_ids.contains(answer.as_str()) {
                return Err(SchemaError::UnknownAnswer {
                    question: q.id,
                    answer,
                });
            }
            key.insert(
                id.clone(),
                KeyEntry {
                    correct: ChoiceId(answer),
                    explanation: non_blank(q.explanation),
                },
            );
        }

        questions.push(Question {
            id,
            topics: q.topics.into_iter().filter(|t| !blank(t)).collect(),
            stem: q.stem,
            choices: q
                .options
                .into_iter()
                .map(|o| Choice {
                    id: ChoiceId(o.id),
                    text: o.text,
                })
                .collect(),
            lock_choices: q.lock_options,
        });
    }

    Ok(Parsed {
        quiz: Quiz {
            id: file.id,
            title: file.title,
            description: non_blank(file.description),
            assets_base_url: non_blank(file.assets_base_url),
            settings: QuizSettings {
                shuffle_questions: file.settings.shuffle_questions,
                shuffle_choices: file.settings.shuffle_options,
                time_limit_minutes: file.settings.time_limit_minutes.filter(|&m| m > 0),
            },
            questions,
        },
        key: (with_answer > 0).then_some(key),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wrap(questions: &str) -> String {
        format!(r#"{{"schema_version":1,"id":"x","title":"X","questions":[{questions}]}}"#)
    }

    const Q: &str = r#"{"id":"q1","stem":"S","options":[{"id":"a","text":"A"},{"id":"b","text":"B"}],"answer":"b","explanation":"  "}"#;

    #[test]
    fn parses_minimal_quiz() {
        let p = parse(&wrap(Q)).unwrap();
        assert_eq!(p.quiz.questions.len(), 1);
        let key = p.key.unwrap();
        let e = key.get(&QuestionId("q1".into())).unwrap();
        assert_eq!(e.correct, ChoiceId("b".into()));
        assert_eq!(e.explanation, None); // blank -> None
        assert_eq!(p.quiz.settings, QuizSettings::default());
    }

    #[test]
    fn answers_are_optional_but_all_or_nothing() {
        let no_answer =
            r#"{"id":"q2","stem":"S","options":[{"id":"a","text":"A"},{"id":"b","text":"B"}]}"#;
        assert_eq!(parse(&wrap(no_answer)).unwrap().key, None);
        assert_eq!(
            parse(&wrap(&format!("{Q},{no_answer}"))),
            Err(SchemaError::PartialAnswers {
                missing: "q2".into()
            })
        );
    }

    #[test]
    fn validation_errors() {
        assert_eq!(parse(&wrap("")), Err(SchemaError::NoQuestions));
        assert_eq!(
            parse(&wrap(&format!("{Q},{Q}"))),
            Err(SchemaError::DuplicateQuestion("q1".into()))
        );
        let unknown = Q.replace(r#""answer":"b""#, r#""answer":"z""#);
        assert_eq!(
            parse(&wrap(&unknown)),
            Err(SchemaError::UnknownAnswer {
                question: "q1".into(),
                answer: "z".into()
            })
        );
        let one = r#"{"id":"q1","stem":"S","options":[{"id":"a","text":"A"}],"answer":"a"}"#;
        assert_eq!(
            parse(&wrap(one)),
            Err(SchemaError::TooFewChoices("q1".into()))
        );
        let dup = Q.replace(r#""id":"b""#, r#""id":"a""#);
        assert_eq!(
            parse(&wrap(&dup)),
            Err(SchemaError::DuplicateChoice {
                question: "q1".into(),
                choice: "a".into()
            })
        );
    }

    #[test]
    fn unknown_fields_are_ignored() {
        let q = Q.replace(r#""stem":"S""#, r#""stem":"S","difficulty":"hard""#);
        assert!(parse(&wrap(&q)).is_ok());
    }
}
