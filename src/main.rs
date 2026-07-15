mod pi;

use dioxus::{desktop::LogicalSize, prelude::*};
use serde_json::Value;
use std::rc::Rc;

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
        --error: #cf3535;
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
            --error: #ff5757;
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

    .transcript {
        display: flex;
        min-height: 100%;
        flex-direction: column;
        gap: 18px;
    }

    .user-message,
    .assistant-message,
    .tool-output {
        margin: 0;
        overflow-wrap: anywhere;
        white-space: pre-wrap;
        font: inherit;
    }

    .user-message {
        width: 100%;
        padding: 13px 15px;
        border-radius: 12px;
        background: var(--surface);
        color: var(--text);
    }

    .assistant-message {
        padding: 0 2px;
        color: var(--text);
    }

    .tool {
        width: 100%;
        padding: 10px 12px;
        border-radius: 10px;
        background: var(--surface);
        font-family: "Berkeley Mono", ui-monospace, monospace;
        font-size: 14px;
    }

    .tool-header {
        display: flex;
        align-items: center;
        gap: 8px;
        min-width: 0;
        font-weight: 600;
        overflow-wrap: anywhere;
    }

    .tool-dot {
        width: 7px;
        height: 7px;
        flex: 0 0 7px;
        border-radius: 50%;
        background: var(--accent);
    }

    .tool.is-error .tool-header,
    .tool-error,
    .error-message {
        color: var(--error);
    }

    .tool-error {
        margin-top: 5px;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .tool-output {
        margin-top: 8px;
        color: var(--muted);
    }

    .error-message {
        padding: 0 2px;
    }

    .transcript-end {
        height: 1px;
        flex: 0 0 1px;
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

#[derive(Clone, Debug, Eq, PartialEq)]
enum ToolState {
    Active,
    Complete,
    Failed,
}

#[derive(Clone, Debug, PartialEq)]
enum TranscriptItem {
    User(String),
    Assistant(String),
    Tool {
        id: String,
        summary: String,
        state: ToolState,
        error: Option<String>,
        output: String,
    },
    Error(String),
}

fn string_arg<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key).and_then(Value::as_str)
}

fn path_arg(args: &Value) -> &str {
    string_arg(args, "path")
        .or_else(|| string_arg(args, "file_path"))
        .filter(|path| !path.is_empty())
        .unwrap_or("...")
}

fn tool_summary(name: &str, args: &Value) -> String {
    match name {
        "read" => {
            let mut summary = format!("read {}", path_arg(args));
            let offset = args.get("offset").and_then(Value::as_u64);
            let limit = args.get("limit").and_then(Value::as_u64);
            if offset.is_some() || limit.is_some() {
                let start = offset.unwrap_or(1);
                summary.push_str(&format!(":{start}"));
                if let Some(limit) = limit {
                    summary.push_str(&format!(
                        "-{}",
                        start.saturating_add(limit).saturating_sub(1)
                    ));
                }
            }
            summary
        }
        "bash" => format!("$ {}", string_arg(args, "command").unwrap_or("...")),
        "edit" => format!("edit {}", path_arg(args)),
        "write" => format!("write {}", path_arg(args)),
        "grep" => format!(
            "grep /{}/ in {}",
            string_arg(args, "pattern").unwrap_or(""),
            string_arg(args, "path")
                .filter(|path| !path.is_empty())
                .unwrap_or("."),
        ),
        "find" => format!(
            "find {} in {}",
            string_arg(args, "pattern").unwrap_or("..."),
            string_arg(args, "path")
                .filter(|path| !path.is_empty())
                .unwrap_or("."),
        ),
        "ls" => format!(
            "ls {}",
            string_arg(args, "path")
                .filter(|path| !path.is_empty())
                .unwrap_or("."),
        ),
        _ => name.to_owned(),
    }
}

