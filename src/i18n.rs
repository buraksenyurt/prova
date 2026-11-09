//! A small, dependency-free translation layer (same approach as log-lens).
//!
//! - Fixed texts are fields of [`Texts`]; each language is one `const` value,
//!   so a missing translation is a compile error.
//! - Texts with values in them are methods on [`Lang`] with exhaustive matches.

use leptos::prelude::*;

use crate::convert::{ConvertError, Warning};
use crate::dto::SchemaError;
use crate::source::LoadError;
use crate::storage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Tr,
    En,
}

impl Lang {
    pub const ALL: [Lang; 2] = [Lang::Tr, Lang::En];

    pub fn code(self) -> &'static str {
        match self {
            Lang::Tr => "tr",
            Lang::En => "en",
        }
    }

    pub fn native_name(self) -> &'static str {
        match self {
            Lang::Tr => "Türkçe",
            Lang::En => "English",
        }
    }

    pub fn from_code(code: &str) -> Option<Lang> {
        let primary = code.split(['-', '_']).next()?.to_ascii_lowercase();
        Lang::ALL.into_iter().find(|l| l.code() == primary)
    }

    pub fn texts(self) -> &'static Texts {
        match self {
            Lang::Tr => &TR,
            Lang::En => &EN,
        }
    }

    // ---------- texts with values ----------

    pub fn question_of(self, n: usize, total: usize) -> String {
        match self {
            Lang::Tr => format!("Soru {n} / {total}"),
            Lang::En => format!("Question {n} of {total}"),
        }
    }

    pub fn answered_of(self, n: usize, total: usize) -> String {
        match self {
            Lang::Tr => format!("{n}/{total} cevaplandı"),
            Lang::En => format!("{n}/{total} answered"),
        }
    }

    pub fn question_count(self, n: usize) -> String {
        match self {
            Lang::Tr => format!("{n} soru"),
            Lang::En if n == 1 => "1 question".into(),
            Lang::En => format!("{n} questions"),
        }
    }

    pub fn minutes(self, n: u32) -> String {
        match self {
            Lang::Tr => format!("{n} dakika"),
            Lang::En if n == 1 => "1 minute".into(),
            Lang::En => format!("{n} minutes"),
        }
    }

    /// 754_000 ms -> "12:34", 3_754_000 -> "1:02:34"
    pub fn clock(self, ms: f64) -> String {
        let total = (ms / 1000.0).ceil().max(0.0) as u64;
        let (h, m, s) = (total / 3600, total / 60 % 60, total % 60);
        if h > 0 {
            format!("{h}:{m:02}:{s:02}")
        } else {
            format!("{m:02}:{s:02}")
        }
    }

    pub fn percent(self, v: f64) -> String {
        let n = v.round() as i64;
        match self {
            Lang::Tr => format!("%{n}"),
            Lang::En => format!("{n}%"),
        }
    }

    pub fn ratio(self, correct: usize, total: usize) -> String {
        match self {
            Lang::Tr => format!("{correct}/{total} doğru"),
            Lang::En => format!("{correct}/{total} correct"),
        }
    }

    pub fn unanswered_warning(self, n: usize) -> String {
        match self {
            Lang::Tr => format!(
                "Cevaplamadığın {n} soru var. Boş sorular yanlış sayılmaz ama puan da getirmez."
            ),
            Lang::En if n == 1 => "1 question is unanswered. Empty answers earn no points.".into(),
            Lang::En => format!("{n} questions are unanswered. Empty answers earn no points."),
        }
    }

    pub fn retry_missed(self, n: usize) -> String {
        match self {
            Lang::Tr => format!("Yanlış ve boşları tekrar çöz ({n})"),
            Lang::En => format!("Retry wrong and empty ({n})"),
        }
    }

    pub fn resume_detail(self, answered: usize, total: usize) -> String {
        match self {
            Lang::Tr => format!("{total} sorudan {answered} tanesini cevaplamıştın."),
            Lang::En => format!("You answered {answered} of {total} questions."),
        }
    }

    /// Local date and time of a timestamp, formatted by the browser.
    pub fn date_time(self, ms: f64) -> String {
        let d = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(ms));
        String::from(d.to_locale_string(self.code(), &wasm_bindgen::JsValue::UNDEFINED))
    }

    pub fn convert_error(self, e: &ConvertError) -> String {
        match (self, e) {
            (Lang::Tr, ConvertError::NoQuestions) => {
                "Markdown'da soru bulunamadı. Sorular \"## Soru 1\" biçiminde başlıklarla başlamalı.".into()
            }
            (Lang::En, ConvertError::NoQuestions) => {
                "No questions found. Each question needs a heading like \"## Question 1\".".into()
            }
            (Lang::Tr, ConvertError::NoOptions { question }) => {
                format!("Soru {question}: en az iki şık bulunamadı (\"- A) ...\" biçiminde).")
            }
            (Lang::En, ConvertError::NoOptions { question }) => {
                format!("Question {question}: needs at least two options (\"- A) ...\").")
            }
            (Lang::Tr, ConvertError::OptionOrder { question }) => {
                format!("Soru {question}: şıklar A'dan başlayıp sırayla gitmeli.")
            }
            (Lang::En, ConvertError::OptionOrder { question }) => {
                format!("Question {question}: options must start at A and be in order.")
            }
        }
    }

    pub fn convert_warning(self, w: &Warning) -> String {
        match (self, w) {
            (Lang::Tr, Warning::NoTitle) => "Başlık (# ...) yok; varsayılan başlık kullanıldı.".into(),
            (Lang::En, Warning::NoTitle) => "No title (# ...); a default title was used.".into(),
            (Lang::Tr, Warning::MissingAnswer { question }) => format!(
                "Soru {question} için cevap yok. Cevaplar ya tüm sorularda olmalı ya hiçbirinde; cevap anahtarı yok sayıldı."
            ),
            (Lang::En, Warning::MissingAnswer { question }) => format!(
                "Question {question} has no answer. Answers must cover all questions or none, so they were dropped."
            ),
            (Lang::Tr, Warning::AnswerNotAnOption { question, letter }) => {
                format!("Soru {question}: cevap anahtarındaki '{letter}' şıkkı yok.")
            }
            (Lang::En, Warning::AnswerNotAnOption { question, letter }) => {
                format!("Question {question}: the answer '{letter}' is not one of its options.")
            }
            (Lang::Tr, Warning::UnknownKeyEntry { question }) => {
                format!("Cevap anahtarında {question}. soru var ama böyle bir soru yok.")
            }
            (Lang::En, Warning::UnknownKeyEntry { question }) => {
                format!("The answer key lists question {question}, which does not exist.")
            }
            (Lang::Tr, Warning::IgnoredText { question }) => {
                format!("Soru {question}: şıklardan sonraki metin yok sayıldı (açıklama için '>' ile başlayın).")
            }
            (Lang::En, Warning::IgnoredText { question }) => {
                format!("Question {question}: text after the options was ignored (start explanations with '>').")
            }
            (Lang::Tr, Warning::RelativeImages) => {
                "Göreli görsel yolları var ama temel adres bilinmiyor; görseller görünmeyebilir. Dosyayı bir URL'den yükleyin.".into()
            }
            (Lang::En, Warning::RelativeImages) => {
                "Relative image paths but no base URL; images may not show. Load the file from a URL instead.".into()
            }
        }
    }

    pub fn last_score(self, percent: f64, attempts: usize) -> String {
        let p = self.percent(percent);
        match self {
            Lang::Tr => format!("Son: {p} · {attempts} deneme"),
            Lang::En if attempts == 1 => format!("Last: {p} · 1 attempt"),
            Lang::En => format!("Last: {p} · {attempts} attempts"),
        }
    }

    pub fn change(self, delta: f64) -> String {
        let n = delta.round() as i64;
        match n {
            0 => "±0".into(),
            n if n > 0 => format!("▲ {n}"),
            n => format!("▼ {}", -n),
        }
    }

    pub fn load_error(self, e: &LoadError) -> String {
        match (self, e) {
            (_, LoadError::Schema(s)) => self.schema_error(s),
            (_, LoadError::Convert(c)) => self.convert_error(c),
            (Lang::Tr, LoadError::Read(m)) => format!("Dosya okunamadı: {m}"),
            (Lang::En, LoadError::Read(m)) => format!("Could not read the file: {m}"),
            (Lang::Tr, LoadError::Network(_)) => {
                "Adrese ulaşılamadı. Adresin doğru olduğundan ve sunucunun CORS'a izin verdiğinden emin ol.".into()
            }
            (Lang::En, LoadError::Network(_)) => {
                "Could not reach the URL. Check the address and that the server allows CORS.".into()
            }
            (Lang::Tr, LoadError::Http(c)) => format!("Sunucu {c} hatası döndü."),
            (Lang::En, LoadError::Http(c)) => format!("The server answered with HTTP {c}."),
        }
    }

    pub fn schema_error(self, e: &SchemaError) -> String {
        use SchemaError::*;
        match (self, e) {
            (
                Lang::Tr,
                Json {
                    line,
                    column,
                    message,
                },
            ) => {
                format!("Geçersiz JSON (satır {line}, sütun {column}): {message}")
            }
            (
                Lang::En,
                Json {
                    line,
                    column,
                    message,
                },
            ) => {
                format!("Invalid JSON (line {line}, column {column}): {message}")
            }
            (Lang::Tr, UnsupportedVersion(v)) => {
                format!(
                    "Bu dosya şema sürümü {v} ile yazılmış; uygulama bu sürümü henüz tanımıyor."
                )
            }
            (Lang::En, UnsupportedVersion(v)) => {
                format!("The file uses schema version {v}, which this app does not know yet.")
            }
            (Lang::Tr, NoQuestions) => "Dosyada hiç soru yok.".into(),
            (Lang::En, NoQuestions) => "The file contains no questions.".into(),
            (
                Lang::Tr,
                EmptyField {
                    question: Some(q),
                    field,
                },
            ) => format!("'{q}' sorusunda '{field}' boş."),
            (
                Lang::En,
                EmptyField {
                    question: Some(q),
                    field,
                },
            ) => format!("Question '{q}': '{field}' is empty."),
            (
                Lang::Tr,
                EmptyField {
                    question: None,
                    field,
                },
            ) => format!("'{field}' alanı boş."),
            (
                Lang::En,
                EmptyField {
                    question: None,
                    field,
                },
            ) => format!("'{field}' is empty."),
            (Lang::Tr, DuplicateQuestion(q)) => {
                format!("'{q}' kimliği birden fazla soruda kullanılmış.")
            }
            (Lang::En, DuplicateQuestion(q)) => {
                format!("The id '{q}' is used by more than one question.")
            }
            (Lang::Tr, TooFewChoices(q)) => format!("'{q}' sorusunda en az iki şık olmalı."),
            (Lang::En, TooFewChoices(q)) => format!("Question '{q}' needs at least two options."),
            (Lang::Tr, DuplicateChoice { question, choice }) => {
                format!("'{question}' sorusunda '{choice}' şık kimliği tekrar ediyor.")
            }
            (Lang::En, DuplicateChoice { question, choice }) => {
                format!("Question '{question}': option id '{choice}' is repeated.")
            }
            (Lang::Tr, UnknownAnswer { question, answer }) => {
                format!("'{question}' sorusunun cevabı '{answer}', ama böyle bir şık yok.")
            }
            (Lang::En, UnknownAnswer { question, answer }) => {
                format!("Question '{question}': the answer '{answer}' is not one of its options.")
            }
            (Lang::Tr, PartialAnswers { missing }) => {
                format!("Bazı soruların cevabı var, bazılarının yok (ilk eksik: '{missing}').")
            }
            (Lang::En, PartialAnswers { missing }) => {
                format!("Some questions have answers and some do not (first missing: '{missing}').")
            }
        }
    }
}

