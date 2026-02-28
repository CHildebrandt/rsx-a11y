use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicUsize, Ordering};

use clap::Parser;
use rayon::prelude::*;
use strum::IntoEnumIterator;
use walkdir::WalkDir;

use rsx_a11y::diagnostics::{self, OutputFormat};
use rsx_a11y::lints::{self, LintDiagnostic, Rule};
use rsx_a11y::parser;

/// rsx-a11y: Lint ARIA and accessibility attributes in Rust web frameworks.
///
/// Checks `html!` (Yew), `view!` (Leptos), and `rsx!` (Dioxus) macros for
/// accessibility issues based on the WAI-ARIA specifications.
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

    /// Specific lint rules to enable (comma-separated). If not set, all rules are enabled.
    #[arg(long, value_delimiter = ',')]
    only: Option<Vec<String>>,

    /// Lint rules to disable (comma-separated).
    #[arg(long, value_delimiter = ',')]
    skip: Option<Vec<String>>,

    /// Write diagnostic output to a file instead of stdout (useful for snapshot testing).
    #[arg(long)]
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

fn main() {
    let cli = Cli::parse();

    if cli.list_rules {
        println!("Available lint rules:");
        println!();
        for rule in Rule::iter() {
            println!("  {}", rule.to_string());
        }
        process::exit(0);
    }

    let format: OutputFormat = cli.format.into();
    let path = &cli.path;

    if !path.exists() {
        eprintln!("Error: path '{}' does not exist.", path.display());
        process::exit(1);
    }

    // Refuse to scan filesystem roots — almost certainly a mistake.
    // On Windows, "/" resolves to the current drive root (e.g. C:\), not
    // the current directory. Use "." for the current directory instead.
    if let Ok(canonical) = dunce::canonicalize(path) {
        if canonical.parent().is_none() {
            eprintln!(
                "Error: '{}' resolves to filesystem root '{}'. Did you mean '.'?",
                path.display(),
                canonical.display()
            );
            process::exit(1);
        }
    }

    let start_time = std::time::Instant::now();

    let rust_files = collect_rust_files(path);

    if rust_files.is_empty() {
        if format == OutputFormat::Pretty {
            eprintln!("No Rust files found in '{}'.", path.display());
        }
        process::exit(0);
    }

    if format == OutputFormat::Pretty {
        eprintln!("Scanning {} file(s)...", rust_files.len());
    }

    let CliLintSummary {
        diagnostics: all_diagnostics,
        parse_errors,
        files_checked,
    } = parse_files(
        &rust_files,
        cli.only
            .as_ref()
            .map(|only| only.iter().filter_map(|s| Rule::from_str(s)).collect()),
        cli.skip
            .as_ref()
            .map(|skip| skip.iter().filter_map(|s| Rule::from_str(s)).collect()),
        cli.quiet,
    );

    // Build writer: either a file or stdout.
    let mut writer: Box<dyn Write> = match cli.out_file {
        Some(ref path) => {
            let file = File::create(path).unwrap_or_else(|e| {
                eprintln!("Error: could not create '{}': {}", path.display(), e);
                process::exit(1);
            });
            Box::new(BufWriter::new(file))
        }
        None => Box::new(BufWriter::new(io::stdout().lock())),
    };

    diagnostics::print_diagnostics(&all_diagnostics, format, &mut *writer);

    if format == OutputFormat::Pretty {
        for err in &parse_errors {
            eprintln!("Parse error: {}", err);
        }
    }

    diagnostics::print_summary(
        &all_diagnostics,
        files_checked,
        start_time.elapsed(),
        format,
        &mut *writer,
    );

    // Exit with non-zero if there are errors
    let has_errors = all_diagnostics
        .iter()
        .any(|d| d.severity == lints::Severity::Error);
    if has_errors {
        process::exit(1);
    }
}

struct CliLintSummary {
    diagnostics: Vec<LintDiagnostic>,
    parse_errors: Vec<String>,
    files_checked: usize,
}

fn parse_files(
    rust_files: &[PathBuf],
    only: Option<Vec<Rule>>,
    skip: Option<Vec<Rule>>,
    only_errors: bool,
) -> CliLintSummary {
    let files_checked = AtomicUsize::new(0);

    // Process files in parallel with rayon.
    // Use fold + reduce to accumulate diagnostics directly, avoiding an
    // intermediate Vec<Result<…>> allocation.
    let (mut all_diagnostics, parse_errors) = rust_files
        .par_iter()
        .fold(
            || (Vec::new(), Vec::new()),
            |(mut diags, mut errors), file| {
                match parser::parse_file(file) {
                    Ok(elements) => {
                        if !elements.is_empty() {
                            files_checked.fetch_add(1, Ordering::Relaxed);

                            // Build a lazy iterator chain — filters run without
                            // allocating an intermediate Vec.
                            let file_diags = lints::run_all_lints(&elements)
                                .filter(|d| {
                                    only.as_ref()
                                        .map_or(true, |only| only.iter().any(|o| *o == d.rule))
                                })
                                .filter(|d| {
                                    skip.as_ref()
                                        .map_or(true, |skip| !skip.iter().any(|o| *o == d.rule))
                                })
                                .filter(|d| !only_errors || d.severity == lints::Severity::Error);

                            diags.extend(file_diags);
                        }
                    }
                    Err(e) => errors.push(e.to_string()),
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

    // Sort diagnostics by file, then line, then column
    all_diagnostics.sort_unstable_by(|a, b| {
        a.file
            .cmp(&b.file)
            .then(a.line.cmp(&b.line))
            .then(a.column.cmp(&b.column))
    });
    CliLintSummary {
        diagnostics: all_diagnostics,
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
        if path.extension().map_or(false, |ext| ext == "rs") {
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
            if entry.file_type().is_dir() {
                let name = entry.file_name().to_string_lossy();
                return !matches!(name.as_ref(), "target" | "node_modules")
                    && !name.starts_with('.');
            }
            true
        })
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.file_type().is_file() && entry.path().extension().map_or(false, |ext| ext == "rs")
        })
        .filter_map(|entry| {
            // Hard check: the file's canonical path must start with root.
            // Use dunce::canonicalize here too so both sides have consistent
            // prefix handling on Windows (std::canonicalize adds \\?\ prefix).
            let canonical = dunce::canonicalize(entry.path()).ok()?;
            if !canonical.starts_with(&root) {
                return None;
            }
            // Return a relative path when possible for cleaner output.
            if let Some(ref cwd) = cwd {
                if let Ok(rel) = canonical.strip_prefix(cwd) {
                    return Some(rel.to_path_buf());
                }
            }
            Some(canonical)
        })
        .collect()
}
