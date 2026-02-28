//! Test fixture: Yew component with various accessibility issues.
//! This file is used for integration testing.

use yew::prelude::*;

#[function_component(BadComponent)]
fn bad_component() -> Html {
    let onclick = Callback::from(|_| {});

    html! {
        <div>
            // missing-alt-text: img without alt
            <img src="logo.png" />

            // invalid-aria-attribute: aria-foo is not a valid ARIA attribute
            <div aria-foo="bar">{"Content"}</div>

            // invalid-aria-value: aria-hidden expects "true" or "false"
            <div aria-hidden="yes">{"Hidden?"}</div>

            // invalid-role: "banana" is not a valid ARIA role
            <div role="banana">{"Role?"}</div>

            // abstract-role: "widget" is an abstract role
            <span role="widget">{"Abstract"}</span>

            // redundant-role: button already has implicit role "button"
            <button role="button">{"Click me"}</button>

            // no-access-key: accesskey creates keyboard shortcut conflicts
            <button accesskey="s">{"Save"}</button>

            // no-autofocus: autofocus can reduce usability
            <input autofocus="true" />

            // click-events-have-key-events: div with onclick but no keyboard handler
            <div onclick={onclick.clone()}>{"Click this div"}</div>

            // no-positive-tabindex: tabindex > 0 creates unexpected tab order
            <div tabindex="5">{"Tabbable"}</div>

            // no-noninteractive-tabindex: span is non-interactive
            <span tabindex="0">{"Focusable span"}</span>

            // anchor-is-valid: href="#" is not a valid link
            <a href="#">{"Bad link"}</a>

            // no-distracting-elements: marquee should not be used
            <marquee>{"Scrolling text!"}</marquee>

            // iframe-has-title: iframe without title
            <iframe src="https://example.com"></iframe>

            // no-redundant-alt: alt text contains "image"
            <img src="cat.jpg" alt="image of a cat" />

            // heading-has-content: empty heading
            <h1></h1>

            // aria-unsupported-elements: aria on <meta>
            <meta aria-label="test" />

            // scope on non-th element
            <td scope="row">{"Data"}</td>

            // missing-aria-label: input without accessible name
            <input />

            // label without associated control
            <label></label>
        </div>
    }
}

#[function_component(GoodComponent)]
fn good_component() -> Html {
    let onclick = Callback::from(|_| {});
    let onkeydown = Callback::from(|_: KeyboardEvent| {});

    html! {
        <div>
            // Correct: img with alt text
            <img src="logo.png" alt="Company logo" />

            // Correct: valid ARIA attributes
            <div aria-label="Main content" aria-hidden="true" role="main">
                {"Content"}
            </div>

            // Correct: button without redundant role
            <button onclick={onclick.clone()}>{"Click me"}</button>

            // Correct: div with both click and keyboard handlers
            <div onclick={onclick.clone()} onkeydown={onkeydown}>{"Interactive div"}</div>

            // Correct: valid anchor
            <a href="/about">{"About us"}</a>

            // Correct: input with aria-label
            <input aria-label="Search" />

            // Correct: heading with content
            <h1>{"Welcome"}</h1>

            // Correct: iframe with title
            <iframe src="https://example.com" title="Example iframe"></iframe>

            // Correct: img with descriptive alt (no redundant words)
            <img src="cat.jpg" alt="A fluffy orange tabby sleeping on a cushion" />

            // Correct: decorative image
            <img src="decoration.png" role="presentation" />

            // Correct: label with for attribute
            <label for="email">{"Email"}</label>

            // Correct: tabindex 0 on interactive element
            <button tabindex="0">{"Focus me"}</button>

            // Correct: th with scope
            <th scope="col">{"Header"}</th>
        </div>
    }
}
