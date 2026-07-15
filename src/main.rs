mod pi;

use dioxus::{desktop::LogicalSize, prelude::*};

const APP_STYLE: &str = r#"
    :root {
        color-scheme: light dark;
        --nano-background: #ffffff;
        --nano-foreground: #37474f;
        --nano-ink: #173541;
        --nano-text: #556e79;
        --nano-muted-text: #657983;
        --nano-highlight: #fafafa;
        --nano-critical: #ff6f00;
        --nano-critical-text: #173541;
        --nano-salient: #673ab7;
        --nano-strong: #0e2a35;
        --nano-subtle: #eceff1;
        --nano-faded: #b0bec5;
        --nano-badge: #556e79;
        --nano-badge-text: #ffffff;
        --sidebar-width: 136px;
    }

    @media (prefers-color-scheme: dark) {
        :root {
            --nano-background: #2e3440;
            --nano-foreground: #eceff4;
            --nano-ink: #81a1c1;
            --nano-text: #d8dee9;
            --nano-muted-text: #94a4b8;
            --nano-highlight: #3b4252;
            --nano-critical: #ebcb8b;
            --nano-critical-text: #2e3440;
            --nano-salient: #81a1c1;
            --nano-strong: #eceff4;
            --nano-subtle: #434c5e;
            --nano-faded: #677691;
            --nano-badge: #81a1c1;
            --nano-badge-text: #2e3440;
        }
    }

    * {
        box-sizing: border-box;
    }

    html,
    body,
    #main {
        width: 100%;
        height: 100%;
    }

    body {
        margin: 0;
        overflow: hidden;
        background: var(--nano-background);
        color: var(--nano-foreground);
        font-family: "Berkeley Mono", "Roboto Mono", "Fira Code",
            "SFMono-Regular", Menlo, Consolas, monospace;
        font-size: 14px;
        font-variant-ligatures: none;
        line-height: 19px;
        -webkit-font-smoothing: antialiased;
    }

    button,
    input {
        appearance: none;
        font: inherit;
        -webkit-appearance: none;
    }

    ::selection {
        background: var(--nano-subtle);
        color: var(--nano-strong);
    }

    .app {
        position: fixed;
        inset: 18px;
        display: grid;
        grid-template-columns: var(--sidebar-width) minmax(0, 1fr);
        min-width: 0;
        min-height: 0;
        background: var(--nano-background);
    }

    .side-tabs {
        position: relative;
        z-index: 2;
        align-self: start;
        min-width: 0;
    }

    .side-cell {
        position: relative;
        display: flex;
        width: calc(100% + 1px);
        height: 38px;
        margin: 0 -1px -1px 0;
        align-items: center;
        padding: 0 12px;
        overflow: visible;
        border: 1px solid var(--nano-ink);
        background: var(--nano-background);
        color: var(--nano-text);
        white-space: nowrap;
    }

    .side-brand {
        letter-spacing: 0.05em;
    }

    .side-tab {
        color: var(--nano-strong);
        font-weight: 700;
    }

    .side-tab::after {
        position: absolute;
        z-index: 3;
        top: 1px;
        right: -2px;
        bottom: 1px;
        width: 4px;
        background: var(--nano-background);
        content: "";
    }

    .workspace {
        display: grid;
        grid-template-rows: 20px minmax(0, 1fr) auto;
        min-width: 0;
        min-height: 0;
        padding: 10px 20px 16px;
        overflow: hidden;
        border: 1px solid var(--nano-ink);
        background: var(--nano-background);
    }

    .modeline {
        display: grid;
        grid-template-columns: 34px max-content minmax(0, 1fr) max-content;
        min-width: 0;
        height: 20px;
        overflow: hidden;
        background: var(--nano-subtle);
        line-height: 20px;
        white-space: nowrap;
    }

    .modeline-status {
        overflow: hidden;
        background: var(--nano-badge);
        color: var(--nano-badge-text);
        text-align: center;
    }

    .modeline-status.is-running {
        background: var(--nano-critical);
        color: var(--nano-critical-text);
    }

    .modeline-name {
        padding-left: 1.1ch;
        overflow: hidden;
        color: var(--nano-strong);
        font-weight: 700;
        text-overflow: ellipsis;
    }

    .modeline-mode {
        min-width: 0;
        padding-left: 1ch;
        overflow: hidden;
        color: var(--nano-foreground);
        text-overflow: ellipsis;
    }

    .modeline-secondary {
        padding: 0 1ch;
        color: var(--nano-text);
    }

    .buffer {
        position: relative;
        min-width: 0;
        min-height: 0;
        padding: 6px 0 12px;
        overflow: auto;
        background: var(--nano-background);
        scrollbar-color: var(--nano-faded) transparent;
        scrollbar-gutter: stable;
        scrollbar-width: thin;
    }

    .buffer::-webkit-scrollbar {
        width: 8px;
        height: 8px;
    }

    .buffer::-webkit-scrollbar-thumb {
        background: var(--nano-faded);
    }

    .buffer::-webkit-scrollbar-track {
        background: transparent;
    }

    .output {
        min-width: 0;
        margin: 0;
        overflow-wrap: anywhere;
        white-space: pre-wrap;
        color: var(--nano-text);
        font: inherit;
    }

    .working-line {
        display: inline-flex;
        align-items: center;
        gap: 1ch;
        color: var(--nano-text);
    }

    .cursor {
        width: 0.72ch;
        height: 1.05em;
        background: var(--nano-text);
        animation: cursor-blink 1s steps(1, end) infinite;
    }

    @keyframes cursor-blink {
        50% {
            opacity: 0;
        }
    }

    .splash {
        position: absolute;
        inset: 0;
        display: grid;
        place-content: center;
        color: var(--nano-muted-text);
        text-align: left;
    }

    .splash-line {
        white-space: nowrap;
    }

    .splash-title {
        color: var(--nano-strong);
        font-weight: 700;
    }

    .prompt {
        display: grid;
        grid-template-columns: 32px minmax(0, 1fr) auto;
        min-width: 0;
        min-height: 40px;
        border: 1px solid var(--nano-ink);
        background: var(--nano-background);
    }

    .prompt-mark {
        display: grid;
        place-items: center;
        background: var(--nano-subtle);
        color: var(--nano-foreground);
        font-weight: 700;
        user-select: none;
    }

    .prompt input {
        min-width: 0;
        padding: 9px 10px;
        border: 0;
        border-radius: 0;
        outline: 0;
        background: transparent;
        color: var(--nano-strong);
        caret-color: var(--nano-ink);
    }

    .prompt input::placeholder {
        color: var(--nano-muted-text);
        opacity: 1;
    }

    .prompt input:focus-visible {
        box-shadow: inset 0 -2px var(--nano-salient);
    }

    .prompt button {
        min-width: 72px;
        padding: 0 12px;
        border: 0;
        border-left: 1px solid var(--nano-ink);
        border-radius: 0;
        background: var(--nano-ink);
        color: var(--nano-background);
        cursor: pointer;
    }

    .prompt button:not(:disabled):hover {
        background: var(--nano-strong);
    }

    .prompt button:focus-visible {
        outline: 2px solid var(--nano-salient);
        outline-offset: -4px;
    }

    .prompt button:disabled {
        background: var(--nano-subtle);
        color: var(--nano-text);
        cursor: default;
    }

    @media (max-width: 639px) {
        :root {
            --sidebar-width: 112px;
        }

        .app {
            inset: 10px;
        }

        .workspace {
            padding: 10px 12px 12px;
        }

        .modeline-mode {
            visibility: hidden;
        }
    }

    @media (max-width: 479px) {
        .app {
            inset: 0;
            grid-template-columns: minmax(0, 1fr);
            grid-template-rows: 38px minmax(0, 1fr);
        }

        .side-tabs {
            display: flex;
            min-width: 0;
        }

        .side-cell {
            width: auto;
            min-width: 112px;
            flex: 0 0 auto;
            margin: 0 -1px 0 0;
        }

        .side-tab::after {
            top: auto;
            right: 1px;
            bottom: -2px;
            left: 1px;
            width: auto;
            height: 4px;
        }

        .workspace {
            padding: 10px;
        }

        .modeline {
            grid-template-columns: 34px minmax(0, 1fr) max-content;
        }

        .modeline-mode {
            display: none;
        }
    }

    @media (prefers-reduced-motion: reduce) {
        .cursor {
            animation: none;
        }
    }
