//! Diagnostic formatting and output.
//!
//! Supports both human-readable (optionally colored) terminal output and a
//! machine-readable JSON report.

use std::io::{self, Write};
use std::time::Duration;

use crate::lints::{LintDiagnostic, Severity};
use crate::parser::ParseError;

/// Output format for diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Human-readable terminal output.
    Pretty,
    /// Machine-readable JSON output (see [`JsonReport`]).
    Json,
}

/// Number of diagnostics per severity.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
pub struct SeverityCounts {
    /// Number of [`Severity::Error`] diagnostics.
    pub errors: usize,
    /// Number of [`Severity::Warning`] diagnostics.
    pub warnings: usize,
    /// Number of [`Severity::Info`] diagnostics.
    pub infos: usize,
}

impl SeverityCounts {
    /// Count the diagnostics in `diagnostics` by severity.
    pub fn from_diagnostics(diagnostics: &[LintDiagnostic]) -> Self {
        let mut counts = Self::default();
        for d in diagnostics {
            match d.severity {
                Severity::Error => counts.errors += 1,
                Severity::Warning => counts.warnings += 1,
                Severity::Info => counts.infos += 1,
            }
        }
        counts
    }
}

/// Everything a report needs to render.
#[derive(Debug, Clone, Copy)]
pub struct Report<'a> {
    /// Diagnostics to print, already filtered and sorted.
    pub diagnostics: &'a [LintDiagnostic],
    /// Files that could not be read or parsed.
    pub parse_errors: &'a [ParseError],
    /// Number of `.rs` files that were scanned.
    pub files_scanned: usize,
    /// Number of files that contained at least one lintable element.
    pub files_checked: usize,
    /// Wall-clock time spent linting.
    pub duration: Duration,
}

/// Shape of the `--format json` output.
#[derive(Debug, serde::Serialize)]
pub struct JsonReport<'a> {
    /// All diagnostics, sorted by file, line and column.
    pub diagnostics: &'a [LintDiagnostic],
    /// Files that could not be read or parsed (and were therefore not linted).
    pub parse_errors: &'a [ParseError],
    /// Aggregate counts.
    pub summary: JsonSummary,
}

/// The `summary` object of a [`JsonReport`].
#[derive(Debug, serde::Serialize)]
pub struct JsonSummary {
    /// Number of `.rs` files that were scanned.
    pub files_scanned: usize,
    /// Number of files that contained at least one lintable element.
    pub files_checked: usize,
    /// Diagnostic counts per severity.
    #[serde(flatten)]
    pub counts: SeverityCounts,
    /// Number of files that could not be read or parsed.
    pub parse_errors: usize,
}

/// Write a full report (diagnostics, parse errors and summary) to `w`.
///
/// When `color` is true, ANSI colors are used for the pretty format. This only
/// has an effect when the `cli` feature is enabled.
pub fn write_report(
    report: &Report<'_>,
    format: OutputFormat,
    color: bool,
    w: &mut dyn Write,
) -> io::Result<()> {
    match format {
        OutputFormat::Pretty => write_pretty(report, color, w),
        OutputFormat::Json => write_json(report, w),
    }
}

fn write_json(report: &Report<'_>, w: &mut dyn Write) -> io::Result<()> {
    let json = JsonReport {
        diagnostics: report.diagnostics,
        parse_errors: report.parse_errors,
        summary: JsonSummary {
            files_scanned: report.files_scanned,
            files_checked: report.files_checked,
            counts: SeverityCounts::from_diagnostics(report.diagnostics),
            parse_errors: report.parse_errors.len(),
        },
    };
    serde_json::to_writer_pretty(&mut *w, &json)?;
    writeln!(w)
}

