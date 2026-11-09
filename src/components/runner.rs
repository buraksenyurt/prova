use std::time::Duration;

use leptos::prelude::*;
use wasm_bindgen::JsCast;

use super::question::{self, ChoiceMode};
use crate::app::{Screen, now_ms, use_app};
use crate::domain::{Mode, Outcome};
use crate::i18n::use_i18n;

#[component]
pub fn Runner() -> impl IntoView {
    let i18n = use_i18n();
    let app = use_app();
    let Some(session) = app.session.get_untracked() else {
        app.go(Screen::Home);
        return ().into_any();
    };
    if app.attempt.with_untracked(|a| a.is_none()) {
        app.go(Screen::Intro);
        return ().into_any();
    }

    let now = RwSignal::new(now_ms());
    let ticker =
        set_interval_with_handle(move || now.set(now_ms()), Duration::from_millis(500)).ok();
    on_cleanup(move || {
        if let Some(t) = ticker {
            t.clear();
        }
    });

    let confirm = RwSignal::new(false);
    let map_open = RwSignal::new(false);

    // The exam ends by itself when the time is up.
    Effect::new(move |_| {
        let t = now.get();
        let timed_out = app.attempt.with_untracked(|a| {
            a.as_ref()
                .is_some_and(|a| !a.is_finished() && a.is_timed_out(t))
        });
        if timed_out {
            app.finish();
        }
    });

    let mode = app
        .attempt
        .with_untracked(|a| a.as_ref().map(|a| a.mode))
        .unwrap_or(Mode::Practice);

    // ---------- actions ----------

    let pick = Callback::new(move |choice: usize| {
        let mut to_grade = None;
        app.attempt.update(|a| {
            if let Some(a) = a {
                let p = a.current;
                if a.select(p, choice) && a.mode == Mode::Practice {
                    to_grade = Some(p);
                }
            }
        });
        if let Some(p) = to_grade {
            app.grade(vec![p], || ());
        }
    });
    let strike = Callback::new(move |choice: usize| {
        app.attempt.update(|a| {
            if let Some(a) = a {
                let p = a.current;
                a.toggle_eliminated(p, choice);
            }
        })
    });
    let pick_display = move |k: usize| {
        let choice = app.attempt.with_untracked(|a| {
            a.as_ref()
                .and_then(|a| a.current_slot().choice_order.get(k).copied())
        });
        if let Some(c) = choice {
            pick.run(c);
        }
    };
    let next = move || app.attempt.update(|a| a.iter_mut().for_each(|a| a.next()));
    let prev = move || app.attempt.update(|a| a.iter_mut().for_each(|a| a.prev()));
    let go_to = move |p: usize| {
        app.attempt
            .update(|a| a.iter_mut().for_each(|a| a.go_to(p)));
        map_open.set(false);
    };
    let flag = move || {
        app.attempt.update(|a| {
            if let Some(a) = a {
                let p = a.current;
                a.toggle_flag(p);
            }
        })
    };
    let clear = move || {
        app.attempt.update(|a| {
            if let Some(a) = a {
                let p = a.current;
                a.clear(p);
            }
        })
    };
    let request_finish = move || {
        let unanswered = app
            .attempt
            .with_untracked(|a| a.as_ref().map_or(0, |a| a.unanswered()));
        if mode == Mode::Practice && unanswered == 0 {
            app.finish();
        } else {
            confirm.set(true);
        }
    };

    // ---------- keyboard ----------

    let keys = window_event_listener(leptos::ev::keydown, move |ev: web_sys::KeyboardEvent| {
        if ev.ctrl_key() || ev.meta_key() || ev.alt_key() {
            return;
        }
        let in_field = ev
            .target()
            .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
            .is_some_and(|el| matches!(el.tag_name().as_str(), "INPUT" | "SELECT" | "TEXTAREA"));
        if in_field {
            return;
        }
        let key = ev.key();
        if confirm.get_untracked() {
            if key == "Escape" {
                confirm.set(false);
            }
            return;
        }
        match key.as_str() {
            "ArrowRight" => next(),
            "ArrowLeft" => prev(),
            "f" | "F" => flag(),
            "Escape" => map_open.set(false),
            k if k.len() == 1 => {
                let c = k.chars().next().unwrap().to_ascii_lowercase();
                let index = match c {
                    'a'..='h' => Some(c as usize - 'a' as usize),
                    '1'..='9' => Some(c as usize - '1' as usize),
                    _ => None,
                };
                if let Some(i) = index {
                    pick_display(i);
                } else {
                    return;
                }
            }
            _ => return,
        }
        ev.prevent_default();
    });
    on_cleanup(move || keys.remove());

    // ---------- view ----------

    let title = session.quiz.title.clone();
    let s_card = session.clone();
    let s_map = session.clone();

    let progress = move || {
        app.attempt.with(|a| {
            a.as_ref()
                .map_or((0, 1), |a| (a.answered(), a.len().max(1)))
        })
    };

    let timer = move || {
        let t = now.get();
        app.attempt.with(|a| {
            let a = a.as_ref()?;
            Some(match a.remaining_ms(t) {
                Some(left) => (i18n.t().time_left, i18n.lang().clock(left), left < 60_000.0),
                None => (i18n.t().elapsed, i18n.lang().clock(a.elapsed_ms(t)), false),
            })
        })
    };

    let card = move || {
        let session = s_card.clone();
        app.attempt.with(|a| {
            let a = a.as_ref()?;
            let pos = a.current;
            let slot = &a.slots[pos];
            let quiz = &session.quiz;
            let locked = a.is_locked(pos);
            let reveal = a.mode == Mode::Practice && slot.answer.is_some();
            let is_last = pos + 1 == a.len();
            let answered = slot.answer.is_some();
            let flagged = slot.flagged;
            let number = i18n.lang().question_of(pos + 1, a.len());
            Some(view! {
                <div class="q-head">
                    <span class="q-number">{number}</span>
                    <div class="q-topics">{question::topics(quiz, slot)}</div>
                    <button
                        class="btn small flag-btn"
                        class:active=flagged
                        on:click=move |_| flag()
                        aria-pressed=flagged.to_string()
                    >
                        "⚑ "
                        {move || if flagged { i18n.t().unflag } else { i18n.t().flag }}
                    </button>
                </div>
                {question::stem(quiz, slot)}
                {question::choices(
                    quiz,
                    slot,
                    ChoiceMode {
                        reveal,
                        locked,
                        on_pick: Some(pick),
                        on_strike: Some(strike),
                    },
                    i18n,
                )}
                {reveal.then(|| question::feedback(quiz, slot, i18n))}
                <div class="q-nav">
                    <button class="btn" on:click=move |_| prev() disabled=pos == 0>
                        "← "
                        {move || i18n.t().prev}
                    </button>
                    {(mode == Mode::Exam && answered)
                        .then(|| {
                            view! {
                                <button class="btn link" on:click=move |_| clear()>
                                    {move || i18n.t().clear_answer}
                                </button>
                            }
                        })}
                    {if is_last {
                        view! {
                            <button class="btn primary" on:click=move |_| request_finish()>
                                {move || {
                                    if mode == Mode::Practice {
                                        i18n.t().see_results
                                    } else {
                                        i18n.t().finish
                                    }
                                }}
                            </button>
                        }
                            .into_any()
                    } else {
                        view! {
                            <button
                                class="btn"
                                class:primary=mode == Mode::Exam || answered
                                on:click=move |_| next()
                            >
                                {move || i18n.t().next}
                                " →"
                            </button>
                        }
                            .into_any()
                    }}
                </div>
            })
        })
    };

    let map = move || {
        let session = s_map.clone();
        app.attempt.with(|a| {
            let a = a.as_ref()?;
            Some(
                (0..a.len())
                    .map(|p| {
                        let slot = &a.slots[p];
                        let mut class = String::from("qmap-item");
                        if p == a.current {
                            class.push_str(" current");
                        }
                        if slot.answer.is_some() {
                            class.push_str(" answered");
                        }
                        if slot.flagged {
                            class.push_str(" flagged");
                        }
                        if a.mode == Mode::Practice {
                            match a.outcome(&session.quiz, p) {
                                Some(Outcome::Correct) => class.push_str(" correct"),
                                Some(Outcome::Wrong) => class.push_str(" wrong"),
                                _ => {}
                            }
                        }
                        view! {
                            <button class=class on:click=move |_| go_to(p)>
                                {(p + 1).to_string()}
                            </button>
                        }
                    })
                    .collect_view(),
            )
        })
    };

    view! {
        <section class="runner">
            <div class="run-bar card">
                <div class="run-title">
                    <strong>{title}</strong>
                    <span class="muted small">
                        {move || {
                            let (n, total) = progress();
                            i18n.lang().answered_of(n, total)
                        }}
                    </span>
                </div>
                <div class="progress" aria-hidden="true">
                    <div
                        class="progress-fill"
                        style:width=move || {
                            let (n, total) = progress();
                            format!("{}%", n * 100 / total)
                        }
                    ></div>
                </div>
                <div class="run-actions">
                    {move || {
                        timer()
                            .map(|(label, value, urgent)| {
                                view! {
                                    <span class="timer" class:urgent=urgent title=label>
                                        "⏱ "
                                        {value}
                                    </span>
                                }
                            })
                    }}
                    <button class="btn small map-toggle" on:click=move |_| map_open.update(|o| *o = !*o)>
                        "▦"
                        <span class="btn-label">{move || i18n.t().question_map}</span>
                    </button>
                    <button class="btn small primary" on:click=move |_| request_finish()>
                        {move || i18n.t().finish}
                    </button>
                </div>
            </div>

            <div class="run-layout">
                <article class="card question-card">
                    {card}
                    <Show when=move || app.grade_failed.get()>
                        <div class="alert error">{move || i18n.t().grading_failed}</div>
                    </Show>
                </article>
                <aside class="qmap card" class:open=move || map_open.get()>
                    <div class="qmap-head">
                        <strong>{move || i18n.t().question_map}</strong>
                        <button class="icon-btn map-close" on:click=move |_| map_open.set(false)>
                            "✕"
                        </button>
                    </div>
                    <div class="qmap-grid">{map}</div>
                    <div class="legend small muted">
                        <span>
                            <i class="dot answered"></i>
                            {move || i18n.t().legend_answered}
                        </span>
                        <span>
                            <i class="dot flagged"></i>
                            {move || i18n.t().legend_flagged}
                        </span>
                    </div>
                    <p class="small muted shortcuts">{move || i18n.t().shortcuts}</p>
                </aside>
            </div>

            <Show when=move || confirm.get()>
                <div class="modal-backdrop" on:click=move |_| confirm.set(false)>
                    <div
                        class="modal card"
                        role="dialog"
                        aria-modal="true"
                        on:click=|ev| ev.stop_propagation()
                    >
                        <h3>{move || i18n.t().confirm_title}</h3>
                        <p>
                            {move || {
                                let n = app
                                    .attempt
                                    .with(|a| a.as_ref().map_or(0, |a| a.unanswered()));
                                if n == 0 {
                                    i18n.t().confirm_all_answered.to_string()
                                } else {
                                    i18n.lang().unanswered_warning(n)
                                }
                            }}
                        </p>
                        <div class="row end">
                            <button class="btn ghost" on:click=move |_| confirm.set(false)>
                                {move || i18n.t().cancel}
                            </button>
                            <button
                                class="btn primary"
                                on:click=move |_| {
                                    confirm.set(false);
                                    app.finish();
                                }
                            >
                                {move || i18n.t().confirm_yes}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </section>
    }
    .into_any()
}
