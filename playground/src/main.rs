use leptos::prelude::*;
use rsx_a11y::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

/// A supported framework and the example shown when it is selected.
struct Framework {
    name: &'static str,
    macro_name: &'static str,
    example: &'static str,
}

const FRAMEWORKS: &[Framework] = &[
    Framework {
        name: "Yew",
        macro_name: "html!",
        example: r##"fn component() -> Html {
    html! {
        <div>
            <img src="photo.jpg" />
            <div role="fake-role" aria-hidden="yes">
                <div onclick={handler}>{"Click me"}</div>
                <marquee>{"Breaking news!"}</marquee>
                <a href="#">{"Link"}</a>
                <h1></h1>
            </div>
            <label>{"Email"}</label>
            <input type="email" />
        </div>
    }
}"##,
    },
    Framework {
        name: "Leptos",
        macro_name: "view!",
        example: r##"#[component]
fn Component() -> impl IntoView {
    view! {
        <img src="photo.jpg" />
        <div role="fake-role" aria-hidden="yes">
            <div on:click=move |_| handler()>"Click me"</div>
            <marquee>"Breaking news!"</marquee>
            <a href="#">"Link"</a>
            <h1></h1>
        </div>
        <label>"Email"</label>
        <input type="email" />
    }
}"##,
    },
    Framework {
        name: "Dioxus",
        macro_name: "rsx!",
        example: r##"fn Component() -> Element {
    rsx! {
        img { src: "photo.jpg" }
        div { role: "fake-role", aria_hidden: "yes",
            div { onclick: move |_| handler(), "Click me" }
            marquee { "Breaking news!" }
            a { href: "#", "Link" }
            h1 {}
        }
        label { "Email" }
        input { r#type: "email" }
    }
}"##,
    },
];

/// Run the linter on editor code (should be a complete Rust snippet).
fn lint_snippet(code: &str) -> LintResult {
    match rsx_a11y::parser::parse_source(code, "<playground>") {
        Ok(parsed) => {
            let diagnostics = rsx_a11y::lints::lint_parsed(&parsed, Rule::all());
            let errors: Vec<String> = parsed.errors.iter().map(|e| e.to_string()).collect();
            LintResult {
                diagnostics,
                elements_found: parsed.elements.len(),
                error: (!errors.is_empty()).then(|| errors.join("\n")),
            }
        }
        Err(e) => LintResult {
            diagnostics: vec![],
            elements_found: 0,
            error: Some(e.to_string()),
        },
    }
}

#[derive(Clone, PartialEq)]
struct LintResult {
    diagnostics: Vec<LintDiagnostic>,
    elements_found: usize,
    error: Option<String>,
}

impl LintResult {
    /// One-line status, announced to screen readers when it changes.
    fn status(&self) -> String {
        let plural = |n: usize, word: &str| format!("{n} {word}{}", if n == 1 { "" } else { "s" });
        let mut status = format!(
            "{}, {}",
            plural(self.elements_found, "element"),
            plural(self.diagnostics.len(), "issue")
        );
        if self.error.is_some() {
            status.push_str(", parse error");
        }
        status
    }
}

/// Split text into plain-text and URL segments, rendering URLs as clickable `<a>` elements.
fn linkify(text: &str) -> Vec<AnyView> {
    let mut parts: Vec<AnyView> = Vec::new();
    let mut remaining = text;
    while let Some(start) = remaining
        .find("https://")
        .or_else(|| remaining.find("http://"))
    {
        if start > 0 {
            parts.push(remaining[..start].to_string().into_any());
        }
        let after = &remaining[start..];
        let end = after
            .find(|c: char| c.is_whitespace())
            .unwrap_or(after.len());
        let url = after[..end].to_string();
        let href = url.clone();
        parts.push(
            view! {
                <a class="help-link" href=href target="_blank" rel="noopener noreferrer">
                    {url}
                    <span class="visually-hidden">" (opens in a new tab)"</span>
                </a>
            }
            .into_any(),
        );
        remaining = &after[end..];
    }
    if !remaining.is_empty() {
        parts.push(remaining.to_string().into_any());
    }
    parts
}

