//! Phase 1 sources, built on browser APIs.

use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{File, Response};

use super::{LoadError, LoadedQuiz, LocalFuture, Origin, QuizSource, from_text, raw_github_url};

fn describe(e: JsValue) -> String {
    e.as_string()
        .or_else(|| {
            e.dyn_ref::<js_sys::Error>()
                .map(|e| String::from(e.message()))
        })
        .unwrap_or_else(|| format!("{e:?}"))
}

/// JSON text already in memory: the built-in sample or the saved copy.
pub struct TextSource {
    pub text: String,
    pub origin: Origin,
}

impl QuizSource for TextSource {
    fn load(&self) -> LocalFuture<Result<LoadedQuiz, LoadError>> {
        let result = from_text(self.text.clone(), self.origin.clone());
        Box::pin(std::future::ready(result))
    }
}

/// A file picked or dropped by the student.
pub struct FileSource {
    pub file: File,
}

impl QuizSource for FileSource {
    fn load(&self) -> LocalFuture<Result<LoadedQuiz, LoadError>> {
        let file = self.file.clone();
        Box::pin(async move {
            let text = JsFuture::from(file.text())
                .await
                .map_err(|e| LoadError::Read(describe(e)))?;
            let text = text
                .as_string()
                .ok_or_else(|| LoadError::Read("not text".into()))?;
            from_text(text, Origin::File(file.name()))
        })
    }
}

/// A quiz published somewhere (raw GitHub, gist, Pages ...). The server must allow CORS.
pub struct UrlSource {
    pub url: String,
}

impl QuizSource for UrlSource {
    fn load(&self) -> LocalFuture<Result<LoadedQuiz, LoadError>> {
        let url = raw_github_url(self.url.trim());
        Box::pin(async move {
            let window = web_sys::window().ok_or_else(|| LoadError::Network("no window".into()))?;
            let resp = JsFuture::from(window.fetch_with_str(&url))
                .await
                .map_err(|e| LoadError::Network(describe(e)))?;
            let resp: Response = resp
                .dyn_into()
                .map_err(|e| LoadError::Network(describe(e)))?;
            if !resp.ok() {
                return Err(LoadError::Http(resp.status()));
            }
            let text = JsFuture::from(resp.text().map_err(|e| LoadError::Network(describe(e)))?)
                .await
                .map_err(|e| LoadError::Network(describe(e)))?;
            let text = text.as_string().unwrap_or_default();
            from_text(text, Origin::Url(url))
        })
    }
}
