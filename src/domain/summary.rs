//! Score and topic breakdown of an attempt.

use super::attempt::{Attempt, Outcome};
use super::quiz::Quiz;

#[derive(Debug, Clone, PartialEq)]
pub struct Summary {
    pub total: usize,
    pub correct: usize,
    pub wrong: usize,
    pub empty: usize,
    /// Questions without a verdict (grading pending or failed).
    pub pending: usize,
    pub duration_ms: f64,
    pub topics: Vec<TopicStat>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TopicStat {
    pub topic: String,
    pub correct: usize,
    pub total: usize,
}

impl Summary {
    /// 0.0..=100.0
    pub fn score_percent(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.correct as f64 * 100.0 / self.total as f64
        }
    }
}

impl TopicStat {
    pub fn percent(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.correct as f64 * 100.0 / self.total as f64
        }
    }
}

pub fn summarize(quiz: &Quiz, attempt: &Attempt, now_ms: f64) -> Summary {
    let mut s = Summary {
        total: attempt.len(),
        correct: 0,
        wrong: 0,
        empty: 0,
        pending: 0,
        duration_ms: attempt.elapsed_ms(now_ms),
        topics: Vec::new(),
    };

    for pos in 0..attempt.len() {
        let outcome = attempt.outcome(quiz, pos);
        match outcome {
            Some(Outcome::Correct) => s.correct += 1,
            Some(Outcome::Wrong) => s.wrong += 1,
            Some(Outcome::Empty) => s.empty += 1,
            None => s.pending += 1,
        }
        for topic in &quiz.questions[attempt.slots[pos].question].topics {
            let stat = match s.topics.iter_mut().find(|t| &t.topic == topic) {
                Some(t) => t,
                None => {
                    s.topics.push(TopicStat {
                        topic: topic.clone(),
                        correct: 0,
                        total: 0,
                    });
                    s.topics.last_mut().unwrap()
                }
            };
            stat.total += 1;
            if outcome == Some(Outcome::Correct) {
                stat.correct += 1;
            }
        }
    }

    // Weakest topics first: that is where the student should look.
    s.topics.sort_by(|a, b| {
        a.percent()
            .total_cmp(&b.percent())
            .then(b.total.cmp(&a.total))
    });
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::attempt::Mode;
    use crate::domain::attempt::tests::{opts, quiz, verdict};

    #[test]
    fn counts_and_topics() {
        let q = quiz(4); // topics: even, odd, even, odd
        let mut a = Attempt::new(&q, 0, opts(Mode::Exam));
        a.select(0, 0); // correct (even)
        a.select(1, 0); // correct (odd)
        a.select(2, 1); // wrong (even)
        for p in 0..3 {
            a.set_verdict(p, verdict("a"));
        }
        a.finish(61_000.0);
        let s = summarize(&q, &a, 99_999.0);
        assert_eq!(
            (s.total, s.correct, s.wrong, s.empty, s.pending),
            (4, 2, 1, 0, 1)
        );
        assert_eq!(s.score_percent(), 50.0);
        assert_eq!(s.duration_ms, 60_000.0);
        // even: 1/2, odd: 1/2 (the pending one counts as not correct)
        assert_eq!(s.topics.len(), 2);
        assert!(s.topics.iter().all(|t| t.correct == 1 && t.total == 2));
    }
}