#[component]
fn App() -> impl IntoView {
    let (code, set_code) = signal(FRAMEWORKS[0].example.to_string());
    let (framework_idx, set_framework_idx) = signal(0usize);

    let result = Memo::new(move |_| lint_snippet(&code.get()));

    let select_framework = move |i: usize| {
        let old_idx = framework_idx.get_untracked();
        if old_idx == i {
            return;
        }
        let (old, new) = (&FRAMEWORKS[old_idx], &FRAMEWORKS[i]);
        set_code.update(|c| {
            *c = if c.as_str() == old.example {
                // Untouched example: show the new framework's example.
                new.example.to_string()
            } else {
                // User code: just swap the macro name.
                c.replacen(old.macro_name, new.macro_name, 1)
            };
        });
        set_framework_idx.set(i);
    };

    view! {
        <a class="skip-link" href="#code-editor">"Skip to editor"</a>
        <div class="app">
            <header class="header">
                <img src="logo.png" alt="" class="logo" />
                <div>
                    <h1>"rsx-a11y Playground"</h1>
                    <p class="subtitle">
                        "Paste a Yew, Leptos or Dioxus component to check it for accessibility issues"
                    </p>
                </div>
            </header>

            <main>
                <div class="toolbar">
                    <div class="framework-picker" role="group" aria-labelledby="framework-label">
                        <span id="framework-label" class="toolbar-label">
                            "Framework:"
                        </span>
                        <div class="framework-tabs">
                            {FRAMEWORKS
                                .iter()
                                .enumerate()
                                .map(|(i, framework)| {
                                    view! {
                                        <button
                                            type="button"
                                            class:active=move || framework_idx.get() == i
                                            aria-pressed=move || {
                                                if framework_idx.get() == i { "true" } else { "false" }
                                            }
                                            on:click=move |_| select_framework(i)
                                        >
                                            {framework.name}
                                        </button>
                                    }
                                })
                                .collect::<Vec<_>>()}
                        </div>
                    </div>
                    <p class="stats" role="status" aria-live="polite">
                        {move || result.get().status()}
                    </p>
                </div>

                <div class="editor-container">
                    <section class="panel" aria-labelledby="input-heading">
                        <h2 class="panel-header" id="input-heading">
                            <label for="code-editor">"Input"</label>
                        </h2>
                        <textarea
                            id="code-editor"
                            class="code-editor"
                            prop:value=move || code.get()
                            on:input=move |ev| {
                                let target = event_target::<web_sys::HtmlTextAreaElement>(&ev);
                                set_code.set(target.value());
                            }
                            spellcheck="false"
                            autocomplete="off"
                            aria-describedby="editor-hint"
                        ></textarea>
                        <p id="editor-hint" class="visually-hidden">
                            "Results update as you type. Press Tab to leave the editor."
                        </p>
                    </section>
                    <section class="panel" aria-labelledby="results-heading">
                        <h2 class="panel-header" id="results-heading">"Results"</h2>
                        <div class="results">
                            {move || {
                                let r = result.get();
                                let error = r
                                    .error
                                    .clone()
                                    .map(|err| {
                                        view! {
                                            <div class="parse-error">
                                                <strong>"Parse error: "</strong>
                                                {err}
                                            </div>
                                        }
                                    });
                                if r.diagnostics.is_empty() {
                                    let none = r
                                        .error
                                        .is_none()
                                        .then(|| {
                                            view! {
                                                <p class="no-issues">
                                                    <span aria-hidden="true">"✓ "</span>
                                                    "No accessibility issues found!"
                                                </p>
                                            }
                                        });
                                    return view! { {error} {none} }.into_any();
                                }
                                view! {
                                    {error}
                                    <ul class="diagnostics-list">
                                        {r
                                            .diagnostics
                                            .iter()
                                            .map(|d| {
                                                let severity_class = match d.severity {
                                                    Severity::Error => "severity-error",
                                                    Severity::Warning => "severity-warning",
                                                    Severity::Info => "severity-info",
                                                };
                                                let help = d.help.clone();
                                                view! {
                                                    <li class="diagnostic">
                                                        <p class="diag-header">
                                                            <span class=severity_class>
                                                                {d.severity.to_string()}
                                                            </span>
                                                            " "
                                                            <code class="lint-id">{d.rule.to_string()}</code>
                                                        </p>
                                                        <p class="diag-message">{d.message.clone()}</p>
                                                        <p class="diag-location">
                                                            {format!("Line {}, column {}", d.line, d.column)}
                                                        </p>
                                                        {help
                                                            .map(|h| {
                                                                view! {
                                                                    <p class="diag-help">
                                                                        <span class="help-label">"Help: "</span>
                                                                        {linkify(&h)}
                                                                    </p>
                                                                }
                                                            })}
                                                    </li>
                                                }
                                            })
                                            .collect::<Vec<_>>()}
                                    </ul>
                                }
                                    .into_any()
                            }}
                        </div>
                    </section>
                </div>
            </main>
            <footer class="footer">
                "Powered by "
                <a
                    href="https://github.com/CHildebrandt/rsx-a11y"
                    target="_blank"
                    rel="noopener noreferrer"
                >
                    "rsx-a11y"
                    <span class="visually-hidden">" (opens in a new tab)"</span>
                </a>
                " — ARIA linting for Rust web frameworks"
            </footer>
        </div>
    }
}
