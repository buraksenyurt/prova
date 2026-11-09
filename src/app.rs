use std::sync::Arc;

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::components::{
    header::AppHeader, home::Home, intro::Intro, results::Results, runner::Runner,
};
use crate::domain::{Attempt, AttemptOptions, Quiz, summarize};
use crate::grading::{Grader, LocalGrader, Submission};
use crate::i18n::I18n;
use crate::source::{LoadError, LoadedQuiz, Origin, QuizSource, TextSource, UrlSource, from_text};
use crate::storage::{self, HistoryEntry, SavedQuiz};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Home,
    Intro,
    Run,
    Results,
}

/// A loaded quiz and the grader that goes with it.
pub struct Session {
    pub quiz: Quiz,
    /// `None` when the quiz has no answer key and no remote grader exists yet.
    pub grader: Option<Arc<dyn Grader>>,
    pub origin: Origin,
    pub fingerprint: u64,
}

/// App-wide state, shared through context. All fields are `Copy` signals.
#[derive(Clone, Copy)]
pub struct AppState {
    pub screen: RwSignal<Screen>,
    pub session: RwSignal<Option<Arc<Session>>>,
    pub attempt: RwSignal<Option<Attempt>>,
    pub loading: RwSignal<bool>,
    pub load_error: RwSignal<Option<LoadError>>,
    pub grade_failed: RwSignal<bool>,
}

pub fn use_app() -> AppState {
    expect_context::<AppState>()
}

pub fn now_ms() -> f64 {
    js_sys::Date::now()
}

impl AppState {
    fn new() -> Self {
        AppState {
            screen: RwSignal::new(Screen::Home),
            session: RwSignal::new(None),
            attempt: RwSignal::new(None),
            loading: RwSignal::new(false),
            load_error: RwSignal::new(None),
            grade_failed: RwSignal::new(false),
        }
    }

    /// Loads a quiz from any source and opens its intro screen.
    pub fn load(self, source: impl QuizSource + 'static) {
        if self.loading.get_untracked() {
            return;
        }
        self.loading.set(true);
        self.load_error.set(None);
        let fut = source.load();
        spawn_local(async move {
            match fut.await {
                Ok(loaded) => {
                    storage::save_quiz(&SavedQuiz {
                        raw: loaded.raw.clone(),
                        label: loaded.origin.label(),
                    });
                    self.open(loaded);
                    self.screen.set(Screen::Intro);
                }
                Err(e) => self.load_error.set(Some(e)),
            }
            self.loading.set(false);
        });
    }

    fn open(self, loaded: LoadedQuiz) {
        let grader = loaded
            .parsed
            .key
            .map(|key| Arc::new(LocalGrader::new(key)) as Arc<dyn Grader>);
        self.session.set(Some(Arc::new(Session {
            quiz: loaded.parsed.quiz,
            grader,
            origin: loaded.origin,
            fingerprint: loaded.fingerprint,
        })));
        self.attempt.set(None);
        self.grade_failed.set(false);
    }

    /// Continues the attempt saved in the browser, if its quiz is still there.
    pub fn resume(self, saved: SavedQuiz, attempt: Attempt) {
        match from_text(saved.raw, Origin::Saved(saved.label)) {
            Ok(loaded) if loaded.fingerprint == attempt.fingerprint => {
                self.open(loaded);
                let finished = attempt.is_finished();
                self.attempt.set(Some(attempt));
                self.screen.set(if finished {
                    Screen::Results
                } else {
                    Screen::Run
                });
            }
            _ => storage::save_attempt(None),
        }
    }

    pub fn start(self, opts: AttemptOptions) {
        let Some(session) = self.session.get_untracked() else {
            return;
        };
        self.grade_failed.set(false);
        self.attempt
            .set(Some(Attempt::new(&session.quiz, session.fingerprint, opts)));
        self.screen.set(Screen::Run);
        scroll_top();
    }

    /// Asks the grader for verdicts of the given positions.
    pub fn grade(self, positions: Vec<usize>, then: impl FnOnce() + 'static) {
        let Some(session) = self.session.get_untracked() else {
            return;
        };
        let Some(grader) = session.grader.clone() else {
            self.grade_failed.set(true);
            then();
            return;
        };
        let submissions: Vec<Submission> = self.attempt.with_untracked(|a| {
            let Some(a) = a else { return Vec::new() };
            positions
                .iter()
                .map(|&p| {
                    let slot = &a.slots[p];
                    let q = &session.quiz.questions[slot.question];
                    Submission {
                        question: q.id.clone(),
                        choice: slot.answer.map(|i| q.choices[i].id.clone()),
                    }
                })
                .collect()
        });
        if submissions.is_empty() {
            then();
            return;
        }
        let fut = grader.grade(submissions);
        spawn_local(async move {
            match fut.await {
                Ok(verdicts) => self.attempt.update(|a| {
                    if let Some(a) = a {
                        for (p, v) in positions.into_iter().zip(verdicts) {
                            a.set_verdict(p, v);
                        }
                    }
                }),
                Err(_) => self.grade_failed.set(true),
            }
            then();
        });
    }

    /// Ends the attempt, grades what is left and records it in the history.
    pub fn finish(self) {
        let now = now_ms();
        let mut ungraded = Vec::new();
        self.attempt.update(|a| {
            if let Some(a) = a {
                if a.is_finished() {
                    return;
                }
                a.finish(now);
                ungraded = a.ungraded();
            }
        });
        self.screen.set(Screen::Results);
        scroll_top();
        self.grade(ungraded, move || {
            let Some(session) = self.session.get_untracked() else {
                return;
            };
            self.attempt.with_untracked(|a| {
                if let Some(a) = a {
                    let s = summarize(&session.quiz, a, now);
                    storage::push_history(
                        &session.quiz.id,
                        HistoryEntry {
                            finished_at_ms: a.finished_at_ms.unwrap_or(now),
                            mode: a.mode,
                            correct: s.correct,
                            total: s.total,
                            duration_ms: s.duration_ms,
                        },
                    );
                }
            });
        });
    }

    pub fn go(self, screen: Screen) {
        self.screen.set(screen);
        scroll_top();
    }
}

fn scroll_top() {
    if let Some(w) = web_sys::window() {
        w.scroll_to_with_x_and_y(0.0, 0.0);
    }
}

/// `?quiz=<url>`: a link the instructor can share.
fn quiz_url_param() -> Option<String> {
    let search = web_sys::window()?.location().search().ok()?;
    let params = web_sys::UrlSearchParams::new_with_str(&search).ok()?;
    params.get("quiz").filter(|u| !u.trim().is_empty())
}

#[component]
pub fn App() -> impl IntoView {
    provide_context(I18n::new());
    let state = AppState::new();
    provide_context(state);

    // Every change of the attempt is saved, so a refresh loses nothing.
    Effect::new(move |_| {
        state.attempt.with(|a| {
            if let Some(a) = a {
                storage::save_attempt(Some(a));
            }
        })
    });

    if let Some(url) = quiz_url_param() {
        state.load(UrlSource { url });
    }

    view! {
        <AppHeader />
        <main class="container">
            {move || match state.screen.get() {
                Screen::Home => view! { <Home /> }.into_any(),
                Screen::Intro => view! { <Intro /> }.into_any(),
                Screen::Run => view! { <Runner /> }.into_any(),
                Screen::Results => view! { <Results /> }.into_any(),
            }}
        </main>
    }
}

/// The built-in sample, used by the home screen.
pub fn sample_source() -> TextSource {
    TextSource {
        text: crate::sample::DEMO_JSON.to_string(),
        origin: Origin::Sample,
    }
}
