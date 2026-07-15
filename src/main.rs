mod pi;

use dioxus::{desktop::LogicalSize, prelude::*};

const APP_STYLE: &str = r#"
    :root {
        color-scheme: light dark;

        /* Doric Light */
        --doric-cursor: #2266bb;
        --doric-bg-main: #ffffff;
        --doric-fg-main: #000000;
        --doric-border: #b0b0b0;
        --doric-fg-shadow-subtle: #5a6268;
        --doric-bg-shadow-intense: #a0bcd0;
        --doric-fg-shadow-intense: #213067;
        --doric-bg-accent: #d8f1f3;
        --doric-fg-accent: #084092;
        --doric-bg-yellow: #f0f0b0;
        --sidebar-width: 136px;
    }

    @media (prefers-color-scheme: dark) {
        :root {
            /* Doric Dark */
            --doric-cursor: #ccaaee;
            --doric-bg-main: #000000;
            --doric-fg-main: #ffffff;
            --doric-border: #707070;
            --doric-fg-shadow-subtle: #a2a0b2;
            --doric-bg-shadow-intense: #50447f;
            --doric-fg-shadow-intense: #cfcff8;
            --doric-bg-accent: #521e40;
            --doric-fg-accent: #cda4df;
            --doric-bg-yellow: #504432;
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
        background: var(--doric-bg-main);
        color: var(--doric-fg-main);
        font-family: "Berkeley Mono", "Roboto Mono", "Fira Code",
            "SFMono-Regular", Menlo, Consolas, monospace;
        font-size: 14px;
        font-variant-ligatures: none;
        line-height: 19px;
        -webkit-font-smoothing: antialiased;
    }

    input {
        appearance: none;
        font: inherit;
        -webkit-appearance: none;
    }

    ::selection {
        background: var(--doric-bg-accent);
        color: var(--doric-fg-main);
    }

    .app {
        position: fixed;
        inset: 18px;
        display: grid;
        grid-template-columns: var(--sidebar-width) minmax(0, 1fr);
        min-width: 0;
        min-height: 0;
        background: var(--doric-bg-main);
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
        border: 1px solid var(--doric-border);
        background: var(--doric-bg-main);
        color: var(--doric-fg-shadow-subtle);
        white-space: nowrap;
    }

    .side-tab {
        color: var(--doric-fg-main);
        font-weight: 700;
    }

    .side-tab::after {
        position: absolute;
        z-index: 3;
        top: 1px;
        right: -2px;
        bottom: 1px;
        width: 4px;
        background: var(--doric-bg-main);
        content: "";
    }

    .workspace {
        display: grid;
        grid-template-rows: 20px minmax(0, 1fr) auto;
        min-width: 0;
        min-height: 0;
        padding: 10px 20px 16px;
        overflow: hidden;
        border: 1px solid var(--doric-border);
        background: var(--doric-bg-main);
    }

    .modeline {
        display: grid;
        grid-template-columns: 34px max-content minmax(0, 1fr) max-content;
        min-width: 0;
        height: 20px;
        overflow: hidden;
        background: var(--doric-bg-shadow-intense);
        color: var(--doric-fg-shadow-intense);
        line-height: 20px;
        white-space: nowrap;
    }

    .modeline-status {
        overflow: hidden;
        background: var(--doric-fg-shadow-intense);
        color: var(--doric-bg-main);
        text-align: center;
    }

    .modeline-status.is-running {
        background: var(--doric-bg-yellow);
        color: var(--doric-fg-main);
    }

    .modeline-status.is-shell {
        background: var(--doric-bg-accent);
        color: var(--doric-fg-accent);
    }

    .modeline-name {
        padding-left: 1.1ch;
        overflow: hidden;
        font-weight: 700;
        text-overflow: ellipsis;
    }

    .modeline-mode {
        min-width: 0;
        padding-left: 1ch;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .modeline-secondary {
        padding: 0 1ch;
    }

    .buffer {
        position: relative;
        min-width: 0;
        min-height: 0;
        padding: 6px 0 12px;
        overflow: auto;
        background: var(--doric-bg-main);
        scrollbar-color: var(--doric-border) transparent;
        scrollbar-gutter: stable;
        scrollbar-width: thin;
    }

    .buffer::-webkit-scrollbar {
        width: 8px;
        height: 8px;
    }

    .buffer::-webkit-scrollbar-thumb {
        background: var(--doric-border);
    }

    .buffer::-webkit-scrollbar-track {
        background: transparent;
    }

    .output {
        min-width: 0;
        margin: 0;
        overflow-wrap: anywhere;
        white-space: pre-wrap;
        color: var(--doric-fg-main);
        font: inherit;
    }

    .working-line {
        display: inline-flex;
        align-items: center;
        gap: 1ch;
        color: var(--doric-fg-main);
    }

    .cursor {
        width: 0.72ch;
        height: 1.05em;
        background: var(--doric-cursor);
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
        color: var(--doric-fg-shadow-subtle);
        text-align: left;
    }

    .splash-title {
        color: var(--doric-fg-main);
        font-weight: 700;
    }

    .prompt {
        display: grid;
        min-width: 0;
        min-height: 40px;
        border: 1px solid var(--doric-border);
        background: var(--doric-bg-main);
    }

    .prompt.is-shell {
        border-color: var(--doric-fg-accent);
    }

    .prompt input {
        min-width: 0;
        padding: 9px 10px;
        border: 0;
        border-radius: 0;
        outline: 0;
        background: transparent;
        color: var(--doric-fg-main);
        caret-color: var(--doric-cursor);
    }

    .prompt input::placeholder {
        color: var(--doric-fg-shadow-subtle);
        opacity: 1;
    }

    .prompt input:focus-visible {
        box-shadow: inset 0 -2px var(--doric-fg-accent);
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

fn render_bash_output(command: &str, outcome: pi::BashOutcome) -> String {
    let mut rendered = format!("$ {command}\n\n");
    if outcome.output.is_empty() {
        rendered.push_str("(no output)");
    } else {
        rendered.push_str(&outcome.output);
    }

    let mut notices = Vec::new();
    if outcome.cancelled {
        notices.push("[cancelled]".to_owned());
    } else if let Some(exit_code) = outcome.exit_code
        && exit_code != 0
    {
        notices.push(format!("[exit {exit_code}]"));
    }

    if outcome.truncated {
        notices.push(match outcome.full_output_path {
            Some(path) => format!("[output truncated; full output: {path}]"),
            None => "[output truncated]".to_owned(),
        });
    }

    for notice in notices {
        if !rendered.ends_with('\n') {
            rendered.push('\n');
        }
        rendered.push_str(&notice);
        rendered.push('\n');
    }

    rendered
}

fn main() {
    dioxus::LaunchBuilder::desktop()
        .with_cfg(
            dioxus::desktop::Config::new().with_window(
                dioxus::desktop::WindowBuilder::new()
                    .with_title("Spigot")
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
    let mut running_bash = use_signal(|| false);
    let mut input_element = use_signal(|| None::<std::rc::Rc<MountedData>>);

    let input_text = input();
    let is_running = running();
    let is_running_bash = running_bash();
    let is_bash_input = matches!(
        pi::UserRequest::from_input(input_text.clone()),
        pi::UserRequest::Bash { .. }
    );
    let status_label = if is_running_bash {
        "RUNNING"
    } else if is_running {
        "WORKING"
    } else {
        "READY"
    };
    let status_badge = if is_running_bash { "$" } else { "PI" };
    let mode_label = if is_running_bash {
        "(shell command)"
    } else {
        "(agent session)"
    };
    let output_text = output();
    let show_splash = !is_running && output_text.is_empty();
    let status_class = if is_running_bash {
        "modeline-status is-shell"
    } else if is_running {
        "modeline-status is-running"
    } else {
        "modeline-status"
    };
    let prompt_class = if is_bash_input {
        "prompt is-shell"
    } else {
        "prompt"
    };

    rsx! {
        style { {APP_STYLE} }
        main { class: "app",
            aside { class: "side-tabs", aria_label: "Buffers",
                div { class: "side-cell side-tab", "spigot.chat" }
            }
            section { class: "workspace", aria_label: "Spigot chat buffer",
                header { class: "modeline",
                    span { class: "{status_class}", "{status_badge}" }
                    span { class: "modeline-name", "Spigot" }
                    span { class: "modeline-mode", "{mode_label}" }
                    span { class: "modeline-secondary", "{status_label}" }
                }
                section { class: "buffer", aria_live: "polite", aria_busy: is_running,
                    if show_splash {
                        div { class: "splash",
                            span { class: "splash-title", "tasty" }
                        }
                    } else if is_running && output_text.is_empty() {
                        div { class: "working-line",
                            span { "Pi is working" }
                            span { class: "cursor", aria_hidden: "true" }
                        }
                    } else {
                        pre { class: "output", "{output_text}" }
                    }
                }
                form {
                    class: "{prompt_class}",
                    onsubmit: move |event| {
                        event.prevent_default();

                        if running() {
                            return;
                        }

                        let message = input().trim().to_owned();
                        if message.is_empty() {
                            return;
                        }

                        let request = pi::UserRequest::from_input(message);
                        let bash_command = match &request {
                            pi::UserRequest::Bash { command, .. } => Some(command.clone()),
                            pi::UserRequest::Prompt(_) => None,
                        };
                        let client = client.clone();
                        input.set(String::new());
                        output.set(
                            bash_command
                                .as_ref()
                                .map(|command| format!("$ {command}\n\n"))
                                .unwrap_or_default(),
                        );
                        running_bash.set(bash_command.is_some());
                        running.set(true);

                        spawn(async move {
                            let mut streamed_output = output;
                            let result = pi::run(&client, request, move |delta| {
                                streamed_output.write().push_str(delta);
                            })
                            .await;

                            match result {
                                Err(error) => output.set(format!("Error: {error}")),
                                Ok(pi::RequestOutcome::Prompt) if output.read().is_empty() => {
                                    output.set("(Pi returned no text output.)".to_owned());
                                }
                                Ok(pi::RequestOutcome::Bash(outcome)) => {
                                    if let Some(command) = bash_command {
                                        output.set(render_bash_output(&command, outcome));
                                    } else {
                                        output.set(
                                            "Error: the shell command was not available.".to_owned(),
                                        );
                                    }
                                }
                                Ok(pi::RequestOutcome::Prompt) => {}
                            }

                            running.set(false);
                            running_bash.set(false);
                            if let Some(input_element) = input_element.cloned() {
                                let _ = input_element.set_focus(true).await;
                            }
                        });
                    },
                    input {
                        aria_label: "Prompt or shell command",
                        autocomplete: "off",
                        autofocus: true,
                        placeholder: "Ask Pi or run !command…",
                        value: "{input_text}",
                        oninput: move |event| input.set(event.value()),
                        onmounted: move |element| input_element.set(Some(element.data())),
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{pi::BashOutcome, render_bash_output};

    #[test]
    fn renders_bash_exit_and_truncation_details() {
        let rendered = render_bash_output(
            "make test",
            BashOutcome {
                output: "failed".to_owned(),
                exit_code: Some(7),
                cancelled: false,
                truncated: true,
                full_output_path: Some("/tmp/full.log".to_owned()),
            },
        );

        assert_eq!(
            rendered,
            "$ make test\n\nfailed\n[exit 7]\n[output truncated; full output: /tmp/full.log]\n"
        );
    }
}
