//! rsx-a11y: a linter for ARIA and accessibility issues in Rust web
//! frameworks (Yew, Leptos, Dioxus).
//!
//! The crate parses Rust source files, finds the markup inside `html!`,
//! `view!` and `rsx!` macro invocations, and checks it against
//! WAI-ARIA 1.2 and common accessibility rules inspired by
//! [`eslint-plugin-jsx-a11y`](https://github.com/jsx-eslint/eslint-plugin-jsx-a11y).
//! See [`Rule`] for the full list of rules.
//!
//! # Linting a project in a test
//!
//! ```rust,no_run
//! use rsx_a11y::prelude::*;
//!
//! let summary = check_project(concat!(env!("CARGO_MANIFEST_DIR"), "/src")).unwrap();
//! assert!(summary.is_clean(), "accessibility issues found:\n{summary}");
//! ```
//!
//! # Linting a snippet
//!
//! ```rust
//! use rsx_a11y::prelude::*;
//!
//! let diagnostics = check_source(r#"fn app() { html! { <img src="cat.png" /> } }"#, "app.rs").unwrap();
//! assert_eq!(diagnostics[0].rule, Rule::AltText);
//! ```

#![warn(missing_docs)]

pub mod diagnostics;
pub mod dom;
pub mod lints;
pub mod parser;
pub mod prelude;

use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use lints::{LintDiagnostic, Rule, Severity};
use parser::ParseError;

/// Which diagnostics to keep.
///
/// The default keeps every diagnostic from every rule.
///
/// ```
/// use rsx_a11y::prelude::*;
///
/// let options = LintOptions::default()
///     .skip([Rule::NoAutofocus])
///     .min_severity(Severity::Warning);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintOptions {
    only: Option<Vec<Rule>>,
    skip: Vec<Rule>,
    min_severity: Severity,
}

impl Default for LintOptions {
    fn default() -> Self {
        Self {
            only: None,
            skip: Vec::new(),
            min_severity: Severity::Info,
        }
    }
}

impl LintOptions {
    /// Only run the given rules.
    pub fn only(mut self, rules: impl IntoIterator<Item = Rule>) -> Self {
        self.only = Some(rules.into_iter().collect());
        self
    }

    /// Do not run the given rules.
    pub fn skip(mut self, rules: impl IntoIterator<Item = Rule>) -> Self {
        self.skip.extend(rules);
        self
    }

    /// Drop diagnostics less severe than `severity`.
    pub fn min_severity(mut self, severity: Severity) -> Self {
        self.min_severity = severity;
        self
    }

    /// Whether `rule` is enabled by these options.
    pub fn is_enabled(&self, rule: Rule) -> bool {
        self.only.as_ref().is_none_or(|only| only.contains(&rule))
            && !self.skip.contains(&rule)
            && rule.severity() >= self.min_severity
    }

    /// Whether `diagnostic` passes these options.
    pub fn keep(&self, diagnostic: &LintDiagnostic) -> bool {
        self.is_enabled(diagnostic.rule) && diagnostic.severity >= self.min_severity
    }
}

/// The result of linting a set of files.
///
/// Implements [`Display`](fmt::Display) as a plain-text report, which makes
/// it convenient in assertion messages.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct LintSummary {
    /// All diagnostics found, sorted by file → line → column.
    pub diagnostics: Vec<LintDiagnostic>,
    /// Files that could not be read or parsed. These files were **not** linted.
    pub parse_errors: Vec<ParseError>,
    /// Number of `.rs` files that were scanned.
    pub files_scanned: usize,
    /// Number of files that contained at least one lintable element.
    pub files_checked: usize,
}

impl LintSummary {
    /// `true` when there are no error-level diagnostics and every file parsed.
    pub fn is_clean(&self) -> bool {
        self.parse_errors.is_empty() && self.errors().next().is_none()
    }

    /// Diagnostics with [`Severity::Error`].
    pub fn errors(&self) -> impl Iterator<Item = &LintDiagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
    }

    /// Diagnostics produced by `rule`.
    pub fn by_rule(&self, rule: Rule) -> impl Iterator<Item = &LintDiagnostic> {
        self.diagnostics.iter().filter(move |d| d.rule == rule)
    }
}

impl fmt::Display for LintSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for d in &self.diagnostics {
            writeln!(
                f,
                "{}: {} [{}]\n  --> {}:{}:{}",
                d.severity, d.message, d.rule, d.file, d.line, d.column
            )?;
        }
        for e in &self.parse_errors {
            writeln!(f, "error: {e}")?;
        }
        Ok(())
    }
}

/// Lint a single Rust source string with every rule.
///
/// `file_name` is only used to label diagnostics. Diagnostics are sorted by
/// line and column.
pub fn check_source(source: &str, file_name: &str) -> Result<Vec<LintDiagnostic>, ParseError> {
    let elements = parser::parse_source(source, file_name)?;
    let mut diagnostics: Vec<_> = lints::run_all_lints(&elements).collect();
    sort_diagnostics(&mut diagnostics);
    Ok(diagnostics)
}

