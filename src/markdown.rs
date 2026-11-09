//! Markdown → HTML for stems, choices and explanations.
//!
//! - Raw HTML in the content is shown as text, not executed: a quiz may come
//!   from any URL, and `inner_html` must not become an XSS hole.
//! - Relative image paths are resolved against `assets_base_url`.
//! - `$...$` / `$$...$$` math becomes `.math` elements holding the TeX source;
//!   `assets/enhance.js` renders them with KaTeX (the source stays as fallback).
//! - Optionally, negative words in a stem ("değildir", "yanlıştır", "NOT" ...)
//!   are highlighted: students miss them all the time.

use pulldown_cmark::{CowStr, Event, Options, Parser, Tag, TagEnd, html};

const NEGATIVE_WORDS: [&str; 6] = [
    "değildir",
    "yanlıştır",
    "söylenemez",
    "olamaz",
    "beklenmez",
    "yapılamaz",
];
/// Only matched in capitals: "not" is far too common otherwise.
const NEGATIVE_WORDS_UPPER: [&str; 3] = ["NOT", "EXCEPT", "FALSE"];

#[derive(Debug, Clone, Copy, Default)]
pub struct RenderOptions<'a> {
    pub assets_base_url: Option<&'a str>,
    pub highlight_negatives: bool,
}

pub fn render(text: &str, opts: RenderOptions) -> String {
    let md = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_MATH
        | Options::ENABLE_TASKLISTS;
    let mut in_code = false;
    let events = Parser::new_ext(text, md).map(|ev| match ev {
        Event::Start(Tag::CodeBlock(kind)) => {
            in_code = true;
            Event::Start(Tag::CodeBlock(kind))
        }
        Event::End(TagEnd::CodeBlock) => {
            in_code = false;
            Event::End(TagEnd::CodeBlock)
        }
        Event::Html(s) | Event::InlineHtml(s) => Event::Text(s),
        Event::InlineMath(s) => {
            Event::Html(format!("<span class=\"math math-inline\">{}</span>", escape(&s)).into())
        }
        Event::DisplayMath(s) => {
            Event::Html(format!("<div class=\"math math-display\">{}</div>", escape(&s)).into())
        }
        Event::Start(Tag::Image {
            link_type,
            dest_url,
            title,
            id,
        }) => Event::Start(Tag::Image {
            link_type,
            dest_url: resolve(dest_url, opts.assets_base_url),
            title,
            id,
        }),
        Event::Text(s) if opts.highlight_negatives && !in_code => match highlight(&s) {
            Some(html) => Event::Html(html.into()),
            None => Event::Text(s),
        },
        other => other,
    });
    let mut out = String::with_capacity(text.len() * 3 / 2);
    html::push_html(&mut out, events);
    out
}

/// Whether rendered HTML has code blocks or math that `enhance.js` should process.
pub fn needs_enhance(html: &str) -> bool {
    html.contains("<code class=\"language-") || html.contains("class=\"math ")
}

fn resolve<'a>(url: CowStr<'a>, base: Option<&str>) -> CowStr<'a> {
    let absolute = url.starts_with("http://")
        || url.starts_with("https://")
        || url.starts_with("data:")
        || url.starts_with('/');
    match base {
        Some(base) if !absolute => {
            let rel = url.trim_start_matches("./");
            let sep = if base.ends_with('/') { "" } else { "/" };
            format!("{base}{sep}{rel}").into()
        }
        _ => url,
    }
}

fn is_negative(word: &str) -> bool {
    NEGATIVE_WORDS_UPPER.contains(&word) || NEGATIVE_WORDS.contains(&word.to_lowercase().as_str())
}

/// HTML with negative words wrapped in `<mark>`, or `None` if there are none.
fn highlight(text: &str) -> Option<String> {
    let mut out = String::new();
    let mut found = false;
    let mut word_start: Option<usize> = None;
    let mut last = 0;

    let mut flush = |start: usize, end: usize, out: &mut String, last: &mut usize| {
        let word = &text[start..end];
        if is_negative(word) {
            out.push_str(&escape(&text[*last..start]));
            out.push_str("<mark class=\"neg\">");
            out.push_str(&escape(word));
            out.push_str("</mark>");
            *last = end;
            found = true;
        }
    };

    for (i, c) in text.char_indices() {
        if c.is_alphabetic() {
            word_start.get_or_insert(i);
        } else if let Some(s) = word_start.take() {
            flush(s, i, &mut out, &mut last);
        }
    }
    if let Some(s) = word_start {
        flush(s, text.len(), &mut out, &mut last);
    }
    if !found {
        return None;
    }
    out.push_str(&escape(&text[last..]));
    Some(out)
}

fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(s: &str) -> String {
        render(s, RenderOptions::default())
    }

    #[test]
    fn raw_html_is_escaped() {
        let out = r("<script>alert(1)</script> and <b>x</b>");
        assert!(!out.contains("<script>"));
        assert!(out.contains("&lt;script&gt;"));
    }

    #[test]
    fn tables_and_code() {
        assert!(r("| a | b |\n|---|---|\n| 1 | 2 |").contains("<table>"));
        assert!(r("```csharp\nvar x = 1;\n```").contains("language-csharp"));
    }

    #[test]
    fn math_elements() {
        assert!(r("$a^2 < b$").contains("<span class=\"math math-inline\">a^2 &lt; b</span>"));
        assert!(r("$$x$$").contains("<div class=\"math math-display\">x</div>"));
        assert!(needs_enhance(&r("$x$")));
        assert!(needs_enhance(&r("```rust\nfn a() {}\n```")));
        assert!(!needs_enhance(&r("plain `code`")));
    }

    #[test]
    fn images_resolve_against_base() {
        let o = RenderOptions {
            assets_base_url: Some("https://x.io/repo"),
            highlight_negatives: false,
        };
        assert!(
            render("![a](./images/1.png)", o).contains("src=\"https://x.io/repo/images/1.png\"")
        );
        assert!(render("![a](https://y.io/2.png)", o).contains("src=\"https://y.io/2.png\""));
    }

    #[test]
    fn negatives_highlighted_outside_code() {
        let o = RenderOptions {
            assets_base_url: None,
            highlight_negatives: true,
        };
        let out = render("Hangisi **doğru değildir**? Which is NOT <x>", o);
        assert!(out.contains("<mark class=\"neg\">değildir</mark>"));
        assert!(out.contains("<mark class=\"neg\">NOT</mark>"));
        assert!(out.contains("&lt;x&gt;"));
        let code = render("```\nreturn değildir;\n```", o);
        assert!(!code.contains("<mark"));
        assert_eq!(highlight("is not here"), None);
    }
}
