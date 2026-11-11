use std::fs::File;
use std::io::{self, BufWriter, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicUsize, Ordering};

use clap::Parser;
use rayon::prelude::*;
use strum::IntoEnumIterator;
use walkdir::WalkDir;

use rsx_a11y::diagnostics::{self, OutputFormat, Report};
use rsx_a11y::lints::{self, LintDiagnostic, Rule, Severity};
use rsx_a11y::parser::{self, ParseError};

/// Exit code when every file was linted and no error-level issue was found.
const EXIT_OK: u8 = 0;
/// Exit code when at least one error-level accessibility issue was found.
const EXIT_LINT_ERRORS: u8 = 1;
/// Exit code for usage errors, I/O failures, or files that could not be parsed.
const EXIT_FAILURE: u8 = 2;

/// rsx-a11y: Lint ARIA and accessibility attributes in Rust web frameworks.
///
/// Checks `html!` (Yew), `view!` (Leptos), and `rsx!` (Dioxus) macros for
/// accessibility issues based on the WAI-ARIA specifications.
///
/// Exit status: 0 = no errors, 1 = accessibility errors found,
/// 2 = invalid usage or files that could not be read/parsed.
#[derive(Parser, Debug)]
#[command(name = "rsx-a11y", version, about, long_about = None)]
struct Cli {
    /// Path to a Rust file or directory to lint.
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Output format.
    #[arg(long, value_enum, default_value = "pretty")]
    format: Format,

    /// Only show errors (hide warnings and info).
    #[arg(short, long)]
    quiet: bool,

    /// List all available lint rules and exit.
    #[arg(long)]
    list_rules: bool,

    /// Only enable specific rules (comma-separated). If not set, all rules are enabled.
    #[arg(long, value_delimiter = ',', value_parser = parse_rule, value_name = "RULES")]
    only: Option<Vec<Rule>>,

    /// Disable specific rules (comma-separated).
    #[arg(long, value_delimiter = ',', value_parser = parse_rule, value_name = "RULES")]
    skip: Option<Vec<Rule>>,

    /// Write output to a file instead of stdout.
    #[arg(long, value_name = "PATH")]
    out_file: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum Format {
    Pretty,
    Json,
}

impl From<Format> for OutputFormat {
    fn from(f: Format) -> Self {
        match f {
            Format::Pretty => OutputFormat::Pretty,
            Format::Json => OutputFormat::Json,
        }
    }
}

/// Parse a rule name for `--only` / `--skip`, rejecting unknown names so a
/// typo can never silently disable linting.
fn parse_rule(s: &str) -> Result<Rule, String> {
    Rule::from_str(s.trim()).ok_or_else(|| {
        let names: Vec<String> = Rule::iter().map(|r| r.to_string()).collect();
        format!(
            "unknown rule '{s}'. Run `rsx-a11y --list-rules` to see all rules.\n  valid rules: {}",
            names.join(", ")
        )
    })
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(code) => ExitCode::from(code),
        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => ExitCode::from(EXIT_OK),
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::from(EXIT_FAILURE)
        }
    }
}

fn run(cli: Cli) -> io::Result<u8> {
    if cli.list_rules {
        let mut out = io::stdout().lock();
        writeln!(out, "Available lint rules:")?;
        writeln!(out)?;
        for rule in Rule::iter() {
            writeln!(out, "  {}", rule.to_string())?;
        }
        return Ok(EXIT_OK);
    }

    let format: OutputFormat = cli.format.into();
    let path = &cli.path;

    if !path.exists() {
        eprintln!("Error: path '{}' does not exist.", path.display());
        return Ok(EXIT_FAILURE);
    }

    // Refuse to scan filesystem roots — almost certainly a mistake.
    // On Windows, "/" resolves to the current drive root (e.g. C:\), not
    // the current directory. Use "." for the current directory instead.
    if let Ok(canonical) = dunce::canonicalize(path)
        && canonical.parent().is_none()
    {
        eprintln!(
            "Error: '{}' resolves to filesystem root '{}'. Did you mean '.'?",
            path.display(),
            canonical.display()
        );
        return Ok(EXIT_FAILURE);
    }

    let start_time = std::time::Instant::now();
    let rust_files = collect_rust_files(path);

    if format == OutputFormat::Pretty {
        if rust_files.is_empty() {
            eprintln!("No Rust files found in '{}'.", path.display());
        } else {
            eprintln!("Scanning {} file(s)...", rust_files.len());
        }
    }

    let filter = RuleFilter {
        only: cli.only,
        skip: cli.skip,
        only_errors: cli.quiet,
    };
    let result = lint_files(&rust_files, &filter);

    // Colors only make sense for an interactive terminal.
    let to_terminal = cli.out_file.is_none() && io::stdout().is_terminal();
    let color = to_terminal && std::env::var_os("NO_COLOR").is_none();

    let mut writer: Box<dyn Write> = match &cli.out_file {
        Some(out) => match File::create(out) {
            Ok(file) => Box::new(BufWriter::new(file)),
            Err(e) => {
                eprintln!("Error: could not create '{}': {}", out.display(), e);
                return Ok(EXIT_FAILURE);
            }
        },
        None => Box::new(BufWriter::new(io::stdout().lock())),
    };

    let report = Report {
        diagnostics: &result.diagnostics,
        parse_errors: &result.parse_errors,
        files_scanned: rust_files.len(),
        files_checked: result.files_checked,
        duration: start_time.elapsed(),
    };
    diagnostics::write_report(&report, format, color, &mut *writer)?;
    // Flush explicitly: buffered output must reach its destination before we
    // report the exit status.
    writer.flush()?;

    let code = if !result.parse_errors.is_empty() {
        EXIT_FAILURE
    } else if result
        .diagnostics
        .iter()
        .any(|d| d.severity == Severity::Error)
    {
        EXIT_LINT_ERRORS
    } else {
        EXIT_OK
    };
    Ok(code)
}

