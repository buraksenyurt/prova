//! Exam Markdown → quiz JSON (`schema_version: 1`).
//!
//! The format the instructor already writes exams in:
//!
//! ```markdown
//! # Title
//! Optional description...
//!
//! ## Soru 1            (or "Question 1")
//! Stem in Markdown: code blocks, tables, images, "- I." statements...
//!
//! - A) First option
//! - B) Second option
//!
//! > Açıklama: optional explanation (a blockquote after the options)
//! <!-- topics: RAG, Test -->   optional
//! <!-- answer: B -->           optional, overrides the answer key
//! <!-- lock -->                optional, keeps option order (also detected)
//!
//! ## Cevap Anahtarı     (or "Answer Key")
//! | Soru No | Doğru Cevap |
//! |---|---|
//! | 1 | B |
//! ```
//!
//! Before the first question, `<!-- id: ... -->` and `<!-- time: 40 -->` set
//! the quiz id and the time limit. Used by the `md2quiz` CLI and by the app
//! when a `.md` file or URL is loaded.

use std::collections::BTreeMap;

use serde_json::{Map, Value, json};

#[derive(Debug, Clone, Default)]
pub struct ConvertOptions {
    /// Forces the quiz id (CLI `--id`, catalog entry id).
    pub id: Option<String>,
    /// Used only when the file has no `<!-- id: -->` (e.g. the file name).
    pub default_id: Option<String>,
    pub assets_base_url: Option<String>,
    pub time_limit_minutes: Option<u32>,
    pub shuffle_questions: bool,
    /// `None`: shuffle (the default for converted exams).
    pub shuffle_options: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Converted {
    pub json: String,
    pub questions: usize,
    pub answers: usize,
    pub warnings: Vec<Warning>,
}

/// Question numbers are the ones written in the headings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Warning {
    NoTitle,
    /// Some questions have answers, this one has none; answers were dropped.
    MissingAnswer {
        question: u32,
    },
    AnswerNotAnOption {
        question: u32,
        letter: char,
    },
    /// The answer key lists a question that does not exist.
    UnknownKeyEntry {
        question: u32,
    },
    /// Text after the options that is neither an explanation nor a comment.
    IgnoredText {
        question: u32,
    },
    /// Relative image paths but no base URL to resolve them.
    RelativeImages,
    /// The explanation refers to options by letter (**A**) but the options
    /// may be shuffled, so the letters would not match what the student sees.
    LettersInShuffledExplanation {
        question: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConvertError {
    NoQuestions,
    NoOptions { question: u32 },
    OptionOrder { question: u32 },
}

struct RawQuestion {
    number: u32,
    lines: Vec<String>,
}

#[derive(Default)]
struct Parsed {
    title: Option<String>,
    description: Vec<String>,
    meta: BTreeMap<String, String>,
    questions: Vec<RawQuestion>,
    key: BTreeMap<u32, char>,
}

enum Section {
    Preamble,
    Question,
    Key,
    Other,
}

fn is_fence(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("```") || t.starts_with("~~~")
}

/// `<!-- key: value -->` or `<!-- key -->`
fn meta_comment(line: &str) -> Option<(String, String)> {
    let inner = line
        .trim()
        .strip_prefix("<!--")?
        .strip_suffix("-->")?
        .trim();
    let (k, v) = inner.split_once(':').unwrap_or((inner, ""));
    let k = k.trim().to_lowercase();
    if k.is_empty() || k.contains(' ') {
        return None;
    }
    Some((k, v.trim().to_string()))
}

fn heading(line: &str) -> Option<(usize, &str)> {
    let level = line.chars().take_while(|&c| c == '#').count();
    if (1..=6).contains(&level) && line[level..].starts_with(' ') {
        Some((level, line[level..].trim()))
    } else {
        None
    }
}

fn question_number(title: &str) -> Option<Option<u32>> {
    let lower = title.to_lowercase();
    let rest = lower
        .strip_prefix("soru")
        .or_else(|| lower.strip_prefix("question"))?;
    if rest.chars().next().is_some_and(|c| c.is_alphabetic()) {
        return None; // "Sorular", "Questions"
    }
    let digits: String = rest
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(char::is_ascii_digit)
        .collect();
    Some(digits.parse().ok())
}

fn is_key_heading(title: &str) -> bool {
    let lower = title.to_lowercase();
    lower.contains("cevap anahtar") || lower.contains("answer key")
}

fn split_sections(text: &str) -> Parsed {
    let mut p = Parsed::default();
    let mut section = Section::Preamble;
    let mut in_fence = false;
    let mut in_comment = false;

    for line in text.lines() {
        // Multi-line HTML comments are notes for the author; drop them.
        if in_comment {
            in_comment = !line.contains("-->");
            continue;
        }
        if !in_fence && line.trim_start().starts_with("<!--") && !line.contains("-->") {
            in_comment = true;
            continue;
        }
        if is_fence(line) {
            in_fence = !in_fence;
        }
        if !in_fence && let Some((level, title)) = heading(line) {
            if level == 1 && p.title.is_none() && matches!(section, Section::Preamble) {
                p.title = Some(title.to_string());
                continue;
            }
            if level <= 2 {
                if let Some(n) = question_number(title) {
                    let number = n.unwrap_or(p.questions.len() as u32 + 1);
                    p.questions.push(RawQuestion {
                        number,
                        lines: Vec::new(),
                    });
                    section = Section::Question;
                    continue;
                }
                if is_key_heading(title) {
                    section = Section::Key;
                    continue;
                }
                if !matches!(section, Section::Preamble) {
                    section = Section::Other;
                    continue;
                }
            }
        }
        if !in_fence && !matches!(section, Section::Key) && is_key_table_header(line) {
            section = Section::Key;
            continue;
        }
        match section {
            Section::Preamble => match meta_comment(line) {
                Some((k, v)) if !in_fence => {
                    p.meta.insert(k, v);
                }
                _ => p.description.push(line.to_string()),
            },
            Section::Question => p.questions.last_mut().unwrap().lines.push(line.to_string()),
            Section::Key => {
                if let Some((n, letter)) = key_row(line) {
                    p.key.insert(n, letter);
                }
            }
            Section::Other => {}
        }
    }
    p
}

/// `| Soru No | Doğru Cevap |` starts an answer key even without a heading.
fn is_key_table_header(line: &str) -> bool {
    let cells: Vec<String> = line
        .split('|')
        .map(|c| c.trim().trim_matches('*').to_lowercase())
        .filter(|c| !c.is_empty())
        .collect();
    cells.len() >= 2
        && (cells[0].starts_with("soru") || cells[0].starts_with("question"))
        && cells[1..]
            .iter()
            .any(|c| c.contains("cevap") || c.contains("answer"))
}

/// `| 12 | C |` → (12, 'C')
fn key_row(line: &str) -> Option<(u32, char)> {
    let cells: Vec<&str> = line
        .split('|')
        .map(str::trim)
        .filter(|c| !c.is_empty())
        .collect();
    if cells.len() < 2 {
        return None;
    }
    let n: u32 = cells[0].trim_matches('*').parse().ok()?;
    let letter = cells[1]
        .trim_matches('*')
        .chars()
        .next()?
        .to_ascii_uppercase();
    ('A'..='H').contains(&letter).then_some((n, letter))
}

/// Option lines, in the styles exam authors actually use:
/// `- A) text`, `* **B)** text`, `- c) text`, `A) text`, `A-) text`, and a bare
/// `A)` followed by a code block. Without a list bullet only capital letters
/// count, so ordinary sentences are not mistaken for options.
fn option_line(line: &str) -> Option<(char, String)> {
    let indent = line.len() - line.trim_start().len();
    if indent > 3 {
        return None;
    }
    let t = line.trim_start();
    let (bulleted, t) = match t
        .strip_prefix(['-', '*', '+'])
        .and_then(|r| r.strip_prefix(' '))
    {
        Some(rest) => (true, rest.trim_start()),
        None => (false, t),
    };
    let t = t.trim_start_matches("**");
    let mut chars = t.chars();
    let first = chars.next()?;
    let letter = first.to_ascii_uppercase();
    if !('A'..='H').contains(&letter) || (!bulleted && !first.is_ascii_uppercase()) {
        return None;
    }
    // ")", "-)", ")**", "**)"
    let rest = chars.as_str();
    let rest = rest.strip_prefix("**").unwrap_or(rest);
    let rest = rest.strip_prefix("-)").or_else(|| rest.strip_prefix(')'))?;
    let rest = rest.strip_prefix("**").unwrap_or(rest);
    if !bulleted && !(rest.is_empty() || rest.starts_with(' ')) {
        return None;
    }
    Some((letter, rest.trim().to_string()))
}

fn trim_blank(lines: &[String]) -> String {
    let start = lines
        .iter()
        .position(|l| !l.trim().is_empty())
        .unwrap_or(lines.len());
    let end = lines
        .iter()
        .rposition(|l| !l.trim().is_empty() && l.trim() != "---" && l.trim() != "***")
        .map_or(start, |i| i + 1);
    lines[start..end.max(start)].join("\n")
}

/// Option sets whose order carries meaning.
fn looks_order_dependent(options: &[String]) -> bool {
    const WORDS: [&str; 8] = [
        "hepsi",
        "hiçbiri",
        "yalnızca",
        "yukarıdaki",
        "all of the above",
        "none of the above",
        "only",
        "both",
    ];
    options.iter().any(|o| {
        let lower = o.to_lowercase();
        if WORDS.iter().any(|w| lower.contains(w)) {
            return true;
        }
        let romans = o
            .split(|c: char| !c.is_alphanumeric())
            .filter(|t| matches!(*t, "I" | "II" | "III" | "IV" | "V" | "VI"))
            .count();
        romans >= 2
    })
}

/// `**A**`, `**B)**` ... in an explanation.
fn mentions_option_letter(text: &str) -> bool {
    text.match_indices("**").any(|(i, _)| {
        let rest = &text[i + 2..];
        let mut chars = rest.chars();
        matches!(chars.next(), Some('A'..='H'))
            && matches!(chars.as_str().trim_start_matches(')').get(..2), Some("**"))
    })
}

fn has_relative_image(text: &str) -> bool {
    text.match_indices("](").any(|(i, _)| {
        let before = &text[..i];
        let is_image = before
            .rfind("![")
            .is_some_and(|b| !before[b..].contains(']'));
        let url = &text[i + 2..];
        is_image
            && !(url.starts_with("http://")
                || url.starts_with("https://")
                || url.starts_with("data:")
                || url.starts_with('/'))
    })
}

pub fn slug(s: &str) -> String {
    let mut out = String::new();
    for c in s.to_lowercase().chars() {
        let c = match c {
            'ç' => 'c',
            'ğ' => 'g',
            'ı' => 'i',
            'ö' => 'o',
            'ş' => 's',
            'ü' => 'u',
            c => c,
        };
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    let out = out.trim_end_matches('-');
    out.chars().take(60).collect()
}

struct BuiltQuestion {
    number: u32,
    value: Map<String, Value>,
    letters: Vec<char>,
    inline_answer: Option<char>,
}

fn build_question(
    q: RawQuestion,
    warnings: &mut Vec<Warning>,
) -> Result<BuiltQuestion, ConvertError> {
    let mut in_fence = false;
    let mut stem = Vec::new();
    let mut options: Vec<(char, Vec<String>)> = Vec::new();
    let mut explanation = Vec::new();
    let mut topics = Vec::new();
    let mut lock: Option<bool> = None;
    let mut inline_answer = None;
    let mut ignored = false;
    #[derive(PartialEq)]
    enum Part {
        Stem,
        Options,
        After,
    }
    let mut part = Part::Stem;
    let mut fence_indent = 0;

    for line in q.lines {
        let fence = is_fence(&line);
        if !in_fence {
            if let Some((k, v)) = meta_comment(&line) {
                match k.as_str() {
                    "topics" | "topic" | "konular" | "konu" => {
                        topics = v
                            .split(',')
                            .map(|t| t.trim().to_string())
                            .filter(|t| !t.is_empty())
                            .collect()
                    }
                    "answer" | "cevap" => {
                        inline_answer = v.trim().chars().next().map(|c| c.to_ascii_uppercase())
                    }
                    "lock" => lock = Some(true),
                    "shuffle" => lock = Some(false),
                    _ => {}
                }
                continue;
            }
            if part != Part::After
                && let Some((letter, text)) = option_line(&line)
            {
                part = Part::Options;
                options.push((letter, vec![text]));
                continue;
            }
        }
        if fence {
            in_fence = !in_fence;
        }
        match part {
            Part::Stem => stem.push(line),
            Part::Options => {
                let continuation = line.starts_with("  ") || line.starts_with('\t');
                let lead = line.len() - line.trim_start().len();
                if fence && in_fence {
                    fence_indent = lead; // opening fence of a code block
                }
                if in_fence || fence {
                    // Keep code indentation; only drop the list item's own indent.
                    let cut = lead.min(fence_indent);
                    options.last_mut().unwrap().1.push(line[cut..].to_string());
                } else if line.trim().is_empty() {
                    continue;
                } else if continuation {
                    options
                        .last_mut()
                        .unwrap()
                        .1
                        .push(line.trim_start().to_string());
                } else {
                    part = Part::After;
                    after_line(&line, &mut explanation, &mut ignored);
                }
            }
            Part::After => after_line(&line, &mut explanation, &mut ignored),
        }
    }

    if options.len() < 2 {
        return Err(ConvertError::NoOptions { question: q.number });
    }
    let letters: Vec<char> = options.iter().map(|(l, _)| *l).collect();
    if letters
        .iter()
        .enumerate()
        .any(|(i, &l)| l != (b'A' + i as u8) as char)
    {
        return Err(ConvertError::OptionOrder { question: q.number });
    }
    if ignored {
        warnings.push(Warning::IgnoredText { question: q.number });
    }

    let option_texts: Vec<String> = options.iter().map(|(_, l)| l.join("\n")).collect();
    let lock = lock.unwrap_or_else(|| looks_order_dependent(&option_texts));

    let mut v = Map::new();
    v.insert("id".into(), json!(format!("q{:02}", q.number)));
    if !topics.is_empty() {
        v.insert("topics".into(), json!(topics));
    }
    v.insert("stem".into(), json!(trim_blank(&stem)));
    v.insert(
        "options".into(),
        Value::Array(
            letters
                .iter()
                .zip(&option_texts)
                .map(|(l, t)| json!({ "id": l.to_ascii_lowercase().to_string(), "text": t }))
                .collect(),
        ),
    );
    if lock {
        v.insert("lock_options".into(), json!(true));
    }
    let explanation = trim_blank(&explanation);
    if !lock && mentions_option_letter(&explanation) {
        warnings.push(Warning::LettersInShuffledExplanation { question: q.number });
    }
    if !explanation.is_empty() {
        v.insert("explanation".into(), json!(explanation));
    }
    Ok(BuiltQuestion {
        number: q.number,
        value: v,
        letters,
        inline_answer,
    })
}

/// A blockquote after the options is the explanation; an optional
/// "Açıklama:" / "Explanation:" label is dropped.
fn after_line(line: &str, explanation: &mut Vec<String>, ignored: &mut bool) {
    let t = line.trim();
    if let Some(q) = t.strip_prefix('>') {
        let mut q = q.trim_start().to_string();
        if explanation.is_empty() {
            for label in [
                "**Açıklama:**",
                "**Açıklama**:",
                "Açıklama:",
                "**Explanation:**",
                "Explanation:",
            ] {
                if let Some(rest) = q.strip_prefix(label) {
                    q = rest.trim_start().to_string();
                    break;
                }
            }
        }
        explanation.push(q);
    } else if !t.is_empty() && t != "---" && t != "***" {
        *ignored = true;
    }
}

pub fn exam_markdown_to_json(text: &str, opts: &ConvertOptions) -> Result<Converted, ConvertError> {
    let text = text.replace("\r\n", "\n");
    let parsed = split_sections(&text);
    if parsed.questions.is_empty() {
        return Err(ConvertError::NoQuestions);
    }
    let mut warnings = Vec::new();

    let title = match parsed.title {
        Some(t) => t,
        None => {
            warnings.push(Warning::NoTitle);
            "Quiz".to_string()
        }
    };
    let id = opts
        .id
        .clone()
        .or_else(|| parsed.meta.get("id").cloned())
        .or_else(|| opts.default_id.clone())
        .map(|s| slug(&s))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            let s = slug(&title);
            if s.is_empty() { "quiz".into() } else { s }
        });
    let time = opts
        .time_limit_minutes
        .or_else(|| parsed.meta.get("time").and_then(|t| t.parse().ok()));

    let mut built = Vec::new();
    for q in parsed.questions {
        built.push(build_question(q, &mut warnings)?);
    }

    // Answers: inline comment first, then the key table.
    let mut answers: Vec<Option<char>> = Vec::new();
    for b in &built {
        let a = b
            .inline_answer
            .or_else(|| parsed.key.get(&b.number).copied());
        let a = match a {
            Some(l) if b.letters.contains(&l) => Some(l),
            Some(l) => {
                warnings.push(Warning::AnswerNotAnOption {
                    question: b.number,
                    letter: l,
                });
                None
            }
            None => None,
        };
        answers.push(a);
    }
    for n in parsed.key.keys() {
        if !built.iter().any(|b| b.number == *n) {
            warnings.push(Warning::UnknownKeyEntry { question: *n });
        }
    }
    let answered = answers.iter().filter(|a| a.is_some()).count();
    if answered > 0 && answered < built.len() {
        // The file format requires all or nothing.
        for (b, a) in built.iter().zip(&answers) {
            if a.is_none() {
                warnings.push(Warning::MissingAnswer { question: b.number });
            }
        }
    }
    let keep_answers = answered == built.len();

    let description = trim_blank(&parsed.description);
    let all_text = format!(
        "{description}\n{}",
        built
            .iter()
            .map(|b| b.value["stem"].to_string())
            .collect::<String>()
    );
    let mut questions = Vec::new();
    for (mut b, a) in built.into_iter().zip(answers) {
        if keep_answers && let Some(l) = a {
            b.value
                .insert("answer".into(), json!(l.to_ascii_lowercase().to_string()));
        }
        // Keep a stable field order: answer before explanation.
        if let Some(e) = b.value.remove("explanation") {
            b.value.insert("explanation".into(), e);
        }
        questions.push(Value::Object(b.value));
    }

    if opts.assets_base_url.is_none() && has_relative_image(&all_text) {
        warnings.push(Warning::RelativeImages);
    }

    let mut root = Map::new();
    root.insert("schema_version".into(), json!(1));
    root.insert("id".into(), json!(id));
    root.insert("title".into(), json!(title));
    if !description.is_empty() {
        root.insert("description".into(), json!(description));
    }
    if let Some(base) = &opts.assets_base_url {
        root.insert("assets_base_url".into(), json!(base));
    }
    let mut settings = Map::new();
    settings.insert("shuffle_questions".into(), json!(opts.shuffle_questions));
    settings.insert(
        "shuffle_options".into(),
        json!(opts.shuffle_options.unwrap_or(true)),
    );
    if let Some(t) = time.filter(|&t| t > 0) {
        settings.insert("time_limit_minutes".into(), json!(t));
    }
    root.insert("settings".into(), Value::Object(settings));
    let count = questions.len();
    root.insert("questions".into(), Value::Array(questions));

    Ok(Converted {
        json: serde_json::to_string_pretty(&Value::Object(root))
            .expect("json values always serialize"),
        questions: count,
        answers: if keep_answers { answered } else { 0 },
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto;

    const EXAM: &str = r#"# Örnek Sınav

Hazırlık soruları.

<!-- time: 30 -->

## Soru 1

TDD döngüsü hangisidir?

- A) Red - Green - Refactor
- B) Build - Test
- C) Plan - Do
- D) Code - Ship

