use std::fs::File;
use std::io::{self, BufWriter, IsTerminal, Write};
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

use clap::Parser;

use rsx_a11y::diagnostics::{self, OutputFormat, Report};
use rsx_a11y::lints::{Rule, Severity};
use rsx_a11y::{LintOptions, check_files, collect_rust_files};

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
    s.trim().parse().map_err(|_| {
        let names: Vec<&str> = Rule::all().map(Rule::as_str).collect();
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

fn list_rules() -> io::Result<u8> {
    let mut out = io::stdout().lock();
    writeln!(out, "Available lint rules:")?;
    for rule in Rule::all() {
        writeln!(out)?;
        writeln!(out, "  {} ({})", rule, rule.severity())?;
        writeln!(out, "      {}", rule.description())?;
        if let Some(url) = rule.guidelines().first() {
            writeln!(out, "      see: {url}")?;
        }
    }
    Ok(EXIT_OK)
}

fn run(cli: Cli) -> io::Result<u8> {
    if cli.list_rules {
        return list_rules();
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

    let start_time = Instant::now();
    let rust_files = collect_rust_files(path)?;

    if format == OutputFormat::Pretty {
        if rust_files.is_empty() {
            eprintln!("No Rust files found in '{}'.", path.display());
        } else {
            eprintln!("Scanning {} file(s)...", rust_files.len());
        }
    }

    let mut options = LintOptions::default();
    if let Some(only) = cli.only {
        options = options.only(only);
    }
    if let Some(skip) = cli.skip {
        options = options.skip(skip);
    }
    if cli.quiet {
        options = options.min_severity(Severity::Error);
    }
    let summary = check_files(&rust_files, &options);

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
        diagnostics: &summary.diagnostics,
        parse_errors: &summary.parse_errors,
        files_scanned: summary.files_scanned,
        files_checked: summary.files_checked,
        duration: start_time.elapsed(),
    };
    diagnostics::write_report(&report, format, color, &mut *writer)?;
    // Flush explicitly: buffered output must reach its destination before we
    // report the exit status.
    writer.flush()?;

    Ok(if !summary.parse_errors.is_empty() {
        EXIT_FAILURE
    } else if summary.errors().next().is_some() {
        EXIT_LINT_ERRORS
    } else {
        EXIT_OK
    })
}