pub struct Texts {
    pub tagline: &'static str,
    pub language: &'static str,
    pub theme_dark: &'static str,
    pub theme_light: &'static str,
    // home
    pub home_title: &'static str,
    pub home_lead: &'static str,
    pub drop_title: &'static str,
    pub drop_hint: &'static str,
    pub or: &'static str,
    pub url_label: &'static str,
    pub url_placeholder: &'static str,
    pub url_button: &'static str,
    pub sample_button: &'static str,
    pub loading: &'static str,
    pub resume_title: &'static str,
    pub resume_button: &'static str,
    pub discard_button: &'static str,
    pub privacy_note: &'static str,
    // intro
    pub change_quiz: &'static str,
    pub source: &'static str,
    pub topics: &'static str,
    pub time_limit: &'static str,
    pub mode_title: &'static str,
    pub mode_practice: &'static str,
    pub mode_practice_desc: &'static str,
    pub mode_exam: &'static str,
    pub mode_exam_desc: &'static str,
    pub shuffle_questions: &'static str,
    pub shuffle_choices: &'static str,
    pub time_limit_input: &'static str,
    pub no_limit_hint: &'static str,
    pub start: &'static str,
    pub history_title: &'static str,
    pub no_key: &'static str,
    // runner
    pub prev: &'static str,
    pub next: &'static str,
    pub flag: &'static str,
    pub unflag: &'static str,
    pub clear_answer: &'static str,
    pub strike_hint: &'static str,
    pub finish: &'static str,
    pub see_results: &'static str,
    pub question_map: &'static str,
    pub time_left: &'static str,
    pub elapsed: &'static str,
    pub correct: &'static str,
    pub wrong: &'static str,
    pub explanation: &'static str,
    pub no_explanation: &'static str,
    pub grading_failed: &'static str,
    pub confirm_title: &'static str,
    pub confirm_all_answered: &'static str,
    pub confirm_yes: &'static str,
    pub cancel: &'static str,
    pub close: &'static str,
    pub shortcuts: &'static str,
    pub legend_answered: &'static str,
    pub legend_flagged: &'static str,
    pub time_up: &'static str,
    // results
    pub results_title: &'static str,
    pub empty: &'static str,
    pub duration: &'static str,
    pub by_topic: &'static str,
    pub review_title: &'static str,
    pub filter_all: &'static str,
    pub filter_wrong: &'static str,
    pub filter_empty: &'static str,
    pub filter_flagged: &'static str,
    pub your_answer: &'static str,
    pub correct_answer: &'static str,
    pub not_answered: &'static str,
    pub nothing_here: &'static str,
    pub restart: &'static str,
    pub new_quiz: &'static str,
    pub perfect: &'static str,
    pub converted_note: &'static str,
    pub download_json: &'static str,
    pub warnings_title: &'static str,
    pub progress_title: &'static str,
    pub progress_hint: &'static str,
    pub retry_tag: &'static str,
    pub vs_previous: &'static str,
    pub catalog_title: &'static str,
    pub own_file_title: &'static str,
}

