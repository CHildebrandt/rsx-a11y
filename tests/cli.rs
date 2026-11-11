//! End-to-end tests of the `rsx-a11y` binary: exit codes, output
//! destinations, formats and flags.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsx-a11y"))
}

fn run(args: &[&str]) -> Output {
    bin()
        .args(args)
        .env_remove("NO_COLOR")
        .output()
        .expect("failed to run rsx-a11y")
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// A fresh scratch directory under the target dir for one test.
fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(dir: &Path, file: &str, contents: &str) -> String {
    let path = dir.join(file);
    std::fs::write(&path, contents).unwrap();
    path.to_string_lossy().into_owned()
}

#[test]
fn errors_exit_1_and_print_the_full_report() {
    let out = run(&["tests/fixtures/yew_component.rs"]);
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("error: <img> element is missing an `alt` attribute. [alt-text]"));
    // The summary is the last thing written; it used to be lost because the
    // buffered writer was never flushed before exiting.
    assert!(
        text.contains("Checked 1 file in"),
        "summary missing:\n{text}"
    );
    assert!(!text.contains('\u{1b}'), "no ANSI colors when piped");
}

#[test]
fn clean_code_exits_0() {
    let out = run(&["tests/fixtures/accessible_components.rs"]);
    assert_eq!(out.status.code(), Some(0), "{}", stdout(&out));
    assert!(stdout(&out).contains("No accessibility issues found!"));
}

#[test]
fn warnings_only_exit_0() {
    let dir = scratch("warnings_only");
    let file = write(
        &dir,
        "w.rs",
        r#"fn c() { html! { <input autofocus aria-label="Search" /> } }"#,
    );
    let out = run(&[&file]);
    assert_eq!(out.status.code(), Some(0));
    assert!(stdout(&out).contains("[no-autofocus]"));
}

#[test]
fn quiet_shows_only_errors_and_still_prints_summary() {
    let out = run(&["--quiet", "tests/fixtures/yew_component.rs"]);
    assert_eq!(out.status.code(), Some(1));
    let text = stdout(&out);
    assert!(text.contains("error:"));
    assert!(!text.contains("warning:"));
    assert!(text.contains("Checked 1 file"));
}

#[test]
fn json_output_is_complete_and_valid() {
    let out = run(&["--format", "json", "tests/fixtures"]);
    assert_eq!(out.status.code(), Some(1));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("valid JSON");
    let diagnostics = v["diagnostics"].as_array().unwrap();
    assert!(!diagnostics.is_empty());
    assert_eq!(v["summary"]["files_scanned"], 3);
    assert_eq!(
        v["summary"]["errors"].as_u64().unwrap() as usize,
        diagnostics
            .iter()
            .filter(|d| d["severity"] == "error")
            .count()
    );
    assert!(stderr(&out).is_empty(), "JSON mode keeps stderr quiet");
}

#[test]
fn out_file_receives_the_report() {
    let dir = scratch("out_file");
    let report = dir.join("report.txt");
    let out = run(&[
        "--out-file",
        report.to_str().unwrap(),
        "tests/fixtures/yew_component.rs",
    ]);
    assert_eq!(out.status.code(), Some(1));
    assert!(stdout(&out).is_empty());
    let text = std::fs::read_to_string(&report).unwrap();
    assert!(text.contains("[alt-text]"));
    assert!(text.contains("Checked 1 file"));
}

#[test]
fn unknown_rule_names_are_rejected() {
    let out = run(&["--only", "alt-txt", "tests/fixtures"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("unknown rule 'alt-txt'"));
}

#[test]
fn only_and_skip_filter_rules() {
    let out = run(&["--format", "json", "--only", "alt-text", "tests/fixtures"]);
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(
        v["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .all(|d| d["rule"] == "alt-text")
    );

    let out = run(&[
        "--format",
        "json",
        "--skip",
        "alt-text,aria-role",
        "tests/fixtures",
    ]);
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(
        v["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .all(|d| d["rule"] != "alt-text" && d["rule"] != "aria-role")
    );
}

#[test]
fn unparseable_markup_exits_2_and_is_reported() {
    let dir = scratch("parse_error");
    write(&dir, "bad.rs", "fn c() {\n    html! { <div></span> }\n}\n");
    let out = run(&[dir.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(2));
    let text = stdout(&out);
    assert!(text.contains("Invalid markup in"), "{text}");
    assert!(text.contains("bad.rs:2"), "error has a line number: {text}");
    assert!(text.contains("could not be parsed"));
}

#[test]
fn ignore_comments_suppress_diagnostics() {
    let dir = scratch("ignore");
    let file = write(
        &dir,
        "ignored.rs",
        r#"fn c() {
    html! {
        <div>
            // rsx-a11y-ignore: alt-text -- decorative, alt set by CSS
            <img src="a.png" />
            <img src="b.png" />
        </div>
    }
}
"#,
    );
    let out = run(&["--format", "json", &file]);
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let lines: Vec<u64> = v["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["line"].as_u64().unwrap())
        .collect();
    assert_eq!(lines, [6], "only the second image is reported");
}

#[test]
fn missing_path_exits_2() {
    let out = run(&["does/not/exist"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("does not exist"));
}

#[test]
fn list_rules_prints_every_rule() {
    let out = run(&["--list-rules"]);
    assert_eq!(out.status.code(), Some(0));
    let text = stdout(&out);
    for rule in rsx_a11y::lints::Rule::all() {
        assert!(text.contains(rule.as_str()), "{rule} missing");
    }
}

/// Snapshot of every diagnostic (rule, position, message) for the bad
/// fixtures. Run with `UPDATE_SNAPSHOTS=1` to accept intended changes.
#[test]
fn diagnostics_snapshot() {
    let out = run(&["--format", "json", "tests/fixtures"]);
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let actual: String = v["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| {
            format!(
                "{}:{}:{} {} [{}] {}\n",
                d["file"].as_str().unwrap().replace('\\', "/"),
                d["line"],
                d["column"],
                d["severity"].as_str().unwrap(),
                d["rule"].as_str().unwrap(),
                d["message"].as_str().unwrap()
            )
        })
        .collect();
    let path = Path::new("tests/snapshots/fixtures.txt");
    if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, &actual).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(path)
        .expect("missing snapshot; run with UPDATE_SNAPSHOTS=1")
        .replace("\r\n", "\n");
    assert_eq!(
        actual, expected,
        "diagnostics changed; run with UPDATE_SNAPSHOTS=1 if intended"
    );
}
