pub mod catalog;
pub mod header;
pub mod home;
pub mod intro;
pub mod question;
pub mod results;
pub mod runner;

use leptos::prelude::*;

use wasm_bindgen::{JsCast, JsValue};

use crate::markdown::{RenderOptions, needs_enhance, render};

/// Rendered Markdown. The content is static for the lifetime of the element.
///
/// Blocks with code or math are handed to `window.provaEnhance`
/// (`assets/enhance.js`) for highlighting and KaTeX.
pub fn md(class: &'static str, text: &str, opts: RenderOptions) -> impl IntoView + use<> {
    let html = render(text, opts);
    let node = NodeRef::<leptos::html::Div>::new();
    if needs_enhance(&html) {
        node.on_load(|el| enhance(&el));
    }
    view! { <div class=format!("md {class}") inner_html=html node_ref=node></div> }
}

fn enhance(el: &JsValue) {
    let Some(window) = web_sys::window() else {
        return;
    };
    if let Ok(f) = js_sys::Reflect::get(&window, &JsValue::from_str("provaEnhance"))
        && let Some(f) = f.dyn_ref::<js_sys::Function>()
    {
        let _ = f.call1(&JsValue::NULL, el);
    }
}

/// Lets the browser save `text` as a file.
pub fn download_text(file_name: &str, text: &str, mime: &str) {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    let Ok(a) = doc.create_element("a") else {
        return;
    };
    let href = format!(
        "data:{mime};charset=utf-8,{}",
        js_sys::encode_uri_component(text)
    );
    let _ = a.set_attribute("href", &href);
    let _ = a.set_attribute("download", file_name);
    if let Ok(a) = a.dyn_into::<web_sys::HtmlElement>() {
        a.click();
    }
}
