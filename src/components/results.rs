use leptos::prelude::*;

use super::question::{self, ChoiceMode};
use crate::app::{Screen, now_ms, use_app};
use crate::domain::{AttemptOptions, Outcome, summarize};
use crate::i18n::use_i18n;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Filter {
    All,
    Wrong,
    Empty,
    Flagged,
}

#[component]
pub fn Results() -> impl IntoView {
    let i18n = use_i18n();
    let app = use_app();
    let Some(session) = app.session.get_untracked() else {
        app.go(Screen::Home);
        return ().into_any();
    };
    let filter = RwSignal::new(Filter::All);

    let s1 = session.clone();
    let summary = Memo::new(move |_| {
        app.attempt.with(|a| {
            a.as_ref()
                .map(|a| summarize(&s1.quiz, a, a.finished_at_ms.unwrap_or_else(now_ms)))
        })
    });

    let s2 = session.clone();
    let retry_missed = move |_| {
        let (missed, mode) = app.attempt.with_untracked(|a| {
            a.as_ref().map_or((Vec::new(), None), |a| {
                (a.missed_questions(&s2.quiz), Some(a.mode))
            })
        });
        let Some(mode) = mode else { return };
        if missed.is_empty() {
            return;
        }
        app.start(AttemptOptions {
            mode,
            shuffle_questions: true,
            shuffle_choices: true,
            time_limit_minutes: None,
            seed: now_ms() as u64,
            now_ms: now_ms(),
            subset: Some(missed),
        });
    };

    let s3 = session.clone();
    let missed = Memo::new(move |_| {
        app.attempt
            .with(|a| a.as_ref().map_or(0, |a| a.missed_questions(&s3.quiz).len()))
    });
    let missed_count = move || missed.get();

    let timed_out = move || {
        app.attempt.with(|a| {
            a.as_ref()
                .is_some_and(|a| a.deadline_ms.is_some() && a.finished_at_ms == a.deadline_ms)
        })
    };

    let s4 = session.clone();
    let counts = move |f: Filter| {
        app.attempt.with(|a| {
            a.as_ref().map_or(0, |a| {
                (0..a.len())
                    .filter(|&p| matches_filter(f, a.outcome(&s4.quiz, p), a.slots[p].flagged))
                    .count()
            })
        })
    };

    let s5 = session.clone();
    let review = move || {
        let session = s5.clone();
        let f = filter.get();
        app.attempt.with(|a| {
            let a = a.as_ref()?;
            let quiz = &session.quiz;
            let items: Vec<_> = (0..a.len())
                .filter(|&p| matches_filter(f, a.outcome(quiz, p), a.slots[p].flagged))
                .map(|p| {
                    let slot = &a.slots[p];
                    let outcome = a.outcome(quiz, p);
                    let badge = match outcome {
                        Some(Outcome::Correct) => "ok",
                        Some(Outcome::Wrong) => "bad",
                        _ => "empty",
                    };
                    view! {
                        <article class=format!("card review-item {badge}")>
                            <div class="q-head">
                                <span class="q-number">{i18n.lang().question_of(p + 1, a.len())}</span>
                                <div class="q-topics">{question::topics(quiz, slot)}</div>
                                {slot.flagged.then(|| view! { <span class="flagged-tag">"⚑"</span> })}
                            </div>
                            {question::stem(quiz, slot)}
                            {question::choices(
                                quiz,
                                slot,
                                ChoiceMode { reveal: true, locked: true, on_pick: None, on_strike: None },
                                i18n,
                            )}
                            {question::feedback(quiz, slot, i18n)}
                        </article>
                    }
                })
                .collect();
            Some(if items.is_empty() {
                view! { <p class="muted center">{move || i18n.t().nothing_here}</p> }.into_any()
            } else {
                items.collect_view().into_any()
            })
        })
    };

    let tab = move |f: Filter| {
        let label = move || match f {
            Filter::All => i18n.t().filter_all,
            Filter::Wrong => i18n.t().filter_wrong,
            Filter::Empty => i18n.t().filter_empty,
            Filter::Flagged => i18n.t().filter_flagged,
        };
        let counts = counts.clone();
        view! {
            <button
                class="tab"
                class:active=move || filter.get() == f
                role="tab"
                on:click=move |_| filter.set(f)
            >
                {label}
                <span class="count">{move || counts(f)}</span>
            </button>
        }
    };

    view! {
        <section class="results">
            {move || {
                summary
                    .get()
                    .map(|s| {
                        let pct = s.score_percent();
                        let perfect = s.total > 0 && s.correct == s.total;
                        view! {
                            <div class="card score-card">
                                <div
                                    class="score-ring"
                                    style=format!("--p: {pct:.1}")
                                    role="img"
                                    aria-label=i18n.lang().percent(pct)
                                >
                                    <span>{i18n.lang().percent(pct)}</span>
                                </div>
                                <div class="score-body">
                                    <h1>{move || i18n.t().results_title}</h1>
                                    <p class="lead">{i18n.lang().ratio(s.correct, s.total)}</p>
                                    {perfect.then(|| view! { <p class="praise">{move || i18n.t().perfect}</p> })}
                                    {move || timed_out().then(|| view! { <p class="alert warn">{i18n.t().time_up}</p> })}
                                    <div class="stats">
                                        <div class="stat ok">
                                            <b>{s.correct}</b>
                                            <span>{move || i18n.t().correct}</span>
                                        </div>
                                        <div class="stat bad">
                                            <b>{s.wrong}</b>
                                            <span>{move || i18n.t().wrong}</span>
                                        </div>
                                        <div class="stat">
                                            <b>{s.empty}</b>
                                            <span>{move || i18n.t().empty}</span>
                                        </div>
                                        <div class="stat">
                                            <b>{i18n.lang().clock(s.duration_ms)}</b>
                                            <span>{move || i18n.t().duration}</span>
                                        </div>
                                    </div>
                                </div>
                            </div>
                            {(!s.topics.is_empty())
                                .then(|| {
                                    view! {
                                        <div class="card topics-card">
                                            <h2>{move || i18n.t().by_topic}</h2>
                                            <ul class="topic-bars">
                                                {s
                                                    .topics
                                                    .iter()
                                                    .map(|t| {
                                                        let p = t.percent();
                                                        let level = if p >= 75.0 {
                                                            "good"
                                                        } else if p >= 50.0 {
                                                            "mid"
                                                        } else {
                                                            "low"
                                                        };
                                                        view! {
                                                            <li>
                                                                <span class="topic-name">{t.topic.clone()}</span>
                                                                <span class="bar">
                                                                    <span
                                                                        class=format!("bar-fill {level}")
                                                                        style:width=format!("{p:.0}%")
                                                                    ></span>
                                                                </span>
                                                                <span class="topic-score">
                                                                    {format!("{}/{}", t.correct, t.total)}
                                                                </span>
                                                            </li>
                                                        }
                                                    })
                                                    .collect_view()}
                                            </ul>
                                        </div>
                                    }
                                })}
                        }
                    })
            }}

            <div class="actions">
                <Show when=move || { missed_count() > 0 }>
                    <button class="btn primary" on:click=retry_missed.clone()>
                        "↻ "
                        {move || i18n.lang().retry_missed(missed_count())}
                    </button>
                </Show>
                <button class="btn" on:click=move |_| app.go(Screen::Intro)>
                    {move || i18n.t().restart}
                </button>
                <button class="btn ghost" on:click=move |_| app.go(Screen::Home)>
                    {move || i18n.t().new_quiz}
                </button>
            </div>

            <h2>{move || i18n.t().review_title}</h2>
            <div class="tabs" role="tablist">
                {tab(Filter::All)}
                {tab(Filter::Wrong)}
                {tab(Filter::Empty)}
                {tab(Filter::Flagged)}
            </div>
            <div class="review-list">{review}</div>
        </section>
    }
    .into_any()
}

fn matches_filter(f: Filter, outcome: Option<Outcome>, flagged: bool) -> bool {
    match f {
        Filter::All => true,
        Filter::Wrong => outcome == Some(Outcome::Wrong),
        Filter::Empty => outcome == Some(Outcome::Empty),
        Filter::Flagged => flagged,
    }
}
