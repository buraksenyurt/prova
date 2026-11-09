use leptos::prelude::*;

use super::md;
use crate::app::{Screen, now_ms, use_app};
use crate::domain::{AttemptOptions, Mode};
use crate::i18n::use_i18n;
use crate::markdown::RenderOptions;
use crate::storage;

#[component]
pub fn Intro() -> impl IntoView {
    let i18n = use_i18n();
    let app = use_app();
    let Some(session) = app.session.get_untracked() else {
        app.go(Screen::Home);
        return ().into_any();
    };
    let quiz = &session.quiz;
    let settings = quiz.settings;

    let mode = RwSignal::new(Mode::Practice);
    let shuffle_q = RwSignal::new(settings.shuffle_questions);
    let shuffle_c = RwSignal::new(settings.shuffle_choices);
    let minutes = RwSignal::new(settings.time_limit_minutes.unwrap_or(0));
    let can_grade = session.grader.is_some();

    let start = move |_| {
        app.start(AttemptOptions {
            mode: mode.get_untracked(),
            shuffle_questions: shuffle_q.get_untracked(),
            shuffle_choices: shuffle_c.get_untracked(),
            time_limit_minutes: Some(minutes.get_untracked()).filter(|&m| m > 0),
            seed: now_ms() as u64,
            now_ms: now_ms(),
            subset: None,
        })
    };

    let count = quiz.questions.len();
    let topics = quiz.topics();
    let history = storage::history(&quiz.id);
    let description = quiz.description.as_deref().map(|d| {
        md(
            "description",
            d,
            RenderOptions {
                assets_base_url: quiz.assets_base_url.as_deref(),
                highlight_negatives: false,
            },
        )
    });
    let origin = session.origin.label();

    let mode_card = move |m: Mode| {
        let (title, desc, icon) = match m {
            Mode::Practice => (
                (move || i18n.t().mode_practice).into_any(),
                (move || i18n.t().mode_practice_desc).into_any(),
                "◎",
            ),
            Mode::Exam => (
                (move || i18n.t().mode_exam).into_any(),
                (move || i18n.t().mode_exam_desc).into_any(),
                "⏱",
            ),
        };
        view! {
            <button
                class="mode-card"
                class:active=move || mode.get() == m
                role="radio"
                aria-checked=move || (mode.get() == m).to_string()
                on:click=move |_| mode.set(m)
            >
                <span class="mode-icon" aria-hidden="true">{icon}</span>
                <strong>{title}</strong>
                <span class="muted">{desc}</span>
            </button>
        }
    };

    view! {
        <section class="intro">
            <button class="btn link back" on:click=move |_| app.go(Screen::Home)>
                "← "
                {move || i18n.t().change_quiz}
            </button>
            <div class="card">
                <h1>{quiz.title.clone()}</h1>
                {description}
                <div class="meta">
                    <span class="pill">{move || i18n.lang().question_count(count)}</span>
                    {settings
                        .time_limit_minutes
                        .map(|m| view! { <span class="pill">"⏱ " {move || i18n.lang().minutes(m)}</span> })}
                    <span class="pill subtle" title=move || i18n.t().source>
                        {origin}
                    </span>
                </div>
                {(!topics.is_empty())
                    .then(|| {
                        view! {
                            <div class="topics">
                                {topics
                                    .into_iter()
                                    .map(|t| view! { <span class="chip">{t}</span> })
                                    .collect_view()}
                            </div>
                        }
                    })}
            </div>

            <h2>{move || i18n.t().mode_title}</h2>
            <div class="mode-grid" role="radiogroup">
                {mode_card(Mode::Practice)}
                {mode_card(Mode::Exam)}
            </div>

            <div class="card options">
                <label class="check">
                    <input
                        type="checkbox"
                        prop:checked=move || shuffle_q.get()
                        on:change=move |ev| shuffle_q.set(event_target_checked(&ev))
                    />
                    <span>{move || i18n.t().shuffle_questions}</span>
                </label>
                <label class="check">
                    <input
                        type="checkbox"
                        prop:checked=move || shuffle_c.get()
                        on:change=move |ev| shuffle_c.set(event_target_checked(&ev))
                    />
                    <span>{move || i18n.t().shuffle_choices}</span>
                </label>
                <Show when=move || mode.get() == Mode::Exam>
                    <label class="number">
                        <span>{move || i18n.t().time_limit_input}</span>
                        <input
                            type="number"
                            min="0"
                            max="600"
                            prop:value=move || minutes.get().to_string()
                            on:input=move |ev| {
                                minutes.set(event_target_value(&ev).parse().unwrap_or(0).min(600))
                            }
                        />
                        <small class="muted">{move || i18n.t().no_limit_hint}</small>
                    </label>
                </Show>
            </div>

            {(!can_grade)
                .then(|| view! { <div class="alert warn">{move || i18n.t().no_key}</div> })}

            <button class="btn primary big" on:click=start disabled=!can_grade>
                {move || i18n.t().start}
                " →"
            </button>

            {(!history.is_empty())
                .then(|| {
                    view! {
                        <div class="card history">
                            <h3>{move || i18n.t().history_title}</h3>
                            <ul>
                                {history
                                    .into_iter()
                                    .map(|h| {
                                        let pct = if h.total == 0 {
                                            0.0
                                        } else {
                                            h.correct as f64 * 100.0 / h.total as f64
                                        };
                                        view! {
                                            <li>
                                                <span class="muted">
                                                    {move || i18n.lang().date_time(h.finished_at_ms)}
                                                </span>
                                                <span>
                                                    {move || match h.mode {
                                                        Mode::Practice => i18n.t().mode_practice,
                                                        Mode::Exam => i18n.t().mode_exam,
                                                    }}
                                                </span>
                                                <span>{move || i18n.lang().ratio(h.correct, h.total)}</span>
                                                <strong>{move || i18n.lang().percent(pct)}</strong>
                                            </li>
                                        }
                                    })
                                    .collect_view()}
                            </ul>
                        </div>
                    }
                })}
        </section>
    }
    .into_any()
}
