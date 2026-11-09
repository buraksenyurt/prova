//! Quiz file formats and their mapping to the domain model.
//!
//! The file says which format it uses (`schema_version`); each version has its
//! own module. A new version only adds a module and a match arm in [`parse`].

pub mod v1;

use serde::Deserialize;

use crate::domain::{AnswerKey, Quiz};

/// A parsed quiz. `key` is `None` when the file carries no answers
/// (a service will grade them).
#[derive(Debug, Clone, PartialEq)]
pub struct Parsed {
    pub quiz: Quiz,
    pub key: Option<AnswerKey>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SchemaError {
    /// Invalid JSON or a field with the wrong type / missing.
    Json {
        line: usize,
        column: usize,
        message: String,
    },
    UnsupportedVersion(u64),
    NoQuestions,
    EmptyField {
        question: Option<String>,
        field: &'static str,
    },
    DuplicateQuestion(String),
    TooFewChoices(String),
    DuplicateChoice {
        question: String,
        choice: String,
    },
    UnknownAnswer {
        question: String,
        answer: String,
    },
    /// Some questions have an answer and some do not.
    PartialAnswers {
        missing: String,
    },
}

impl From<serde_json::Error> for SchemaError {
    fn from(e: serde_json::Error) -> Self {
        // serde's message ends with " at line X column Y"; we show those separately.
        let msg = e.to_string();
        let message = match msg.rfind(" at line ") {
            Some(i) => msg[..i].to_string(),
            None => msg,
        };
        SchemaError::Json {
            line: e.line(),
            column: e.column(),
            message,
        }
    }
}

pub fn parse(text: &str) -> Result<Parsed, SchemaError> {
    #[derive(Deserialize)]
    struct Header {
        schema_version: Option<u64>,
    }
    let header: Header = serde_json::from_str(text)?;
    match header.schema_version.unwrap_or(1) {
        1 => v1::parse(text),
        v => Err(SchemaError::UnsupportedVersion(v)),
    }
}

/// FNV-1a over the content; identifies a quiz version for resuming attempts.
pub fn fingerprint(text: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in text.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_version() {
        let r = parse(r#"{"schema_version": 9}"#);
        assert_eq!(r, Err(SchemaError::UnsupportedVersion(9)));
    }

    #[test]
    fn json_errors_have_position() {
        match parse("{\n  \"schema_version\": 1,\n  oops\n}") {
            Err(SchemaError::Json { line, .. }) => assert_eq!(line, 3),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn fingerprint_changes_with_content() {
        assert_ne!(fingerprint("a"), fingerprint("b"));
        assert_eq!(fingerprint("abc"), fingerprint("abc"));
    }

    #[test]
    fn sample_quiz_is_valid() {
        let parsed = parse(crate::sample::DEMO_JSON).expect("demo.json must parse");
        assert!(parsed.quiz.questions.len() >= 8);
        assert_eq!(parsed.key.unwrap().len(), parsed.quiz.questions.len());
    }
}
