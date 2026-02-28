//! Diagnostic formatting and output.
//!
//! Supports both human-readable (colored terminal) and JSON output formats.

use std::io::Write;
use std::time::Duration;

use crate::lints::{LintDiagnostic, Severity};
#[cfg(feature = "cli")]
use colored::*;

/// Output format for diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Human-readable colored terminal output.
    Pretty,
    /// Machine-readable JSON output.
    Json,
}

/// Print diagnostics in the specified format.
pub fn print_diagnostics(diagnostics: &[LintDiagnostic], format: OutputFormat, w: &mut dyn Write) {
    match format {
        OutputFormat::Pretty => print_pretty(diagnostics, w),
        OutputFormat::Json => print_json(diagnostics, w),
    }
}

/// Print a summary of results.
pub fn print_summary(
    diagnostics: &[LintDiagnostic],
    files_checked: usize,
    duration: Duration,
    format: OutputFormat,
    w: &mut dyn Write,
) {
    if format == OutputFormat::Json {
        return; // JSON output includes everything
    }

    let (mut errors, mut warnings, mut infos) = (0usize, 0usize, 0usize);
    for d in diagnostics {
        match d.severity {
            Severity::Error => errors += 1,
            Severity::Warning => warnings += 1,
            Severity::Info => infos += 1,
        }
    }

    let summary = format!(
        "Checked {} file{} in {:.2?}. Found {} error{}, {} warning{}, {} info{}.",
        files_checked,
        if files_checked == 1 { "" } else { "s" },
        duration,
        errors,
        if errors == 1 { "" } else { "s" },
        warnings,
        if warnings == 1 { "" } else { "s" },
        infos,
        if infos == 1 { "" } else { "s" },
    );

    let _ = writeln!(w);
    #[cfg(feature = "cli")]
    let _ = writeln!(w, "{}", summary.bold());
    #[cfg(not(feature = "cli"))]
    let _ = writeln!(w, "{}", summary);

    if errors > 0 {
        let msg = "  Some issues must be fixed for accessibility compliance.";
        #[cfg(feature = "cli")]
        let _ = writeln!(w, "{}", msg.red().bold());
        #[cfg(not(feature = "cli"))]
        let _ = writeln!(w, "{}", msg);
    } else if warnings > 0 {
        let msg = "  Consider addressing warnings to improve accessibility.";
        #[cfg(feature = "cli")]
        let _ = writeln!(w, "{}", msg.yellow());
        #[cfg(not(feature = "cli"))]
        let _ = writeln!(w, "{}", msg);
    } else {
        let msg = "  No accessibility issues found!";
        #[cfg(feature = "cli")]
        let _ = writeln!(w, "{}", msg.green().bold());
        #[cfg(not(feature = "cli"))]
        let _ = writeln!(w, "{}", msg);
    }
}

#[cfg(feature = "cli")]
fn print_pretty(diagnostics: &[LintDiagnostic], w: &mut dyn Write) {
    for diag in diagnostics {
        let severity_label = match diag.severity {
            Severity::Error => "error".red().bold(),
            Severity::Warning => "warning".yellow().bold(),
            Severity::Info => "info".blue().bold(),
        };

        let lint_id = format!("[{}]", diag.rule.to_string()).dimmed();

        let _ = writeln!(
            w,
            "{}{} {} {}",
            severity_label,
            ":".bold(),
            diag.message,
            lint_id
        );
        let _ = writeln!(
            w,
            "  {} {}:{}:{}",
            "-->".blue().bold(),
            diag.file,
            diag.line,
            diag.column
        );

        if let Some(ref help) = diag.help {
            let _ = writeln!(w, "  {} {}", "help:".green().bold(), help);
        }

        let _ = writeln!(w);
    }
}

#[cfg(not(feature = "cli"))]
fn print_pretty(diagnostics: &[LintDiagnostic], w: &mut dyn Write) {
    for diag in diagnostics {
        let severity_label = match diag.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
        };

        let _ = writeln!(
            w,
            "{}: {} [{}]",
            severity_label,
            diag.message,
            diag.rule.to_string()
        );
        let _ = writeln!(w, "  --> {}:{}:{}", diag.file, diag.line, diag.column);

        if let Some(ref help) = diag.help {
            let _ = writeln!(w, "  help: {}", help);
        }

        let _ = writeln!(w);
    }
}

fn print_json(diagnostics: &[LintDiagnostic], w: &mut dyn Write) {
    let json = serde_json::to_string_pretty(diagnostics).unwrap_or_else(|e| {
        eprintln!("Failed to serialize diagnostics to JSON: {}", e);
        "[]".to_string()
    });
    let _ = writeln!(w, "{}", json);
}