const TR: Texts = Texts {
    tagline: "Sınavdan önce prova",
    language: "Dil",
    theme_dark: "Koyu temaya geç",
    theme_light: "Açık temaya geç",
    home_title: "Sınava hazır mısın?",
    home_lead: "Ders sınavlarından birini seç ya da kendi test dosyanı yükle; ister antrenman yap, ister gerçek sınav gibi süre tutarak çöz.",
    drop_title: "Test dosyasını buraya bırak",
    drop_hint: "ya da tıklayıp seç (.json veya sınav .md dosyası)",
    or: "veya",
    url_label: "Test adresi",
    url_placeholder: "https://.../exam.json",
    url_button: "Yükle",
    sample_button: "Örnek testi dene",
    loading: "Yükleniyor…",
    resume_title: "Yarım kalan bir denemen var",
    resume_button: "Kaldığın yerden devam et",
    discard_button: "Sil",
    privacy_note: "Her şey tarayıcında çalışır; cevapların hiçbir sunucuya gönderilmez.",
    change_quiz: "Başka test seç",
    source: "Kaynak",
    topics: "Konular",
    time_limit: "Süre",
    mode_title: "Nasıl çözmek istersin?",
    mode_practice: "Antrenman",
    mode_practice_desc: "Her cevaptan sonra doğrusunu ve açıklamasını gör. Süre baskısı yok.",
    mode_exam: "Sınav simülasyonu",
    mode_exam_desc: "Süre işler, cevaplarını değiştirebilirsin, sonuçları en sonda görürsün.",
    shuffle_questions: "Soruları karıştır",
    shuffle_choices: "Şıkları karıştır",
    time_limit_input: "Süre (dakika)",
    no_limit_hint: "0 = süresiz",
    start: "Başla",
    history_title: "Önceki denemelerin",
    no_key: "Bu dosyada cevap anahtarı yok; uygulama bu testi şimdilik değerlendiremez.",
    prev: "Önceki",
    next: "Sonraki",
    flag: "Sonra bak",
    unflag: "İşareti kaldır",
    clear_answer: "Cevabı temizle",
    strike_hint: "Şıkkın üstünü çizmek için sağ tık / uzun bas",
    finish: "Bitir",
    see_results: "Sonuçları gör",
    question_map: "Sorular",
    time_left: "Kalan süre",
    elapsed: "Geçen süre",
    correct: "Doğru",
    wrong: "Yanlış",
    explanation: "Açıklama",
    no_explanation: "Bu soru için açıklama eklenmemiş.",
    grading_failed: "Bu soru değerlendirilemedi.",
    confirm_title: "Testi bitirmek istiyor musun?",
    confirm_all_answered: "Tüm soruları cevapladın.",
    confirm_yes: "Evet, bitir",
    cancel: "Vazgeç",
    close: "Kapat",
    shortcuts: "Kısayollar: A–E veya 1–5 seç · ←/→ gezin · F işaretle",
    legend_answered: "Cevaplandı",
    legend_flagged: "İşaretli",
    time_up: "Süre doldu.",
    results_title: "Sonuçlar",
    empty: "Boş",
    duration: "Süre",
    by_topic: "Konulara göre",
    review_title: "Soruları gözden geçir",
    filter_all: "Tümü",
    filter_wrong: "Yanlışlar",
    filter_empty: "Boşlar",
    filter_flagged: "İşaretliler",
    your_answer: "Senin cevabın",
    correct_answer: "Doğru cevap",
    not_answered: "Boş bıraktın",
    nothing_here: "Bu filtrede soru yok.",
    restart: "Baştan çöz",
    new_quiz: "Başka test",
    perfect: "Hepsi doğru, tebrikler!",
    converted_note: "Markdown'dan dönüştürüldü",
    download_json: "JSON olarak indir",
    warnings_title: "Dönüştürme uyarıları",
    progress_title: "Konulara göre gelişim",
    progress_hint: "Son denemelerin, eskiden yeniye. Tekrar çözümleri dahil değil.",
    retry_tag: "tekrar",
    vs_previous: "Önceki denemeye göre",
    catalog_title: "Ders sınavları",
    own_file_title: "Kendi test dosyan",
};