> Açıklama: Önce **başarısız** test yazılır.

<!-- topics: Test, TDD -->

## Soru 2

Kod:

```python
items = [1, 2]
- A) not an option, inside code
```

- I. Birinci ifade
- II. İkinci ifade

Hangileri doğrudur?

- A) Yalnızca I
- B) Yalnızca II
- C) I ve II
- D) Hiçbiri

## Soru 3
![Mimari](./images/x.png)

* **A)** Bir
* **B)** İki
  devamı

## Cevap Anahtarı

| Soru No | Doğru Cevap |
|---------|-------------|
| 1       | A           |
| 2       | C           |
| 3       | B           |
"#;

    fn convert(text: &str) -> Converted {
        exam_markdown_to_json(text, &ConvertOptions::default()).unwrap()
    }

    #[test]
    fn converts_exam_format() {
        let c = convert(EXAM);
        assert_eq!((c.questions, c.answers), (3, 3));
        assert_eq!(c.warnings, vec![Warning::RelativeImages]);
        let p = dto::parse(&c.json).expect("output must be a valid quiz");
        assert_eq!(p.quiz.id, "ornek-sinav");
        assert_eq!(p.quiz.title, "Örnek Sınav");
        assert_eq!(p.quiz.description.as_deref(), Some("Hazırlık soruları."));
        assert_eq!(p.quiz.settings.time_limit_minutes, Some(30));
        assert!(p.quiz.settings.shuffle_choices);

        let q1 = &p.quiz.questions[0];
        assert_eq!(q1.id.0, "q01");
        assert_eq!(q1.topics, vec!["Test", "TDD"]);
        assert_eq!(q1.stem, "TDD döngüsü hangisidir?");
        assert!(!q1.lock_choices);
        let key = p.key.unwrap();
        let e = key.get(&q1.id).unwrap();
        assert_eq!(e.correct.0, "a");
        assert_eq!(
            e.explanation.as_deref(),
            Some("Önce **başarısız** test yazılır.")
        );

        let q2 = &p.quiz.questions[1];
        assert!(q2.stem.contains("- A) not an option, inside code"));
        assert!(q2.stem.contains("- II. İkinci ifade"));
        assert_eq!(q2.choices.len(), 4);
        assert!(
            q2.lock_choices,
            "Roman numeral / 'Hiçbiri' options keep their order"
        );

        let q3 = &p.quiz.questions[2];
        assert_eq!(q3.choices[1].text, "İki\ndevamı");
        assert_eq!(key.get(&q3.id).unwrap().correct.0, "b");
    }

    #[test]
    fn options_and_base_url() {
        let with_default = ConvertOptions {
            default_id: Some("file-name".into()),
            ..Default::default()
        };
        assert_eq!(
            dto::parse(&exam_markdown_to_json(EXAM, &with_default).unwrap().json)
                .unwrap()
                .quiz
                .id,
            "file-name"
        );
        let o = ConvertOptions {
            id: Some("Exam 00".into()),
            assets_base_url: Some("https://x.io/".into()),
            ..Default::default()
        };
        let c = exam_markdown_to_json(EXAM, &o).unwrap();
        assert!(c.warnings.is_empty());
        let p = dto::parse(&c.json).unwrap();
        assert_eq!(p.quiz.id, "exam-00");
        assert_eq!(p.quiz.assets_base_url.as_deref(), Some("https://x.io/"));
    }

    #[test]
    fn inline_answer_and_partial_key() {
        let md =
            "# T\n## Soru 1\nS\n- A) a\n- B) b\n<!-- answer: b -->\n## Soru 2\nS\n- A) a\n- B) b\n";
        let c = convert(md);
        assert_eq!(c.answers, 0, "answers are all or nothing");
        assert_eq!(c.warnings, vec![Warning::MissingAnswer { question: 2 }]);
        assert!(dto::parse(&c.json).unwrap().key.is_none());
    }

    #[test]
    fn bad_answers_are_reported() {
        let md = "# T\n## Soru 1\nS\n- A) a\n- B) b\n## Answer Key\n| 1 | D |\n| 9 | A |\n";
        let c = convert(md);
        assert!(c.warnings.contains(&Warning::AnswerNotAnOption {
            question: 1,
            letter: 'D'
        }));
        assert!(
            c.warnings
                .contains(&Warning::UnknownKeyEntry { question: 9 })
        );
    }

    #[test]
    fn errors() {
        assert_eq!(
            exam_markdown_to_json("# Only a title", &Default::default()),
            Err(ConvertError::NoQuestions)
        );
        assert_eq!(
            exam_markdown_to_json("## Soru 4\nS\n- A) a\n", &Default::default()),
            Err(ConvertError::NoOptions { question: 4 })
        );
        assert_eq!(
            exam_markdown_to_json("## Soru 1\nS\n- A) a\n- C) c\n", &Default::default()),
            Err(ConvertError::OptionOrder { question: 1 })
        );
    }

    #[test]
    fn bundled_markdown_sample_converts_cleanly() {
        let c = convert(include_str!("../samples/exam-sample.md"));
        assert!(c.warnings.is_empty(), "{:?}", c.warnings);
        assert_eq!((c.questions, c.answers), (4, 4));
        let p = dto::parse(&c.json).unwrap();
        assert_eq!(p.quiz.id, "prova-markdown-ornegi");
        assert_eq!(p.quiz.settings.time_limit_minutes, Some(10));
        assert!(p.quiz.questions[1].lock_choices);
    }

    #[test]
    fn multi_line_comments_are_dropped() {
        let md = "# T\n<!--\nnot for students\n-->\nIntro\n## Soru 1\n<!--\nnote\n-->\nS\n- A) a\n- B) b\n";
        let p = dto::parse(&convert(md).json).unwrap();
        assert_eq!(p.quiz.description.as_deref(), Some("Intro"));
        assert_eq!(p.quiz.questions[0].stem, "S");
    }

    #[test]
    fn letters_in_explanation_need_locked_options() {
        let md = "# T\n## Soru 1\nS\n- A) a\n- B) b\n> Açıklama: **B** doğru, **A** değil.\n## Soru 2\n<!-- lock -->\nS\n- A) a\n- B) b\n> **A)** yanlış.\n";
        let c = convert(md);
        assert_eq!(
            c.warnings,
            vec![Warning::LettersInShuffledExplanation { question: 1 }]
        );
        assert!(mentions_option_letter("**C)** x"));
        assert!(!mentions_option_letter("**API** ve **Ajan**"));
    }

    #[test]
    fn other_option_styles_and_key_without_heading() {
        let md = "# T\n## Soru 1\nHangisi?\n\nA)\n\n```csharp\npublic class A\n{\n\n    int x;\n}\n```\n\nB)\n\n```csharp\nclass B {}\n```\n\n## Soru 2\nS\n\nA-) bir\nB-) iki\n\n| Soru | Doğru Cevap |\n|---|---|\n| 1 | A |\n| 2 | B |\n";
        let c = convert(md);
        assert!(c.warnings.is_empty(), "{:?}", c.warnings);
        assert_eq!(c.answers, 2);
        let p = dto::parse(&c.json).unwrap();
        let a = &p.quiz.questions[0].choices[0].text;
        assert!(a.contains("public class A\n{\n\n    int x;\n}"), "{a:?}");
        assert_eq!(p.quiz.questions[1].choices[1].text, "iki");
        assert_eq!(option_line("Ağaç yapısı"), None);
        assert_eq!(option_line("a) küçük harf madde değil"), None);
        assert_eq!(option_line("C# bir dildir"), None);
        assert!(is_key_table_header("| **Soru No** | **Doğru Cevap** |"));
        assert!(!is_key_table_header("| Kavram | Tanım |"));
    }

    #[test]
    fn helpers() {
        assert_eq!(slug("Yapay Zekâ: Final / 2026"), "yapay-zek-final-2026");
        assert_eq!(key_row("| **12** | c |"), Some((12, 'C')));
        assert_eq!(key_row("|---|---|"), None);
        assert_eq!(option_line("- **C)** metin"), Some(('C', "metin".into())));
        assert_eq!(option_line("- I. ifade"), None);
        assert_eq!(question_number("Sorular"), None);
        assert_eq!(question_number("Question 7"), Some(Some(7)));
        assert!(has_relative_image("x ![a](img/b.png)"));
        assert!(!has_relative_image("x ![a](https://b.png) [link](rel.md)"));
    }
}
