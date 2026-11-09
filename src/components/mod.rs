pub mod header;
pub mod home;
pub mod intro;
pub mod question;
pub mod results;
pub mod runner;

use leptos::prelude::*;

use crate::markdown::{RenderOptions, render};

/// Rendered Markdown. The content is static for the lifetime of the element.
pub fn md(class: &'static str, text: &str, opts: RenderOptions) -> impl IntoView + use<> {
    let html = render(text, opts);
    view! { <div class=format!("md {class}") inner_html=html></div> }
}
