mod pi;

use dioxus::prelude::*;

const APP_STYLE: &str = r#"
    * { box-sizing: border-box; }
    body {
        margin: 0;
        background: #f7f7f7;
        color: #171717;
        font-family: ui-sans-serif, system-ui, sans-serif;
    }
    .shell {
        display: grid;
        gap: 12px;
        width: min(760px, calc(100vw - 32px));
        margin: 32px auto;
    }
    .output {
        min-height: 240px;
        margin: 0;
        padding: 16px;
        overflow-wrap: anywhere;
        white-space: pre-wrap;
        background: white;
        border: 1px solid #d8d8d8;
        border-radius: 8px;
        font: inherit;
    }
    .prompt {
        display: flex;
        gap: 8px;
    }
    .prompt input {
        min-width: 0;
        flex: 1;
        padding: 10px 12px;
        border: 1px solid #b8b8b8;
        border-radius: 8px;
        font: inherit;
    }
    .prompt button {
        padding: 10px 16px;
        border: 0;
        border-radius: 8px;
        background: #171717;
        color: white;
        font: inherit;
        cursor: pointer;
    }
    .prompt button:disabled { cursor: default; opacity: 0.45; }
"#;

fn main() {
    dioxus::LaunchBuilder::desktop()
        .with_cfg(
            dioxus::desktop::Config::new()
                .with_window(dioxus::desktop::WindowBuilder::new().with_title("Crust")),
        )
        .launch(App);
}

#[component]
fn App() -> Element {
    let client = use_hook(pi::new_client);
    let mut input = use_signal(String::new);
    let mut output = use_signal(|| "Pi output will appear here.".to_owned());
    let mut running = use_signal(|| false);
    let mut input_element = use_signal(|| None::<std::rc::Rc<MountedData>>);

    let submit_disabled = running() || input.read().trim().is_empty();
    let submit_label = if running() { "Working…" } else { "Send" };

    rsx! {
        style { {APP_STYLE} }
        main { class: "shell",
            pre {
                class: "output",
                aria_live: "polite",
                "{output}"
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