fn apply_stream_event(
    transcript: &mut Vec<TranscriptItem>,
    event: pi::StreamEvent,
    shell_id: Option<&str>,
) {
    match event {
        pi::StreamEvent::AssistantStart => {
            transcript.push(TranscriptItem::Assistant(String::new()));
        }
        pi::StreamEvent::TextDelta(delta) => {
            if let Some(TranscriptItem::Assistant(text)) = transcript.last_mut() {
                text.push_str(&delta);
            } else {
                transcript.push(TranscriptItem::Assistant(delta));
            }
        }
        pi::StreamEvent::ToolStart { id, name, args } => {
            transcript.push(TranscriptItem::Tool {
                id,
                summary: tool_summary(&name, &args),
                state: ToolState::Active,
                error: None,
                output: String::new(),
            });
        }
        pi::StreamEvent::ToolEnd {
            id,
            is_error,
            error,
        } => {
            if let Some(TranscriptItem::Tool {
                state,
                error: tool_error,
                ..
            }) = transcript.iter_mut().rev().find(
                |item| matches!(item, TranscriptItem::Tool { id: tool_id, .. } if tool_id == &id),
            ) {
                *state = if is_error {
                    ToolState::Failed
                } else {
                    ToolState::Complete
                };
                *tool_error = error;
            }
        }
        pi::StreamEvent::BashDelta(delta) => {
            if let Some(shell_id) = shell_id
                && let Some(TranscriptItem::Tool { output, .. }) = transcript
                    .iter_mut()
                    .rev()
                    .find(|item| matches!(item, TranscriptItem::Tool { id, .. } if id == shell_id))
            {
                output.push_str(&delta);
            }
        }
    }
}

fn fail_active_tools(transcript: &mut [TranscriptItem], error: &str) -> bool {
    let error = error
        .lines()
        .find(|line| !line.trim().is_empty())
        .map(str::trim)
        .unwrap_or("Request failed")
        .chars()
        .take(500)
        .collect::<String>();
    let mut marked = false;

    for item in transcript {
        if let TranscriptItem::Tool {
            state,
            error: tool_error,
            ..
        } = item
            && *state == ToolState::Active
        {
            *state = ToolState::Failed;
            *tool_error = Some(error.clone());
            marked = true;
        }
    }

    marked
}

fn push_shell(transcript: &mut Vec<TranscriptItem>, command: &str) -> String {
    let id = format!("shell-{}", transcript.len());
    transcript.push(TranscriptItem::Tool {
        id: id.clone(),
        summary: format!("$ {command}"),
        state: ToolState::Active,
        error: None,
        output: String::new(),
    });
    id
}

fn update_shell(
    transcript: &mut [TranscriptItem],
    id: &str,
    state: ToolState,
    error: Option<String>,
    output: Option<String>,
) {
    if let Some(TranscriptItem::Tool {
        state: tool_state,
        error: tool_error,
        output: tool_output,
        ..
    }) = transcript
        .iter_mut()
        .rev()
        .find(|item| matches!(item, TranscriptItem::Tool { id: tool_id, .. } if tool_id == id))
    {
        *tool_state = state;
        *tool_error = error;
        if let Some(output) = output {
            *tool_output = output;
        }
    }
}

