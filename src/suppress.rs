//! `// rsx-a11y-ignore` comments for silencing individual diagnostics.
//!
//! ```text
//! // rsx-a11y-ignore                      every rule, next line
//! // rsx-a11y-ignore: alt-text, no-autofocus -- reason
//! <img src="x.png" /> // rsx-a11y-ignore  trailing: this line
//! // rsx-a11y-ignore-file: no-autofocus   whole file
//! ```
//!
//! A line directive applies to the next line of code (or, when written after
//! code, to its own line). It suppresses diagnostics reported on that line
//! *and* diagnostics on any element that starts on that line, so one comment
//! covers an element whose attributes span several lines. Anything after
//! `--` is a free-form reason. Unknown rule names are reported as errors.

use std::collections::HashMap;

use crate::lints::Rule;
use crate::parser::{ParseError, ParseErrorKind};

const LINE_DIRECTIVE: &str = "rsx-a11y-ignore";
const FILE_DIRECTIVE: &str = "rsx-a11y-ignore-file";

/// Which rules a directive silences.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
enum Scope {
    #[default]
    Nothing,
    Rules(Vec<Rule>),
    All,
}

impl Scope {
    fn covers(&self, rule: Rule) -> bool {
        match self {
            Scope::Nothing => false,
            Scope::Rules(rules) => rules.contains(&rule),
            Scope::All => true,
        }
    }

    fn merge(&mut self, other: Scope) {
        *self = match (std::mem::take(self), other) {
            (Scope::All, _) | (_, Scope::All) => Scope::All,
            (Scope::Rules(mut a), Scope::Rules(b)) => {
                a.extend(b);
                Scope::Rules(a)
            }
            (Scope::Nothing, s) | (s, Scope::Nothing) => s,
        };
    }
}

/// The `rsx-a11y-ignore` directives found in one source file.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Suppressions {
    file: Scope,
    lines: HashMap<usize, Scope>,
}

impl Suppressions {
    /// Whether `rule` is suppressed for a diagnostic related to any of
    /// `lines` (1-based).
    pub fn is_suppressed(&self, rule: Rule, lines: &[usize]) -> bool {
        self.file.covers(rule)
            || lines
                .iter()
                .any(|line| self.lines.get(line).is_some_and(|s| s.covers(rule)))
    }

    /// Whether the file contains no directives.
    pub fn is_empty(&self) -> bool {
        self.file == Scope::Nothing && self.lines.is_empty()
    }

    /// Find the directives in `source`, reporting malformed ones as errors.
    pub(crate) fn parse(source: &str, file: &str) -> (Self, Vec<ParseError>) {
        let mut out = Suppressions::default();
        let mut errors = Vec::new();
        let lines: Vec<&str> = source.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            let Some((code_before, comment)) = split_comment(line) else {
                continue;
            };
            let (is_file, rest) = if let Some(rest) = comment.strip_prefix(FILE_DIRECTIVE) {
                (true, rest)
            } else if let Some(rest) = comment.strip_prefix(LINE_DIRECTIVE) {
                (false, rest)
            } else {
                continue;
            };
            let line_no = i + 1;
            let scope = match parse_scope(rest) {
                Some(Ok(scope)) => scope,
                // Not a directive after all, e.g. `rsx-a11y-ignored`.
                None => continue,
                Some(Err(unknown)) => {
                    errors.push(
                        ParseError::new(
                            ParseErrorKind::Directive,
                            file,
                            format!(
                                "unknown rule `{unknown}` in `{}` comment",
                                if is_file {
                                    FILE_DIRECTIVE
                                } else {
                                    LINE_DIRECTIVE
                                }
                            ),
                        )
                        .at_line(line_no),
                    );
                    continue;
                }
            };
            if is_file {
                out.file.merge(scope);
                continue;
            }
            let target = if code_before.trim().is_empty() {
                next_code_line(&lines, i + 1)
            } else {
                Some(line_no)
            };
            if let Some(target) = target {
                out.lines.entry(target).or_default().merge(scope);
            }
        }
        (out, errors)
    }
}

/// Split a line into the code before a `//` or `/* … */` comment and the
/// trimmed comment text.
fn split_comment(line: &str) -> Option<(&str, &str)> {
    let (start, body) = match (line.find("//"), line.find("/*")) {
        (Some(a), Some(b)) if b < a => (b, line[b + 2..].split("*/").next().unwrap_or("")),
        (Some(a), _) => (a, &line[a + 2..]),
        (None, Some(b)) => (b, line[b + 2..].split("*/").next().unwrap_or("")),
        (None, None) => return None,
    };
    // `///` and `//!` doc comments are fine too.
    let body = body.trim_start_matches(['/', '!']).trim();
    Some((&line[..start], body))
}

/// Parse what follows the directive keyword: nothing (all rules), `: rules`,
/// and an optional `-- reason`. `None` if this is not actually a directive.
fn parse_scope(rest: &str) -> Option<Result<Scope, String>> {
    let rest = rest.split("--").next().unwrap_or("");
    if rest.trim().is_empty() {
        return Some(Ok(Scope::All));
    }
    let list = rest.trim_start().strip_prefix(':')?;
    let mut rules = Vec::new();
    for name in list.split(',').map(str::trim).filter(|n| !n.is_empty()) {
        match name.parse::<Rule>() {
            Ok(rule) => rules.push(rule),
            Err(_) => return Some(Err(name.to_string())),
        }
    }
    Some(Ok(if rules.is_empty() {
        Scope::All
    } else {
        Scope::Rules(rules)
    }))
}

/// The first line at or after index `from` that contains code.
fn next_code_line(lines: &[&str], from: usize) -> Option<usize> {
    lines[from..]
        .iter()
        .position(|l| {
            let t = l.trim();
            !t.is_empty() && !t.starts_with("//") && !t.starts_with("/*")
        })
        .map(|offset| from + offset + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(source: &str) -> Suppressions {
        let (s, errors) = Suppressions::parse(source, "t.rs");
        assert!(errors.is_empty(), "{errors:?}");
        s
    }

    #[test]
    fn line_directive_targets_next_code_line() {
        let s = parse("a\n// rsx-a11y-ignore: alt-text -- decorative\n\n  // more\n<img />\n");
        assert!(s.is_suppressed(Rule::AltText, &[5]));
        assert!(!s.is_suppressed(Rule::AltText, &[3]));
        assert!(!s.is_suppressed(Rule::NoAutofocus, &[5]));
    }

    #[test]
    fn trailing_directive_targets_its_own_line() {
        let s = parse("<input autofocus /> // rsx-a11y-ignore\n<input autofocus />");
        assert!(s.is_suppressed(Rule::NoAutofocus, &[1]));
        assert!(!s.is_suppressed(Rule::NoAutofocus, &[2]));
    }

    #[test]
    fn block_comment_and_file_directive() {
        let s = parse("/* rsx-a11y-ignore-file: no-autofocus, scope */\nx");
        assert!(s.is_suppressed(Rule::NoAutofocus, &[99]));
        assert!(s.is_suppressed(Rule::Scope, &[1]));
        assert!(!s.is_suppressed(Rule::AltText, &[1]));
    }

    #[test]
    fn unknown_rule_is_an_error() {
        let (_, errors) = Suppressions::parse("// rsx-a11y-ignore: alt-txt\nx", "t.rs");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line, Some(1));
        assert!(errors[0].message.contains("alt-txt"));
    }

    #[test]
    fn similar_words_are_not_directives() {
        let s = parse("// rsx-a11y-ignored is not a directive\nx");
        assert!(s.is_empty());
    }
}
