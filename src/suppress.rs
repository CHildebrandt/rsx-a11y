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
//! code or followed by code on the same line, to that line). Comment markers
//! inside string and char literals are not comments. It suppresses diagnostics reported on that line
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
        let (comments, code_lines) = scan(source);

        for comment in &comments {
            // `///`, `//!`, `/**` and `/*!` doc comments are fine too. Only
            // the first line of a block comment can hold a directive.
            let body = comment
                .text
                .trim_start_matches(['/', '!', '*'])
                .trim_start();
            let body = body.lines().next().unwrap_or("").trim();
            let body = body.strip_suffix("*/").unwrap_or(body);
            let (is_file, rest) = if let Some(rest) = body.strip_prefix(FILE_DIRECTIVE) {
                (true, rest)
            } else if let Some(rest) = body.strip_prefix(LINE_DIRECTIVE) {
                (false, rest)
            } else {
                continue;
            };
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
                        .at_line(comment.start_line),
                    );
                    continue;
                }
            };
            if is_file {
                out.file.merge(scope);
                continue;
            }
            let target = if comment.code_before {
                Some(comment.start_line)
            } else if comment.code_after {
                Some(comment.end_line)
            } else {
                // The next line with code; `code_lines` is 0-based.
                (comment.end_line..code_lines.len())
                    .find(|&i| code_lines[i])
                    .map(|i| i + 1)
            };
            if let Some(target) = target {
                out.lines.entry(target).or_default().merge(scope);
            }
        }
        (out, errors)
    }
}

/// A comment in the source.
#[derive(Debug)]
struct Comment {
    /// The text after `//`, or between `/*` and `*/`.
    text: String,
    /// The lines it starts and ends on (1-based).
    start_line: usize,
    end_line: usize,
    /// Whether there is code before it on its first line.
    code_before: bool,
    /// Whether there is code after it on its last line.
    code_after: bool,
}

/// Lex `source` just enough to find its comments: string, raw string, byte
/// string and char literals are skipped, and block comments nest. Also
/// returns, for each line (0-based), whether it has code outside comments.
fn scan(source: &str) -> (Vec<Comment>, Vec<bool>) {
    let chars: Vec<char> = source.chars().collect();
    let mut lexer = Lexer {
        line: 1,
        code_lines: vec![false],
    };
    let mut comments: Vec<Comment> = Vec::new();
    // Comments that ended on the current line, with no code after them yet.
    let mut ended: Vec<usize> = Vec::new();
    let is_ident = |c: char| c.is_alphanumeric() || c == '_';
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        if c == '\n' {
            lexer.newline();
            ended.clear();
            i += 1;
            continue;
        }
        if c == '/' && next == Some('/') {
            let end = chars[i..]
                .iter()
                .position(|&c| c == '\n')
                .map_or(chars.len(), |n| i + n);
            comments.push(Comment {
                text: chars[i + 2..end].iter().collect(),
                start_line: lexer.line,
                end_line: lexer.line,
                code_before: lexer.has_code(),
                code_after: false,
            });
            i = end;
            continue;
        }
        if c == '/' && next == Some('*') {
            let (start_line, code_before) = (lexer.line, lexer.has_code());
            let body_start = i + 2;
            let mut body_end = chars.len();
            let mut depth = 1;
            let mut j = body_start;
            while j < chars.len() {
                match (chars[j], chars.get(j + 1)) {
                    ('/', Some('*')) => {
                        depth += 1;
                        j += 2;
                    }
                    ('*', Some('/')) => {
                        depth -= 1;
                        j += 2;
                        if depth == 0 {
                            body_end = j - 2;
                            break;
                        }
                    }
                    ('\n', _) => {
                        lexer.newline();
                        j += 1;
                    }
                    _ => j += 1,
                }
            }
            if lexer.line != start_line {
                ended.clear();
            }
            ended.push(comments.len());
            comments.push(Comment {
                text: chars[body_start..body_end].iter().collect(),
                start_line,
                end_line: lexer.line,
                code_before,
                code_after: false,
            });
            i = j;
            continue;
        }

        // Anything else is code.
        if !c.is_whitespace() {
            lexer.mark_code();
            for &k in &ended {
                comments[k].code_after = true;
            }
            ended.clear();
        }
        if c == '"' {
            i = lexer.skip_string(&chars, i + 1);
            continue;
        }
        // Raw strings: r"…", r#"…"#, br"…", cr"…" (but not identifiers
        // ending in `r`, or raw identifiers like `r#type`).
        let starts_raw = c == 'r'
            && match i.checked_sub(1).map(|p| chars[p]) {
                None => true,
                Some('b' | 'c') => i < 2 || !is_ident(chars[i - 2]),
                Some(prev) => !is_ident(prev),
            };
        if starts_raw {
            let hashes = chars[i + 1..].iter().take_while(|&&c| c == '#').count();
            if chars.get(i + 1 + hashes) == Some(&'"') {
                i = lexer.skip_raw_string(&chars, i + 2 + hashes, hashes);
                continue;
            }
        }
        // Char literals ('x', '\n', '\u{…}'), as opposed to lifetimes ('a).
        if c == '\'' {
            if next == Some('\\') {
                let rest = chars.get(i + 3..).unwrap_or_default();
                let close = rest.iter().position(|&c| c == '\'' || c == '\n');
                i = close.map_or(chars.len(), |n| i + 3 + n + 1);
                continue;
            }
            if chars.get(i + 2) == Some(&'\'') {
                i += 3;
                continue;
            }
        }
        i += 1;
    }
    (comments, lexer.code_lines)
}

