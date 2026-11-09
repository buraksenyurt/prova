//! Who decides whether an answer is correct.
//!
//! Phase 1 grades in the browser with the answer key from the quiz file
//! ([`LocalGrader`]). Later a `RemoteGrader` can send the submissions to a
//! service instead; the UI only knows the [`Grader`] trait.
//!
//! The API is a batch on purpose: practice mode sends one question, exam mode
//! sends all of them at the end. Both are one call, local or remote.

use crate::domain::{AnswerKey, ChoiceId, QuestionId, Verdict};
use crate::source::LocalFuture;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Submission {
    pub question: QuestionId,
    /// `None`: left empty. The verdict still tells the correct choice.
    pub choice: Option<ChoiceId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GradeError(pub String);

pub trait Grader: Send + Sync {
    /// Returns one verdict per submission, in the same order.
    fn grade(&self, submissions: Vec<Submission>) -> LocalFuture<Result<Vec<Verdict>, GradeError>>;
}

pub struct LocalGrader {
    key: AnswerKey,
}

impl LocalGrader {
    pub fn new(key: AnswerKey) -> Self {
        LocalGrader { key }
    }

    fn grade_now(&self, submissions: &[Submission]) -> Result<Vec<Verdict>, GradeError> {
        submissions
            .iter()
            .map(|s| {
                self.key
                    .get(&s.question)
                    .map(|e| Verdict {
                        correct_choice: e.correct.clone(),
                        explanation: e.explanation.clone(),
                    })
                    .ok_or_else(|| GradeError(format!("no answer for {}", s.question.0)))
            })
            .collect()
    }
}

impl Grader for LocalGrader {
    fn grade(&self, submissions: Vec<Submission>) -> LocalFuture<Result<Vec<Verdict>, GradeError>> {
        let result = self.grade_now(&submissions);
        Box::pin(std::future::ready(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::KeyEntry;

    #[test]
    fn local_grader_uses_key() {
        let mut key = AnswerKey::default();
        key.insert(
            QuestionId("q1".into()),
            KeyEntry {
                correct: ChoiceId("c".into()),
                explanation: Some("why".into()),
            },
        );
        let g = LocalGrader::new(key);
        let v = g
            .grade_now(&[Submission {
                question: QuestionId("q1".into()),
                choice: None,
            }])
            .unwrap();
        assert_eq!(v[0].correct_choice, ChoiceId("c".into()));
        assert!(
            g.grade_now(&[Submission {
                question: QuestionId("x".into()),
                choice: None
            }])
            .is_err()
        );
    }
}
