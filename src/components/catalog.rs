//! Home screen exam browser: course tabs, search, status chips, tag/level
//! filters, sorting and compact exam cards. The logic lives in `catalog_view`.

use leptos::html::Input;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

use super::home::resumable;
use crate::app::{url_param, use_app};
use crate::catalog_view::{
    ExamCard, ExamStatus, InProgress, PASS_PERCENT, SortKey, StatusFilter, ViewState, apply,
    build_cards, courses, has_levels, tags, weak_topics,
};
use crate::domain::HistoryEntry;
use crate::i18n::use_i18n;
use crate::source::CatalogSource;
use crate::source::catalog::{CATALOG_URL, CatalogEntry, Level};
use crate::storage;

#[component]
pub fn ExamBrowser() -> impl IntoView {
    let app = use_app();
    move || {
        let list = app.catalog.get()?;
        (!list.is_empty()).then(|| view! { <Browser entries=list /> })
    }
}

/// A stable hue per course for the card stripe and tab marker.
fn hue(course_key: &str) -> u32 {
    course_key
        .bytes()
        .fold(7u32, |h, b| h.wrapping_mul(31).wrapping_add(b as u32))
        % 360
}

#[component]
fn Browser(entries: Vec<CatalogEntry>) -> impl IntoView {
    let i18n = use_i18n();

    // Everything below is read once per visit of the home screen.
    let in_progress = storage::load_attempt()
        .filter(|a| !a.is_finished())
        .map(|a| InProgress {
            quiz_id: a.quiz_id.clone(),
            answered: a.answered(),
            total: a.len(),
        });
    let histories: Vec<(String, Vec<HistoryEntry>)> = entries
        .iter()
        .map(|e| (e.id.clone(), storage::history(&e.id)))
        .collect();
    let cards = build_cards(
        &entries,
        |id| {
            histories
                .iter()
                .find(|(i, _)| i == id)
                .map(|(_, h)| h.clone())
                .unwrap_or_default()
        },
        in_progress.as_ref(),
    );
    let weak = weak_topics(&histories.into_iter().map(|(_, h)| h).collect::<Vec<_>>());
    let course_list = courses(&cards);
    let tag_list = tags(&cards);
    let show_levels = has_levels(&cards);

    // Saved selection, overridden by a `?course=` link.
    let mut initial = storage::load_view().unwrap_or_default();
    if let Some(c) = url_param("course") {
        initial.course = Some(c);
    }
    initial.sanitize(&cards);
    let state = RwSignal::new(initial);
    Effect::new(move |_| state.with(storage::save_view));

    let all_cards = StoredValue::new(cards);
    let weak = StoredValue::new(weak);
    let visible = Memo::new(move |_| {
        state.with(|s| all_cards.with_value(|c| weak.with_value(|w| apply(c, s, w))))
    });

    // "/" focuses the search box, Escape in the box clears it.
    let search = NodeRef::<Input>::new();
    let keys = window_event_listener(leptos::ev::keydown, move |ev: web_sys::KeyboardEvent| {
        if ev.key() != "/" || ev.ctrl_key() || ev.meta_key() || ev.alt_key() {
            return;
        }
        let typing = ev
            .target()
            .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
            .is_some_and(|el| matches!(el.tag_name().as_str(), "INPUT" | "SELECT" | "TEXTAREA"));
        if !typing && let Some(input) = search.get() {
            ev.prevent_default();
            let _ = input.focus();
        }
    });
    on_cleanup(move || keys.remove());

    let total = all_cards.with_value(|c| c.len());
    let tabs = (course_list.len() > 1).then(|| {
        let tab = move |key: Option<String>, label: String, count: usize| {
            let k = key.clone();
            let style = key
                .as_deref()
                .map(|k| format!("--hue: {}", hue(k)))
                .unwrap_or_default();
            view! {
                <button
                    class="course-tab"
                    class:with-hue=key.is_some()
                    style=style
                    role="tab"
                    aria-selected=move || state.with(|s| s.course == k).to_string()
                    on:click=move |_| state.update(|s| s.course = key.clone())
                >
                    <span class="tab-label">{label}</span>
                    <span class="count">{count}</span>
                </button>
            }
        };
        view! {
            <div class="course-tabs" role="tablist">
                {tab(None, i18n.t().all_courses.to_string(), total)}
                {course_list
                    .iter()
                    .map(|(key, name, n)| tab(Some(key.clone()), name.clone(), *n))
                    .collect_view()}
            </div>
        }
    });

    let status_chips = StatusFilter::ALL
        .into_iter()
        .map(|f| {
            view! {
                <button
                    class="chip-btn"
                    aria-pressed=move || state.with(|s| s.status == f).to_string()
                    on:click=move |_| state.update(|s| s.status = f)
                >
                    {move || i18n.lang().status_filter(f)}
                </button>
            }
        })
        .collect_view();

    let tag_select = (!tag_list.is_empty()).then(|| {
        view! {
            <select
                aria-label=move || i18n.t().tag_any
                on:change=move |ev| {
                    let v = event_target_value(&ev);
                    state.update(|s| s.tag = (!v.is_empty()).then_some(v));
                }
            >
                <option value="" selected=move || state.with(|s| s.tag.is_none())>
                    {move || i18n.t().tag_any}
                </option>
                {tag_list
                    .iter()
                    .map(|t| {
                        let (t1, t2) = (t.clone(), t.clone());
                        view! {
                            <option
                                value=t1
                                selected=move || state.with(|s| s.tag.as_deref() == Some(t2.as_str()))
                            >
                                {t.clone()}
                            </option>
                        }
                    })
                    .collect_view()}
            </select>
        }
    });

    let level_select = show_levels.then(|| {
        view! {
            <select
                aria-label=move || i18n.t().level_any
                on:change=move |ev| {
                    let v = event_target_value(&ev);
                    state.update(|s| s.level = (!v.is_empty()).then_some(v));
                }
            >
                <option value="" selected=move || state.with(|s| s.level.is_none())>
                    {move || i18n.t().level_any}
                </option>
                {Level::ALL
                    .into_iter()
                    .map(|l| {
                        view! {
                            <option
                                value=l.key()
                                selected=move || state.with(|s| s.level.as_deref() == Some(l.key()))
                            >
                                {move || i18n.lang().level(l)}
                            </option>
                        }
                    })
                    .collect_view()}
            </select>
        }
    });

    let sort_select = view! {
        <label class="sort">
            <span class="muted small">{move || i18n.t().sort_label}</span>
            <select on:change=move |ev| {
                let v = event_target_value(&ev);
                let key = SortKey::ALL.into_iter().find(|k| format!("{k:?}") == v).unwrap_or_default();
                state.update(|s| s.sort = key);
            }>
                {SortKey::ALL
                    .into_iter()
                    .map(|k| {
                        view! {
                            <option value=format!("{k:?}") selected=move || state.with(|s| s.sort == k)>
                                {move || i18n.lang().sort_name(k)}
                            </option>
                        }
                    })
                    .collect_view()}
            </select>
        </label>
    };

    view! {
        <section class="browser" aria-label=move || i18n.t().catalog_title>
            <h2 class="section-title">{move || i18n.t().catalog_title}</h2>
            {tabs}
            <div class="browser-tools">
                <input
                    type="search"
                    class="search"
                    node_ref=search
                    placeholder=move || i18n.t().search_placeholder
                    aria-label=move || i18n.t().search_placeholder
                    prop:value=move || state.with(|s| s.query.clone())
                    on:input=move |ev| state.update(|s| s.query = event_target_value(&ev))
                    on:keydown=move |ev: web_sys::KeyboardEvent| {
                        if ev.key() == "Escape" {
                            state.update(|s| s.query.clear());
                        }
                    }
                />
                <div class="filters">
                    <div class="chips" role="group" aria-label=move || i18n.t().status_label>
                        {status_chips}
                    </div>
                    <div class="selects">{tag_select} {level_select} {sort_select}</div>
                </div>
            </div>
            <div class="results-line">
                <span class="muted small">{move || i18n.lang().exam_count(visible.with(Vec::len))}</span>
                <Show when=move || state.with(ViewState::is_filtered)>
                    <button class="btn link small" on:click=move |_| state.update(ViewState::clear_filters)>
                        {move || i18n.t().clear_filters}
                    </button>
                </Show>
            </div>
            {move || {
                let list = visible.get();
                if list.is_empty() {
                    return view! {
                        <div class="card empty-state">
                            <p>{move || i18n.t().no_results}</p>
                            <button class="btn" on:click=move |_| state.update(ViewState::clear_filters)>
                                {move || i18n.t().clear_filters}
                            </button>
                        </div>
                    }
                        .into_any();
                }
                let show_course = state.with_untracked(|s| s.course.is_none()) && course_list_len_gt1(&list);
                view! {
                    <div class="exam-grid">
                        {list.into_iter().map(|c| view! { <ExamCardView card=c show_course /> }).collect_view()}
                    </div>
                }
                    .into_any()
            }}
        </section>
    }
}