/// Line bookkeeping for [`scan`].
struct Lexer {
    line: usize,
    code_lines: Vec<bool>,
}

impl Lexer {
    fn newline(&mut self) {
        self.line += 1;
        self.code_lines.push(false);
    }

    fn has_code(&self) -> bool {
        self.code_lines[self.line - 1]
    }

    fn mark_code(&mut self) {
        self.code_lines[self.line - 1] = true;
    }

    /// Note a character inside a literal, which is code.
    fn literal_char(&mut self, c: char) {
        if c == '\n' {
            self.newline();
        } else if !c.is_whitespace() {
            self.mark_code();
        }
    }

    /// Skip the rest of a string literal starting at `j`; returns the index
    /// after its closing quote.
    fn skip_string(&mut self, chars: &[char], mut j: usize) -> usize {
        while j < chars.len() {
            match chars[j] {
                '"' => return j + 1,
                '\\' => {
                    self.mark_code();
                    if let Some(&escaped) = chars.get(j + 1) {
                        self.literal_char(escaped);
                    }
                    j += 2;
                }
                c => {
                    self.literal_char(c);
                    j += 1;
                }
            }
        }
        j
    }

    /// Skip the rest of a raw string literal with `hashes` `#`s.
    fn skip_raw_string(&mut self, chars: &[char], mut j: usize, hashes: usize) -> usize {
        while j < chars.len() {
            if chars[j] == '"'
                && chars[j + 1..]
                    .iter()
                    .take(hashes)
                    .filter(|&&c| c == '#')
                    .count()
                    == hashes
            {
                return j + 1 + hashes;
            }
            self.literal_char(chars[j]);
            j += 1;
        }
        j
    }
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
    fn comment_markers_in_literals_are_not_comments() {
        // The URL's `//` must not hide the trailing directive.
        let s = parse(
            "<img src=\"https://example.com/a.png\" /> // rsx-a11y-ignore: alt-text\n<img />",
        );
        assert!(s.is_suppressed(Rule::AltText, &[1]));
        assert!(!s.is_suppressed(Rule::AltText, &[2]));
        // Directives inside strings are data.
        let s =
            parse("let t = r#\"/* rsx-a11y-ignore-file */\"#;\nlet u = \"// rsx-a11y-ignore\";\nx");
        assert!(s.is_empty(), "{s:?}");
        let s = parse("let s = b\"\\\"// rsx-a11y-ignore\";\nx");
        assert!(s.is_empty(), "{s:?}");
        // Char literals and lifetimes do not start strings.
        let s = parse(
            "let q = '\"'; // rsx-a11y-ignore\nfn f<'a>(x: &'a str) {} // rsx-a11y-ignore: scope",
        );
        assert!(s.is_suppressed(Rule::AltText, &[1]));
        assert!(s.is_suppressed(Rule::Scope, &[2]));
    }

    #[test]
    fn unterminated_literals_do_not_panic() {
        for source in ["'\\", "'", "\"abc", "r#\"abc\"", "/* open", "b'", "r#"] {
            let _ = Suppressions::parse(source, "t.rs");
        }
    }

    #[test]
    fn directive_skips_multiline_block_comments() {
        let s = parse("// rsx-a11y-ignore\n/* a\n   b\n*/\n<img />\n<img />");
        assert!(s.is_suppressed(Rule::AltText, &[5]));
        assert!(!s.is_suppressed(Rule::AltText, &[2, 3, 4, 6]));
    }

    #[test]
    fn code_after_a_block_comment_counts() {
        let s = parse("// rsx-a11y-ignore\n/* note */ <img />\n<img />");
        assert!(s.is_suppressed(Rule::AltText, &[2]));
        assert!(!s.is_suppressed(Rule::AltText, &[3]));
        let s = parse("/* rsx-a11y-ignore */ <img />\n<img />");
        assert!(s.is_suppressed(Rule::AltText, &[1]));
        assert!(!s.is_suppressed(Rule::AltText, &[2]));
    }

    #[test]
    fn multiline_block_directive_uses_first_line() {
        let s = parse("/* rsx-a11y-ignore: alt-text\n   decorative */\n<img />");
        assert!(s.is_suppressed(Rule::AltText, &[3]));
        assert!(!s.is_suppressed(Rule::NoAutofocus, &[3]));
    }

    #[test]
    fn similar_words_are_not_directives() {
        let s = parse("// rsx-a11y-ignored is not a directive\nx");
        assert!(s.is_empty());
    }
}
