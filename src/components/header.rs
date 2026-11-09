use leptos::prelude::*;

use crate::app::{Screen, use_app};
use crate::i18n::{LanguageSelect, use_i18n};
use crate::storage;

#[component]
pub fn AppHeader() -> impl IntoView {
    let i18n = use_i18n();
    let app = use_app();
    view! {
        <header class="app-header">
            <div class="container header-inner">
                <button class="brand" on:click=move |_| app.go(Screen::Home) aria-label="Prova">
                    <span class="brand-mark" aria-hidden="true">"P"</span>
                    <span class="brand-text">
                        <strong>"prova"</strong>
                        <small>{move || i18n.t().tagline}</small>
                    </span>
                </button>
                <div class="header-tools">
                    <ThemeToggle />
                    <LanguageSelect />
                </div>
            </div>
        </header>
    }
}

fn prefers_dark() -> bool {
    web_sys::window()
        .and_then(|w| w.match_media("(prefers-color-scheme: dark)").ok().flatten())
        .is_some_and(|m| m.matches())
}

/// Light/dark switch. Without a saved choice the system setting is used.
#[component]
fn ThemeToggle() -> impl IntoView {
    let i18n = use_i18n();
    let dark = RwSignal::new(match storage::get(storage::THEME).as_deref() {
        Some("dark") => true,
        Some("light") => false,
        _ => prefers_dark(),
    });
    Effect::new(move |_| {
        if let Some(el) = document().document_element() {
            let _ = el.set_attribute("data-theme", if dark.get() { "dark" } else { "light" });
        }
    });
    let toggle = move |_| {
        dark.update(|d| *d = !*d);
        storage::set(
            storage::THEME,
            if dark.get_untracked() {
                "dark"
            } else {
                "light"
            },
        );
    };
    view! {
        <button
            class="icon-btn"
            on:click=toggle
            title=move || if dark.get() { i18n.t().theme_light } else { i18n.t().theme_dark }
            aria-label=move || if dark.get() { i18n.t().theme_light } else { i18n.t().theme_dark }
        >
            {move || if dark.get() { "☀" } else { "☾" }}
        </button>
    }
}
