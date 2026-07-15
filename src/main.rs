mod pi;

use dioxus::{desktop::LogicalSize, prelude::*};

static INTER_FONT: Asset = asset!("/assets/InterVariable.woff2");

const APP_STYLE: &str = r#"
    :root {
        color-scheme: light dark;
        --background: #ffffff;
        --surface: #f3f2f2;
        --surface-hover: #eaeaec;
        --text: #171717;
        --muted: #666666;
        --border: #d3d3d3;
        --accent: #ff5700;
    }

    @media (prefers-color-scheme: dark) {
        :root {
            --background: #1a1a1a;
            --surface: #252527;
            --surface-hover: #2b2b2b;
            --text: #f3f3f3;
            --muted: #acadad;
            --border: #444444;
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
        background: var(--background);
        color: var(--text);
        font-family: "InterVariable", system-ui, sans-serif;
        font-size: 15px;
        line-height: 1.5;
        -webkit-font-smoothing: antialiased;
    }

    input {
        font: inherit;
    }

    ::selection {
        background: var(--accent);
        color: #ffffff;
    }

    .app {
        display: grid;
        grid-template-rows: minmax(0, 1fr) auto;
        gap: 16px;
        width: 100%;
        max-width: 760px;
        height: 100%;
        margin: 0 auto;
        padding: 24px;
    }

    .buffer {
        min-width: 0;
        min-height: 0;
        padding: 8px 2px;
        overflow: auto;
        scrollbar-color: var(--border) transparent;
        scrollbar-width: thin;
    }

    .output {
        min-width: 0;
        margin: 0;
        overflow-wrap: anywhere;
        white-space: pre-wrap;
        color: var(--text);
        font: inherit;
    }

    .prompt {
        min-width: 0;
        overflow: hidden;
        border-radius: 12px;
        background: var(--surface);
    }

    .prompt:hover {
        background: var(--surface-hover);
    }

    .prompt input {
        width: 100%;
        min-width: 0;
        padding: 13px 15px;
        border: 0;
        outline: 0;
        background: transparent;
        color: var(--text);
        caret-color: var(--accent);
    }

    .prompt input::placeholder {
        color: var(--muted);
        opacity: 1;
    }

    .prompt input:focus::placeholder {
        opacity: 0;
    }

    @media (max-width: 600px) {
        .app {
            gap: 12px;
            padding: 16px;
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
    let mut input_element = use_signal(|| None::<std::rc::Rc<MountedData>>);

    let input_text = input();
    let is_running = running();
    let output_text = output();

    rsx! {
        style { "@font-face {{ font-family: 'InterVariable'; font-style: normal; font-weight: 100 900; font-display: swap; src: url('{INTER_FONT}') format('woff2'); }}" }
        style { {APP_STYLE} }
        main { class: "app", aria_label: "Spigot",
            section { class: "buffer", aria_live: "polite", aria_busy: is_running,
                pre { class: "output", "{output_text}" }
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