fn write_pretty(report: &Report<'_>, color: bool, w: &mut dyn Write) -> io::Result<()> {
    let p = Painter { color };

    for diag in report.diagnostics {
        let label = match diag.severity {
            Severity::Error => p.paint("error", Style::Error),
            Severity::Warning => p.paint("warning", Style::Warning),
            Severity::Info => p.paint("info", Style::Info),
        };
        writeln!(
            w,
            "{}{} {} {}",
            label,
            p.paint(":", Style::Bold),
            diag.message,
            p.paint(&format!("[{}]", diag.rule.to_string()), Style::Dimmed)
        )?;
        writeln!(
            w,
            "  {} {}:{}:{}",
            p.paint("-->", Style::Arrow),
            diag.file,
            diag.line,
            diag.column
        )?;
        if let Some(help) = &diag.help {
            writeln!(w, "  {} {}", p.paint("help:", Style::Help), help)?;
        }
        writeln!(w)?;
    }

    for err in report.parse_errors {
        writeln!(
            w,
            "{}{} {}",
            p.paint("error", Style::Error),
            p.paint(":", Style::Bold),
            err
        )?;
        writeln!(
            w,
            "  {} this file was not linted.",
            p.paint("note:", Style::Help)
        )?;
        writeln!(w)?;
    }

    let c = SeverityCounts::from_diagnostics(report.diagnostics);
    let plural = |n: usize| if n == 1 { "" } else { "s" };
    let mut summary = format!(
        "Checked {} file{} in {:.2?}. Found {} error{}, {} warning{}, {} info{}.",
        report.files_checked,
        plural(report.files_checked),
        report.duration,
        c.errors,
        plural(c.errors),
        c.warnings,
        plural(c.warnings),
        c.infos,
        plural(c.infos),
    );
    let n_parse = report.parse_errors.len();
    if n_parse > 0 {
        summary.push_str(&format!(
            " {} file{} could not be parsed.",
            n_parse,
            plural(n_parse)
        ));
    }
    writeln!(w, "{}", p.paint(&summary, Style::Bold))?;

    if c.errors > 0 || n_parse > 0 {
        let msg = "  Some issues must be fixed for accessibility compliance.";
        writeln!(w, "{}", p.paint(msg, Style::Error))?;
    } else if c.warnings > 0 {
        let msg = "  Consider addressing warnings to improve accessibility.";
        writeln!(w, "{}", p.paint(msg, Style::Warning))?;
    } else {
        writeln!(
            w,
            "{}",
            p.paint("  No accessibility issues found!", Style::Success)
        )?;
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum Style {
    Error,
    Warning,
    Info,
    Bold,
    Dimmed,
    Arrow,
    Help,
    Success,
}

struct Painter {
    #[cfg_attr(not(feature = "cli"), allow(dead_code))]
    color: bool,
}

impl Painter {
    #[cfg(feature = "cli")]
    fn paint(&self, text: &str, style: Style) -> String {
        use colored::Colorize;
        if !self.color {
            return text.to_string();
        }
        match style {
            Style::Error => text.red().bold(),
            Style::Warning => text.yellow().bold(),
            Style::Info => text.blue().bold(),
            Style::Bold => text.bold(),
            Style::Dimmed => text.dimmed(),
            Style::Arrow => text.blue().bold(),
            Style::Help => text.green().bold(),
            Style::Success => text.green().bold(),
        }
        .to_string()
    }

    #[cfg(not(feature = "cli"))]
    fn paint(&self, text: &str, _style: Style) -> String {
        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::Tag;
    use crate::lints::Rule;
    use crate::parser::ParseErrorKind;

    fn diag(severity: Severity) -> LintDiagnostic {
        LintDiagnostic {
            rule: Rule::AltText,
            message: "msg".into(),
            severity,
            file: "a.rs".into(),
            line: 3,
            column: 5,
            element: Tag::Img,
            help: Some("do x".into()),
        }
    }

    fn report<'a>(d: &'a [LintDiagnostic], e: &'a [ParseError]) -> Report<'a> {
        Report {
            diagnostics: d,
            parse_errors: e,
            files_scanned: 2,
            files_checked: 1,
            duration: Duration::from_millis(1),
        }
    }

    #[test]
    fn pretty_output_includes_location_help_and_parse_errors() {
        let d = [diag(Severity::Error)];
        let e = [ParseError::new(ParseErrorKind::Rust, "b.rs", "boom")];
        let mut out = Vec::new();
        write_report(&report(&d, &e), OutputFormat::Pretty, false, &mut out).unwrap();
        let out = String::from_utf8(out).unwrap();
        assert!(out.contains("error: msg [alt-text]"), "{out}");
        assert!(out.contains("--> a.rs:3:5"), "{out}");
        assert!(out.contains("help: do x"), "{out}");
        assert!(out.contains("Failed to parse b.rs: boom"), "{out}");
        assert!(out.contains("1 file could not be parsed"), "{out}");
        assert!(!out.contains('\u{1b}'), "no ANSI codes when color is off");
    }

    #[test]
    fn json_output_is_a_single_object_with_summary() {
        let d = [diag(Severity::Error), diag(Severity::Warning)];
        let e = [ParseError::new(ParseErrorKind::Markup, "b.rs", "bad")];
        let mut out = Vec::new();
        write_report(&report(&d, &e), OutputFormat::Json, false, &mut out).unwrap();
        let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(v["diagnostics"].as_array().unwrap().len(), 2);
        assert_eq!(v["parse_errors"][0]["kind"], "markup");
        assert_eq!(v["summary"]["errors"], 1);
        assert_eq!(v["summary"]["warnings"], 1);
        assert_eq!(v["summary"]["parse_errors"], 1);
        assert_eq!(v["summary"]["files_scanned"], 2);
    }
}