const EN: Texts = Texts {
    tagline: "Rehearse before the exam",
    language: "Language",
    theme_dark: "Switch to dark theme",
    theme_light: "Switch to light theme",
    home_title: "Ready for the exam?",
    home_lead: "Pick one of the course exams or load your own test file, then practice freely or solve it against the clock like the real exam.",
    drop_title: "Drop the test file here",
    drop_hint: "or click to choose (.json or an exam .md file)",
    or: "or",
    url_label: "Test URL",
    url_placeholder: "https://.../exam.json",
    url_button: "Load",
    sample_button: "Try the sample test",
    loading: "Loading…",
    resume_title: "You have an unfinished attempt",
    resume_button: "Continue where you left off",
    discard_button: "Discard",
    privacy_note: "Everything runs in your browser; your answers are never sent to a server.",
    change_quiz: "Choose another test",
    source: "Source",
    topics: "Topics",
    time_limit: "Time",
    mode_title: "How do you want to solve it?",
    mode_practice: "Practice",
    mode_practice_desc: "See the correct answer and explanation after each question. No time pressure.",
    mode_exam: "Exam simulation",
    mode_exam_desc: "The clock runs, you can change answers, results come at the end.",
    shuffle_questions: "Shuffle questions",
    shuffle_choices: "Shuffle options",
    time_limit_input: "Time (minutes)",
    no_limit_hint: "0 = no limit",
    start: "Start",
    history_title: "Your previous attempts",
    no_key: "This file has no answer key; the app cannot grade it yet.",
    prev: "Previous",
    next: "Next",
    flag: "Review later",
    unflag: "Remove flag",
    clear_answer: "Clear answer",
    strike_hint: "Right-click / long-press an option to strike it out",
    finish: "Finish",
    see_results: "See results",
    question_map: "Questions",
    time_left: "Time left",
    elapsed: "Elapsed",
    correct: "Correct",
    wrong: "Wrong",
    explanation: "Explanation",
    no_explanation: "No explanation was added for this question.",
    grading_failed: "This question could not be graded.",
    confirm_title: "Finish the test?",
    confirm_all_answered: "You answered every question.",
    confirm_yes: "Yes, finish",
    cancel: "Cancel",
    close: "Close",
    shortcuts: "Shortcuts: A–E or 1–5 select · ←/→ navigate · F flag",
    legend_answered: "Answered",
    legend_flagged: "Flagged",
    time_up: "Time is up.",
    results_title: "Results",
    empty: "Empty",
    duration: "Time",
    by_topic: "By topic",
    review_title: "Review the questions",
    filter_all: "All",
    filter_wrong: "Wrong",
    filter_empty: "Empty",
    filter_flagged: "Flagged",
    your_answer: "Your answer",
    correct_answer: "Correct answer",
    not_answered: "Left empty",
    nothing_here: "No questions in this filter.",
    restart: "Start over",
    new_quiz: "Another test",
    perfect: "All correct, well done!",
    converted_note: "Converted from Markdown",
    download_json: "Download as JSON",
    warnings_title: "Conversion warnings",
    progress_title: "Progress by topic",
    progress_hint: "Your recent attempts, oldest to newest. Retries are not included.",
    retry_tag: "retry",
    vs_previous: "Compared to the previous attempt",
    catalog_title: "Course exams",
    own_file_title: "Your own test file",
};