fn render_bash_result(outcome: &pi::BashOutcome) -> String {
    let mut rendered = if outcome.output.is_empty() {
        "(no output)".to_owned()
    } else {
        outcome.output.clone()
    };

    let mut notices = Vec::new();
    if outcome.cancelled {
        notices.push("[cancelled]".to_owned());
    } else if let Some(exit_code) = outcome.exit_code
        && exit_code != 0
    {
        notices.push(format!("[exit {exit_code}]"));
    }

    if outcome.truncated {
        notices.push(match &outcome.full_output_path {
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

#[component]
fn TranscriptEntry(item: TranscriptItem) -> Element {
    match item {
        TranscriptItem::User(text) => rsx! {
            div { class: "user-message", "{text}" }
        },
        TranscriptItem::Assistant(text) if text.is_empty() => rsx! {},
        TranscriptItem::Assistant(text) => rsx! {
            pre { class: "assistant-message", "{text}" }
        },
        TranscriptItem::Tool {
            summary,
            state,
            error,
            output,
            ..
        } => {
            let class = if state == ToolState::Failed {
                "tool is-error"
            } else {
                "tool"
            };
            rsx! {
                div { class,
                    div { class: "tool-header",
                        if state == ToolState::Active {
                            span { class: "tool-dot", aria_hidden: "true" }
                        }
                        span { "{summary}" }
                    }
                    if let Some(error) = error {
                        div { class: "tool-error", "{error}" }
                    }
                    if !output.is_empty() {
                        pre { class: "tool-output", "{output}" }
                    }
                }
            }
        }
        TranscriptItem::Error(text) => rsx! {
            div { class: "error-message", "{text}" }
        },
    }
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
    let mut transcript = use_signal(Vec::<TranscriptItem>::new);
    let mut running = use_signal(|| false);
    let mut follow_output = use_signal(|| true);
    let mut input_element = use_signal(|| None::<Rc<MountedData>>);
    let mut transcript_end = use_signal(|| None::<Rc<MountedData>>);

    use_effect(move || {
        let _ = transcript.read().len();
        if *follow_output.peek()
            && let Some(end) = transcript_end.peek().clone()
        {
            spawn(async move {
                if *follow_output.peek() {
                    let _ = end.scroll_to(ScrollBehavior::Instant).await;
                }
            });
        }
    });

    let input_text = input();
    let is_running = running();
    let transcript_items = transcript();

    rsx! {
        style { "@font-face {{ font-family: 'InterVariable'; font-style: normal; font-weight: 100 900; font-display: swap; src: url('{INTER_FONT}') format('woff2'); }}" }
        style { {APP_STYLE} }
        main { class: "app", aria_label: "Spigot",
            section {
                class: "buffer",
                aria_live: "polite",
                aria_busy: is_running,
                onscroll: move |event| {
                    let data = event.data();
                    let distance = data.scroll_height() as f64
                        - data.client_height() as f64
                        - data.scroll_top();
                    follow_output.set(distance <= 48.0);
                },
                div { class: "transcript",
                    for (index, item) in transcript_items.into_iter().enumerate() {
                        TranscriptEntry { key: "{index}", item }
                    }
                    div {
                        class: "transcript-end",
                        onmounted: move |element| transcript_end.set(Some(element.data())),
                    }
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

                    let request = pi::UserRequest::from_input(message);
                    let shell_id = {
                        let mut items = transcript.write();
                        match &request {
                            pi::UserRequest::Prompt(message) => {
                                items.push(TranscriptItem::User(message.clone()));
                                None
                            }
                            pi::UserRequest::Bash { command, .. } => {
                                Some(push_shell(&mut items, command))
                            }
                        }
                    };
                    let client = client.clone();
                    input.set(String::new());
                    running.set(true);

                    spawn(async move {
                        let mut received_output = false;
                        let event_shell_id = shell_id.clone();
                        let mut streamed_transcript = transcript;
                        let result = pi::run(&client, request, |event| {
                            if matches!(
                                &event,
                                pi::StreamEvent::TextDelta(delta) if !delta.is_empty()
                            ) || matches!(event, pi::StreamEvent::ToolStart { .. })
                            {
                                received_output = true;
                            }
                            apply_stream_event(
                                &mut streamed_transcript.write(),
                                event,
                                event_shell_id.as_deref(),
                            );
                        })
                        .await;

                        match result {
                            Err(error) => {
                                let mut items = transcript.write();
                                if let Some(shell_id) = &shell_id {
                                    update_shell(
                                        &mut items,
                                        shell_id,
                                        ToolState::Failed,
                                        Some(error),
                                        None,
                                    );
                                } else if !fail_active_tools(&mut items, &error) {
                                    items.push(TranscriptItem::Error(format!("Error: {error}")));
                                }
                            }
                            Ok(pi::RequestOutcome::Prompt) if !received_output => {
                                transcript.write().push(TranscriptItem::Error(
                                    "Pi returned no text output.".to_owned(),
                                ));
                            }
                            Ok(pi::RequestOutcome::Bash(outcome)) => {
                                if let Some(shell_id) = &shell_id {
                                    let state = if outcome.cancelled
                                        || outcome.exit_code.is_some_and(|code| code != 0)
                                    {
                                        ToolState::Failed
                                    } else {
                                        ToolState::Complete
                                    };
                                    let output = render_bash_result(&outcome);
                                    update_shell(
                                        &mut transcript.write(),
                                        shell_id,
                                        state,
                                        None,
                                        Some(output),
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
    use super::{
        ToolState, TranscriptItem, apply_stream_event, fail_active_tools, push_shell,
        render_bash_result, tool_summary, update_shell,
    };
    use crate::pi::{BashOutcome, StreamEvent};
    use serde_json::json;

    #[test]
    fn summarizes_builtin_tool_calls() {
        assert_eq!(
            tool_summary(
                "read",
                &json!({ "path": "src/main.rs", "offset": 4, "limit": 3 })
            ),
            "read src/main.rs:4-6"
        );
        assert_eq!(
            tool_summary("bash", &json!({ "command": "cargo test" })),
            "$ cargo test"
        );
        assert_eq!(
            tool_summary("grep", &json!({ "pattern": "TODO", "path": "src" })),
            "grep /TODO/ in src"
        );
        assert_eq!(
            tool_summary("edit", &json!({ "path": "src/main.rs" })),
            "edit src/main.rs"
        );
        assert_eq!(
            tool_summary("write", &json!({ "path": "README.md" })),
            "write README.md"
        );
        assert_eq!(
            tool_summary("find", &json!({ "pattern": "*.rs", "path": "src" })),
            "find *.rs in src"
        );
        assert_eq!(tool_summary("ls", &json!({})), "ls .");
        assert_eq!(tool_summary("custom", &json!({})), "custom");
    }

    #[test]
    fn applies_ordered_assistant_and_parallel_tool_events() {
        let mut transcript = vec![TranscriptItem::User("Inspect it".to_owned())];
        apply_stream_event(&mut transcript, StreamEvent::AssistantStart, None);
        apply_stream_event(
            &mut transcript,
            StreamEvent::TextDelta("Looking now.".to_owned()),
            None,
        );
        apply_stream_event(
            &mut transcript,
            StreamEvent::ToolStart {
                id: "read-1".to_owned(),
                name: "read".to_owned(),
                args: json!({ "path": "src/main.rs" }),
            },
            None,
        );
        apply_stream_event(
            &mut transcript,
            StreamEvent::ToolStart {
                id: "grep-1".to_owned(),
                name: "grep".to_owned(),
                args: json!({ "pattern": "TODO" }),
            },
            None,
        );
        apply_stream_event(
            &mut transcript,
            StreamEvent::ToolEnd {
                id: "read-1".to_owned(),
                is_error: false,
                error: None,
            },
            None,
        );
        apply_stream_event(
            &mut transcript,
            StreamEvent::ToolEnd {
                id: "grep-1".to_owned(),
                is_error: true,
                error: Some("grep failed".to_owned()),
            },
            None,
        );
        apply_stream_event(&mut transcript, StreamEvent::AssistantStart, None);
        apply_stream_event(
            &mut transcript,
            StreamEvent::TextDelta("Finished.".to_owned()),
            None,
        );

        assert_eq!(transcript.len(), 5);
        assert_eq!(
            transcript[1],
            TranscriptItem::Assistant("Looking now.".to_owned())
        );
        assert!(matches!(
            &transcript[2],
            TranscriptItem::Tool {
                state: ToolState::Complete,
                ..
            }
        ));
        assert!(matches!(
            &transcript[3],
            TranscriptItem::Tool {
                state: ToolState::Failed,
                error: Some(error),
                ..
            } if error == "grep failed"
        ));
        assert_eq!(
            transcript[4],
            TranscriptItem::Assistant("Finished.".to_owned())
        );
    }

    #[test]
    fn request_failure_marks_active_tools_without_a_duplicate_error() {
        let mut transcript = vec![
            TranscriptItem::Tool {
                id: "active".to_owned(),
                summary: "read src/main.rs".to_owned(),
                state: ToolState::Active,
                error: None,
                output: String::new(),
            },
            TranscriptItem::Tool {
                id: "complete".to_owned(),
                summary: "ls .".to_owned(),
                state: ToolState::Complete,
                error: None,
                output: String::new(),
            },
        ];

        let marked = fail_active_tools(&mut transcript, "request timed out\nmore details");
        if !marked {
            transcript.push(TranscriptItem::Error("request timed out".to_owned()));
        }

        assert!(matches!(
            &transcript[0],
            TranscriptItem::Tool {
                state: ToolState::Failed,
                error: Some(error),
                ..
            } if error == "request timed out"
        ));
        assert!(matches!(
            &transcript[1],
            TranscriptItem::Tool {
                state: ToolState::Complete,
                error: None,
                ..
            }
        ));
        assert_eq!(transcript.len(), 2);
    }

    #[test]
    fn streams_and_completes_standalone_shell_output() {
        let mut transcript = Vec::new();
        let id = push_shell(&mut transcript, "printf hello");
        apply_stream_event(
            &mut transcript,
            StreamEvent::BashDelta("hello".to_owned()),
            Some(&id),
        );
        update_shell(
            &mut transcript,
            &id,
            ToolState::Complete,
            None,
            Some("hello".to_owned()),
        );

        assert!(matches!(
            &transcript[0],
            TranscriptItem::Tool {
                summary,
                state: ToolState::Complete,
                output,
                ..
            } if summary == "$ printf hello" && output == "hello"
        ));
    }

    #[test]
    fn renders_bash_exit_and_truncation_details() {
        let rendered = render_bash_result(&BashOutcome {
            output: "failed".to_owned(),
            exit_code: Some(7),
            cancelled: false,
            truncated: true,
            full_output_path: Some("/tmp/full.log".to_owned()),
        });

        assert_eq!(
            rendered,
            "failed\n[exit 7]\n[output truncated; full output: /tmp/full.log]\n"
        );
    }
}