/// Lint every `.rs` file under `path` (or `path` itself if it is a file)
/// with every rule.
///
/// This is the primary entry point for programmatic / unit-test usage.
/// `target/`, `node_modules/` and dot-directories are skipped. Files that
/// cannot be parsed are reported in [`LintSummary::parse_errors`].
///
/// # Errors
///
/// Returns an error if `path` does not exist or cannot be read, so a typo in
/// the path cannot make a test pass vacuously.
pub fn check_project(path: impl AsRef<Path>) -> io::Result<LintSummary> {
    check_project_with(path, &LintOptions::default())
}

/// Like [`check_project`], keeping only diagnostics allowed by `options`.
///
/// # Errors
///
/// Returns an error if `path` does not exist or cannot be read.
pub fn check_project_with(
    path: impl AsRef<Path>,
    options: &LintOptions,
) -> io::Result<LintSummary> {
    let files = collect_rust_files(path.as_ref())?;
    Ok(check_files(&files, options))
}

/// Lint the given files, keeping only diagnostics allowed by `options`.
///
/// Files are processed in parallel when the `parallel` feature is enabled.
pub fn check_files(files: &[PathBuf], options: &LintOptions) -> LintSummary {
    let lint_one = |file: &PathBuf| -> Result<Option<Vec<LintDiagnostic>>, ParseError> {
        let elements = parser::parse_file(file)?;
        if elements.is_empty() {
            return Ok(None);
        }
        Ok(Some(
            lints::run_all_lints(&elements)
                .filter(|d| options.keep(d))
                .collect(),
        ))
    };

    #[cfg(feature = "parallel")]
    let results: Vec<_> = {
        use rayon::prelude::*;
        files.par_iter().map(lint_one).collect()
    };
    #[cfg(not(feature = "parallel"))]
    let results: Vec<_> = files.iter().map(lint_one).collect();

    let mut summary = LintSummary {
        files_scanned: files.len(),
        ..LintSummary::default()
    };
    for result in results {
        match result {
            Ok(Some(diagnostics)) => {
                summary.files_checked += 1;
                summary.diagnostics.extend(diagnostics);
            }
            Ok(None) => {}
            Err(e) => summary.parse_errors.push(e),
        }
    }
    sort_diagnostics(&mut summary.diagnostics);
    summary.parse_errors.sort_by(|a, b| a.file.cmp(&b.file));
    summary
}

fn sort_diagnostics(diagnostics: &mut [LintDiagnostic]) {
    diagnostics.sort_by(|a, b| {
        (a.file.as_str(), a.line, a.column).cmp(&(b.file.as_str(), b.line, b.column))
    });
}

/// Find every `.rs` file under `path`, in sorted order.
///
/// If `path` is a file, it is returned when it has an `.rs` extension.
/// `target/`, `node_modules/` and dot-directories below `path` are skipped,
/// and symbolic links to directories are not followed (so results always
/// live under `path`).
///
/// # Errors
///
/// Returns an error if `path` does not exist or cannot be read.
pub fn collect_rust_files(path: &Path) -> io::Result<Vec<PathBuf>> {
    let meta = std::fs::metadata(path)?;
    if meta.is_file() {
        let is_rs = path.extension().is_some_and(|ext| ext == "rs");
        return Ok(if is_rs {
            vec![path.to_path_buf()]
        } else {
            Vec::new()
        });
    }
    let mut files = Vec::new();
    collect_recursive(path, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_recursive(dir: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        // `DirEntry::file_type` does not follow symlinks.
        let file_type = entry.file_type()?;
        let path = entry.path();
        if file_type.is_dir() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if matches!(name.as_ref(), "target" | "node_modules") || name.starts_with('.') {
                continue;
            }
            collect_recursive(&path, out)?;
        } else if path.extension().is_some_and(|ext| ext == "rs")
            && (file_type.is_file() || (file_type.is_symlink() && path.is_file()))
        {
            out.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn options_filter_rules_and_severity() {
        let opts = LintOptions::default()
            .only([Rule::AltText, Rule::NoAutofocus])
            .skip([Rule::NoAutofocus]);
        assert!(opts.is_enabled(Rule::AltText));
        assert!(!opts.is_enabled(Rule::NoAutofocus));
        assert!(!opts.is_enabled(Rule::AriaRole));

        let errors_only = LintOptions::default().min_severity(Severity::Error);
        assert!(errors_only.is_enabled(Rule::AltText));
        assert!(!errors_only.is_enabled(Rule::NoAutofocus));
    }

    #[test]
    fn check_project_errors_on_missing_path() {
        let err = check_project("definitely/not/a/real/path").unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn check_source_sorts_by_position() {
        let src = r#"fn c() { html! { <div>
            <img src="b.png" />
            <img src="a.png" />
        </div> } }"#;
        let diags = check_source(src, "x.rs").unwrap();
        assert!(
            diags
                .windows(2)
                .all(|w| (w[0].line, w[0].column) <= (w[1].line, w[1].column))
        );
    }
}