/// Course names on cards only matter when several courses are mixed.
fn course_list_len_gt1(list: &[ExamCard]) -> bool {
    list.iter().any(|c| c.course_key != list[0].course_key)
}

#[component]
fn ExamCardView(card: ExamCard, show_course: bool) -> impl IntoView {
    let i18n = use_i18n();
    let app = use_app();
    let entry = card.entry.clone();
    let status = card.status;
    let attempts = card.attempts;
    let level = entry.level();
    let minutes = entry.minutes;
    let questions = entry.questions;
    let style = format!("--hue: {}", hue(&card.course_key));

    let open = {
        let entry = entry.clone();
        move |_| {
            // The unfinished attempt of this exam continues where it left off.
            if let Some((saved, attempt)) = resumable().filter(|(_, a)| a.quiz_id == entry.id) {
                app.resume(saved, attempt);
            } else {
                app.load(CatalogSource {
                    entry: entry.clone(),
                    catalog_url: CATALOG_URL.into(),
                });
            }
        }
    };

    let (status_class, low) = match status {
        ExamStatus::NotStarted => ("new", false),
        ExamStatus::InProgress { .. } => ("wip", false),
        ExamStatus::Completed { last } => ("done", last < PASS_PERCENT),
        ExamStatus::Practiced => ("practiced", false),
    };

    view! {
        <button class="card exam-card" style=style disabled=move || app.loading.get() on:click=open>
            {(show_course)
                .then(|| entry.course.clone().map(|c| view! { <span class="exam-course">{c}</span> }))}
            <strong class="exam-title">{entry.title.clone()}</strong>
            {entry.description.clone().map(|d| view! { <span class="muted exam-desc">{d}</span> })}
            <span class="exam-meta">
                {questions.map(|n| view! { <span class="pill">{move || i18n.lang().question_count(n)}</span> })}
                {minutes.map(|m| view! { <span class="pill subtle">"⏱ " {move || i18n.lang().minutes_short(m)}</span> })}
                {level.map(|l| view! { <span class=format!("pill level {}", l.key())>{move || i18n.lang().level(l)}</span> })}
                {entry
                    .tags
                    .iter()
                    .take(4)
                    .map(|t| view! { <span class="chip small">{t.clone()}</span> })
                    .collect_view()}
            </span>
            <span class="exam-foot">
                <span class=format!("exam-status {status_class}") class:low=low>
                    {move || i18n.lang().exam_status(status, attempts)}
                </span>
                <span class="exam-go">
                    {move || {
                        if matches!(status, ExamStatus::InProgress { .. }) {
                            i18n.t().continue_label
                        } else {
                            i18n.t().start
                        }
                    }}
                    " →"
                </span>
            </span>
        </button>
    }
}
