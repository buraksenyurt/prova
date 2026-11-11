use leptos::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{DragEvent, File, HtmlInputElement};

use super::catalog::ExamBrowser;
use crate::app::{sample_source, use_app};
use crate::domain::Attempt;
use crate::i18n::use_i18n;
use crate::source::{FileSource, UrlSource};
use crate::storage::{self, SavedQuiz};

/// Unfinished attempt saved in the browser, together with its quiz.
pub fn resumable() -> Option<(SavedQuiz, Attempt)> {
    let attempt = storage::load_attempt().filter(|a| !a.is_finished())?;
    let quiz = storage::load_quiz()?;
    Some((quiz, attempt))
}

#[component]
pub fn Home() -> impl IntoView {
    let i18n = use_i18n();
    let app = use_app();
    let resume = RwSignal::new(resumable());
    let url = RwSignal::new(String::new());

    let on_file = Callback::new(move |file: File| app.load(FileSource { file }));
    let submit_url = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let u = url.get_untracked().trim().to_string();
        if !u.is_empty() {
            app.load(UrlSource { url: u });
        }
    };

    view! {
        <section class="home">
            <div class="hero">
                <h1>{move || i18n.t().home_title}</h1>
                <p class="lead">{move || i18n.t().home_lead}</p>
            </div>

            {move || {
                resume
                    .get()
                    .map(|(saved, attempt)| {
                        let detail = i18n.lang().resume_detail(attempt.answered(), attempt.len());
                        let label = saved.label.clone();
                        let (s, a) = (saved.clone(), attempt.clone());
                        view! {
                            <div class="card resume-card">
                                <div>
                                    <strong>{move || i18n.t().resume_title}</strong>
                                    <p class="muted">{label} " · " {detail}</p>
                                </div>
                                <div class="row">
                                    <button
                                        class="btn ghost"
                                        on:click=move |_| {
                                            storage::save_attempt(None);
                                            resume.set(None);
                                        }
                                    >
                                        {move || i18n.t().discard_button}
                                    </button>
                                    <button
                                        class="btn primary"
                                        on:click=move |_| app.resume(s.clone(), a.clone())
                                    >
                                        {move || i18n.t().resume_button}
                                    </button>
                                </div>
                            </div>
                        }
                    })
            }}

            {move || {
                app.load_error
                    .get()
                    .map(|e| {
                        view! {
                            <div class="alert error" role="alert">
                                {i18n.lang().load_error(&e)}
                            </div>
                        }
                    })
            }}
            <Show when=move || app.loading.get()>
                <p class="muted center">{move || i18n.t().loading}</p>
            </Show>

            <ExamBrowser />

            <details
                class="own-file"
                open=move || app.catalog.with(|c| c.as_ref().is_some_and(|c| c.is_empty()))
            >
                <summary>
                    <span>{move || i18n.t().own_file_title}</span>
                    <small class="muted">{move || i18n.t().own_file_hint}</small>
                </summary>
                <div class="card loader-card" class:busy=move || app.loading.get()>
                    <FileDrop on_file />
                    <div class="divider">
                        <span>{move || i18n.t().or}</span>
                    </div>
                    <form class="url-form" on:submit=submit_url>
                        <label class="visually-hidden" for="quiz-url">
                            {move || i18n.t().url_label}
                        </label>
                        <input
                            id="quiz-url"
                            type="url"
                            placeholder=move || i18n.t().url_placeholder
                            prop:value=move || url.get()
                            on:input=move |ev| url.set(event_target_value(&ev))
                        />
                        <button class="btn" type="submit">
                            {move || i18n.t().url_button}
                        </button>
                    </form>
                    <button class="btn link sample-btn" on:click=move |_| app.load(sample_source())>
                        {move || i18n.t().sample_button}
                        " →"
                    </button>
                </div>
            </details>
            <p class="muted center small">{move || i18n.t().privacy_note}</p>
        </section>
    }
}

/// File picker and drop area.
#[component]
fn FileDrop(on_file: Callback<File>) -> impl IntoView {
    let i18n = use_i18n();
    let dragging = RwSignal::new(false);

    let on_change = move |ev: leptos::ev::Event| {
        let input: HtmlInputElement = ev.target().unwrap().unchecked_into();
        if let Some(file) = input.files().and_then(|l| l.get(0)) {
            on_file.run(file);
        }
        // Picking the same file again must fire `change` again.
        input.set_value("");
    };

    let on_drop = move |ev: DragEvent| {
        ev.prevent_default();
        dragging.set(false);
        if let Some(file) = ev
            .data_transfer()
            .and_then(|dt| dt.files())
            .and_then(|l| l.get(0))
        {
            on_file.run(file);
        }
    };

    view! {
        <label
            class="drop-zone"
            class:dragging=move || dragging.get()
            on:dragover=move |ev: DragEvent| {
                ev.prevent_default();
                dragging.set(true);
            }
            on:dragleave=move |_| dragging.set(false)
            on:drop=on_drop
        >
            <input type="file" accept=".json,.md,.markdown,application/json,text/markdown" on:change=on_change />
            <span class="drop-icon" aria-hidden="true">"⇪"</span>
            <strong>{move || i18n.t().drop_title}</strong>
            <span class="muted">{move || i18n.t().drop_hint}</span>
        </label>
    }
}