"#;

fn main() {
    dioxus::LaunchBuilder::desktop()
        .with_cfg(
            dioxus::desktop::Config::new().with_window(
                dioxus::desktop::WindowBuilder::new()
                    .with_title("Crust")
                    .with_inner_size(LogicalSize::new(1095.0, 760.0))
                    .with_min_inner_size(LogicalSize::new(420.0, 360.0)),
            ),
        )
        .launch(App);
}

#[component]
fn App() -> Element {
    let client = use_hook(pi::new_client);
    let mut input = use_signal(String::new);
    let mut output = use_signal(String::new);
    let mut running = use_signal(|| false);
    let mut input_element = use_signal(|| None::<std::rc::Rc<MountedData>>);

    let submit_disabled = running() || input.read().trim().is_empty();
    let submit_label = if running() { "Working…" } else { "Send" };
    let status_label = if running() { "WORKING" } else { "READY" };
    let output_text = output();
    let show_splash = !running() && output_text.is_empty();
    let status_class = if running() {
        "modeline-status is-running"
    } else {
        "modeline-status"
    };

    rsx! {
        style { {APP_STYLE} }
        main { class: "app",
            aside { class: "side-tabs", aria_label: "Buffers",
                div { class: "side-cell side-brand", "CRUST" }
                div { class: "side-cell side-tab", "crust.chat" }
            }
            section { class: "workspace", aria_label: "Crust chat buffer",
                header { class: "modeline",
                    span { class: "{status_class}", "PI" }
                    span { class: "modeline-name", "Crust" }
                    span { class: "modeline-mode", "(agent session)" }
                    span { class: "modeline-secondary", "{status_label}" }
                }
                section { class: "buffer", aria_live: "polite", aria_busy: running(),
                    if show_splash {
                        div { class: "splash",
                            div { class: "splash-line",
                                span { class: "splash-title", "CRUST / P I" }
                                span { " — Pi made simple" }
                            }
                            div { class: "splash-line", "Type a request below to begin." }
                        }
                    } else if running() && output_text.is_empty() {
                        div { class: "working-line",
                            span { "Pi is working" }
                            span { class: "cursor", aria_hidden: "true" }
                        }
                    } else {
                        pre { class: "output", "{output_text}" }
                    }
                }
                form {
                    class: "prompt",
                    onsubmit: move |event| {
                        event.prevent_default();

                        if running() {
                            return;
                        }

                        let message = input().trim().to_owned();
                        if message.is_empty() {
                            return;
                        }

                        let client = client.clone();
                        input.set(String::new());
                        output.set(String::new());
                        running.set(true);

                        spawn(async move {
                            let mut streamed_output = output;
                            let result = pi::prompt(&client, message, move |delta| {
                                streamed_output.write().push_str(delta);
                            })
                            .await;

                            if let Err(error) = result {
                                output.set(format!("Error: {error}"));
                            } else if output.read().is_empty() {
                                output.set("(Pi returned no text output.)".to_owned());
                            }

                            running.set(false);
                            if let Some(input_element) = input_element.cloned() {
                                let _ = input_element.set_focus(true).await;
                            }
                        });
                    },
                    span { class: "prompt-mark", aria_hidden: "true", ">_" }
                    input {
                        aria_label: "Prompt",
                        autocomplete: "off",
                        autofocus: true,
                        placeholder: "Ask Pi…",
                        value: "{input}",
                        oninput: move |event| input.set(event.value()),
                        onmounted: move |element| input_element.set(Some(element.data())),
                    }
                    button {
                        r#type: "submit",
                        disabled: submit_disabled,
                        "{submit_label}"
                    }
                }
            }
        }
    }
}
