//! Pieces of a question shared by the runner and the results review.

use leptos::prelude::*;

use super::md;
use crate::domain::{Quiz, Slot};
use crate::i18n::I18n;
use crate::markdown::RenderOptions;

pub fn letter(display_index: usize) -> char {
    (b'A' + display_index as u8) as char
}

fn opts(quiz: &Quiz, highlight_negatives: bool) -> RenderOptions<'_> {
    RenderOptions {
        assets_base_url: quiz.assets_base_url.as_deref(),
        highlight_negatives,
    }
}

pub fn stem(quiz: &Quiz, slot: &Slot) -> impl IntoView + use<> {
    md(
        "stem",
        &quiz.questions[slot.question].stem,
        opts(quiz, true),
    )
}

pub fn topics(quiz: &Quiz, slot: &Slot) -> impl IntoView + use<> {
    quiz.questions[slot.question]
        .topics
        .iter()
        .map(|t| view! { <span class="chip small">{t.clone()}</span> })
        .collect_view()
}

/// How the choice list behaves.
pub struct ChoiceMode {
    /// Show correct/wrong colors (verdict known and allowed to be shown).
    pub reveal: bool,
    pub locked: bool,
    pub on_pick: Option<Callback<usize>>,
    pub on_strike: Option<Callback<usize>>,
}

pub fn choices(quiz: &Quiz, slot: &Slot, mode: ChoiceMode, i18n: I18n) -> impl IntoView + use<> {
    let question = &quiz.questions[slot.question];
    let correct = slot
        .verdict
        .as_ref()
        .filter(|_| mode.reveal)
        .and_then(|v| question.choice_index(&v.correct_choice));

    let items = slot
        .choice_order
        .iter()
        .enumerate()
        .map(|(k, &ci)| {
            let selected = slot.answer == Some(ci);
            let struck = slot.eliminated.contains(&ci);
            let is_correct = correct == Some(ci);
            let is_wrong = correct.is_some() && selected && !is_correct;
            let mut class = String::from("choice");
            for (on, name) in [
                (selected, " selected"),
                (struck, " struck"),
                (is_correct, " correct"),
                (is_wrong, " wrong"),
                (mode.locked, " locked"),
            ] {
                if on {
                    class.push_str(name);
                }
            }
            let mark = if is_correct {
                "✓"
            } else if is_wrong {
                "✗"
            } else {
                ""
            };
            let on_pick = mode.on_pick;
            let on_strike = mode.on_strike;
            let locked = mode.locked;
            let text = md("choice-text", &question.choices[ci].text, opts(quiz, false));
            view! {
                <li class=class>
                    <button
                        class="choice-main"
                        disabled=locked || on_pick.is_none()
                        aria-pressed=selected.to_string()
                        on:click=move |_| {
                            if let Some(cb) = on_pick {
                                cb.run(ci)
                            }
                        }
                        on:contextmenu=move |ev: leptos::ev::MouseEvent| {
                            if let Some(cb) = on_strike {
                                ev.prevent_default();
                                cb.run(ci);
                            }
                        }
                    >
                        <span class="letter">{letter(k).to_string()}</span>
                        {text}
                        <span class="mark" aria-hidden="true">{mark}</span>
                    </button>
                    {(on_strike.is_some() && !locked && !selected)
                        .then(|| {
                            view! {
                                <button
                                    class="strike-btn"
                                    title=move || i18n.t().strike_hint
                                    aria-label=move || i18n.t().strike_hint
                                    on:click=move |_| {
                                        if let Some(cb) = on_strike {
                                            cb.run(ci)
                                        }
                                    }
                                >
                                    {if struck { "↺" } else { "⊘" }}
                                </button>
                            }
                        })}
                </li>
            }
        })
        .collect_view();
    view! { <ol class="choices">{items}</ol> }
}

/// Verdict banner and explanation (practice mode and the review list).
pub fn feedback(quiz: &Quiz, slot: &Slot, i18n: I18n) -> Option<impl IntoView + use<>> {
    let verdict = slot.verdict.clone()?;
    let question = &quiz.questions[slot.question];
    let correct_index = question.choice_index(&verdict.correct_choice);
    let ok = slot.answer.is_some() && slot.answer == correct_index;
    let empty = slot.answer.is_none();
    let correct_letter = correct_index
        .and_then(|ci| slot.choice_order.iter().position(|&c| c == ci))
        .map(letter);
    let explanation = verdict
        .explanation
        .as_deref()
        .map(|e| md("explanation-text", e, opts(quiz, false)));

    Some(view! {
        <div class="feedback" class:ok=ok class:bad=!ok>
            <div class="feedback-head">
                <strong>
                    {move || {
                        if ok {
                            format!("✓ {}", i18n.t().correct)
                        } else if empty {
                            i18n.t().not_answered.to_string()
                        } else {
                            format!("✗ {}", i18n.t().wrong)
                        }
                    }}
                </strong>
                {(!ok)
                    .then(|| {
                        correct_letter
                            .map(|l| {
                                view! {
                                    <span>
                                        {move || i18n.t().correct_answer} ": " <b>{l.to_string()}</b>
                                    </span>
                                }
                            })
                    })}
            </div>
            <div class="explanation">
                <span class="label">{move || i18n.t().explanation}</span>
                {match explanation {
                    Some(e) => e.into_any(),
                    None => view! { <p class="muted">{move || i18n.t().no_explanation}</p> }.into_any(),
                }}
            </div>
        </div>
    })
}