struct RuleFilter {
    only: Option<Vec<Rule>>,
    skip: Option<Vec<Rule>>,
    only_errors: bool,
}

impl RuleFilter {
    fn keep(&self, d: &LintDiagnostic) -> bool {
        self.only.as_ref().is_none_or(|only| only.contains(&d.rule))
            && self
                .skip
                .as_ref()
                .is_none_or(|skip| !skip.contains(&d.rule))
            && (!self.only_errors || d.severity == Severity::Error)
    }
}

struct LintResult {
    diagnostics: Vec<LintDiagnostic>,
    parse_errors: Vec<ParseError>,
    files_checked: usize,
}

fn lint_files(rust_files: &[PathBuf], filter: &RuleFilter) -> LintResult {
    let files_checked = AtomicUsize::new(0);

    let (mut diagnostics, mut parse_errors) = rust_files
        .par_iter()
        .fold(
            || (Vec::new(), Vec::new()),
            |(mut diags, mut errors), file| {
                match parser::parse_file(file) {
                    Ok(elements) => {
                        if !elements.is_empty() {
                            files_checked.fetch_add(1, Ordering::Relaxed);
                            diags
                                .extend(lints::run_all_lints(&elements).filter(|d| filter.keep(d)));
                        }
                    }
                    Err(e) => errors.push(e),
                }
                (diags, errors)
            },
        )
        .reduce(
            || (Vec::new(), Vec::new()),
            |(mut d1, mut e1), (d2, e2)| {
                d1.extend(d2);
                e1.extend(e2);
                (d1, e1)
            },
        );

    diagnostics.sort_unstable_by(|a, b| {
        (a.file.as_str(), a.line, a.column).cmp(&(b.file.as_str(), b.line, b.column))
    });
    parse_errors.sort_unstable_by(|a, b| a.file.cmp(&b.file));

    LintResult {
        diagnostics,
        parse_errors,
        files_checked: files_checked.load(Ordering::Relaxed),
    }
}

/// Collect all `.rs` files from a path (file or directory).
///
/// All returned paths are guaranteed to be descendants of `path`.
/// Paths are returned relative to the current working directory when possible.
fn collect_rust_files(path: &Path) -> Vec<PathBuf> {
    if path.is_file() {
        if path.extension().is_some_and(|ext| ext == "rs") {
            return vec![path.to_path_buf()];
        }
        return Vec::new();
    }

    // Canonicalize the root so we can verify every result is a true descendant.
    let root = match dunce::canonicalize(path).or_else(|_| path.canonicalize()) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    // Also canonicalize cwd so we can produce relative display paths.
    let cwd = std::env::current_dir()
        .ok()
        .and_then(|d| dunce::canonicalize(&d).ok());

    WalkDir::new(&root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| {
            if entry.file_type().is_dir() && entry.depth() > 0 {
                let name = entry.file_name().to_string_lossy();
                return !matches!(name.as_ref(), "target" | "node_modules")
                    && !name.starts_with('.');
            }
            true
        })
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.file_type().is_file() && entry.path().extension().is_some_and(|ext| ext == "rs")
        })
        .filter_map(|entry| {
            // Hard check: the file's canonical path must start with root.
            let canonical = dunce::canonicalize(entry.path()).ok()?;
            if !canonical.starts_with(&root) {
                return None;
            }
            // Return a relative path when possible for cleaner output.
            if let Some(cwd) = &cwd
                && let Ok(rel) = canonical.strip_prefix(cwd)
            {
                return Some(rel.to_path_buf());
            }
            Some(canonical)
        })
        .collect()
}