/// Selected language, shared through context.
#[derive(Clone, Copy)]
pub struct I18n {
    pub lang: RwSignal<Lang>,
}

impl Default for I18n {
    fn default() -> Self {
        Self::new()
    }
}

impl I18n {
    /// Saved choice, else the browser language, else Turkish.
    pub fn new() -> Self {
        let lang = storage::get(storage::LANG)
            .and_then(|c| Lang::from_code(&c))
            .or_else(|| {
                web_sys::window()
                    .and_then(|w| w.navigator().language())
                    .and_then(|c| Lang::from_code(&c))
            })
            .unwrap_or(Lang::Tr);
        let lang = RwSignal::new(lang);
        Effect::new(move |_| {
            let l = lang.get();
            storage::set(storage::LANG, l.code());
            if let Some(el) = document().document_element() {
                let _ = el.set_attribute("lang", l.code());
            }
        });
        I18n { lang }
    }

    pub fn lang(&self) -> Lang {
        self.lang.get()
    }

    pub fn t(&self) -> &'static Texts {
        self.lang.get().texts()
    }
}

pub fn use_i18n() -> I18n {
    expect_context::<I18n>()
}

#[component]
pub fn LanguageSelect() -> impl IntoView {
    let i18n = use_i18n();
    view! {
        <select
            class="lang-select"
            aria-label=move || i18n.t().language
            on:change=move |ev| {
                if let Some(lang) = Lang::from_code(&event_target_value(&ev)) {
                    i18n.lang.set(lang);
                }
            }
        >
            {Lang::ALL
                .into_iter()
                .map(|l| {
                    view! {
                        <option value=l.code() selected=move || i18n.lang() == l>
                            {l.native_name()}
                        </option>
                    }
                })
                .collect_view()}
        </select>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes() {
        assert_eq!(Lang::from_code("tr-TR"), Some(Lang::Tr));
        assert_eq!(Lang::from_code("en_US"), Some(Lang::En));
        assert_eq!(Lang::from_code("de"), None);
    }

    #[test]
    fn formats() {
        assert_eq!(Lang::Tr.clock(754_000.0), "12:34");
        assert_eq!(Lang::En.clock(3_754_000.0), "1:02:34");
        assert_eq!(Lang::En.clock(500.0), "00:01");
        assert_eq!(Lang::Tr.percent(66.6), "%67");
        assert_eq!(Lang::En.percent(66.6), "67%");
        assert_eq!(Lang::En.question_count(1), "1 question");
        assert_eq!(Lang::Tr.question_of(3, 10), "Soru 3 / 10");
    }

    #[test]
    fn errors_are_translated() {
        let e = SchemaError::TooFewChoices("q1".into());
        assert!(Lang::Tr.schema_error(&e).contains("q1"));
        assert!(Lang::En.load_error(&LoadError::Http(404)).contains("404"));
    }
}
