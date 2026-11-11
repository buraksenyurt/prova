//! Converts an exam written in Markdown into a Prova quiz file.
//!
//! ```text
//! cargo run --bin md2quiz -- Exam00.md -o exam00.json \
//!     --assets-base-url https://raw.githubusercontent.com/<user>/<repo>/main/
//! ```

use std::path::Path;
use std::process::ExitCode;

use prova::convert::{ConvertOptions, exam_markdown_to_json};
use prova::dto;
use prova::i18n::Lang;

const USAGE: &str = "\
Usage: md2quiz <exam.md> [options]

Options:
  -o, --output <file>        Write the JSON here (default: stdout)
      --id <id>              Quiz id (default: from <!-- id: --> or the file name)
      --assets-base-url <u>  Base URL for relative image paths
      --time <minutes>       Time limit for exam mode
      --shuffle-questions    Shuffle questions by default
      --no-shuffle-options   Keep option order by default
  -h, --help                 Show this help";

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("error: {msg}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    let mut input = None;
    let mut output = None;
    let mut opts = ConvertOptions::default();
    let mut it = args.into_iter();
    while let Some(arg) = it.next() {
        let mut value = |name: &str| {
            it.next()
                .ok_or_else(|| format!("{name} needs a value\n\n{USAGE}"))
        };
        match arg.as_str() {
            "-h" | "--help" => {
                println!("{USAGE}");
                return Ok(());
            }
            "-o" | "--output" => output = Some(value("--output")?),
            "--id" => opts.id = Some(value("--id")?),
            "--assets-base-url" => opts.assets_base_url = Some(value("--assets-base-url")?),
            "--time" => {
                let v = value("--time")?;
                opts.time_limit_minutes =
                    Some(v.parse().map_err(|_| format!("invalid --time: {v}"))?);
            }
            "--shuffle-questions" => opts.shuffle_questions = true,
            "--no-shuffle-options" => opts.shuffle_options = Some(false),
            a if a.starts_with('-') => return Err(format!("unknown option {a}\n\n{USAGE}")),
            a if input.is_none() => input = Some(a.to_string()),
            a => return Err(format!("unexpected argument {a}")),
        }
    }
    let input = input.ok_or_else(|| USAGE.to_string())?;
    let text = std::fs::read_to_string(&input).map_err(|e| format!("{input}: {e}"))?;
    // Used when the file has no <!-- id: --> comment.
    opts.default_id = Path::new(&input)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned());

    let lang = Lang::En;
    let converted = exam_markdown_to_json(&text, &opts).map_err(|e| lang.convert_error(&e))?;
    // The output must load in the app; this catches converter bugs early.
    dto::parse(&converted.json).map_err(|e| lang.schema_error(&e))?;

    for w in &converted.warnings {
        eprintln!("warning: {}", lang.convert_warning(w));
    }
    match output {
        Some(path) => {
            std::fs::write(&path, &converted.json).map_err(|e| format!("{path}: {e}"))?;
            eprintln!(
                "{}: {} questions, {} answers → {path}",
                input, converted.questions, converted.answers
            );
        }
        None => println!("{}", converted.json),
    }
    Ok(())
}
