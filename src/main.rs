mod pi;

use dioxus::{
    desktop::{
        LogicalSize,
        tao::{dpi::LogicalUnit, window::WindowSizeConstraints},
    },
    prelude::*,
};
use serde_json::Value;
use std::rc::Rc;

static INTER_FONT: Asset = asset!("/assets/InterVariable.woff2");

#[cfg(target_os = "macos")]
const NEW_TAB_MENU_ID: &str = "spigot-new-tab";
#[cfg(target_os = "macos")]
const CLOSE_TAB_MENU_ID: &str = "spigot-close-tab";
#[cfg(target_os = "macos")]
const TABBING_IDENTIFIER: &str = "spigot";

#[cfg(target_os = "macos")]
std::thread_local! {
    static NATIVE_MENU: dioxus::desktop::muda::Menu = native_menu();
}

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

        /* Modus Operandi foregrounds. Syntax never owns the block background. */
        --syntax-text: #000000;
        --syntax-comment: #595959;
        --syntax-builtin: #8f0075;
        --syntax-constant: #0000b0;
        --syntax-docstring: #2a5045;
        --syntax-function: #721045;
        --syntax-function-call: #7b435c;
        --syntax-keyword: #531ab6;
        --syntax-preprocessor: #a0132f;
        --syntax-property: #005e8b;
        --syntax-regexp: #00663f;
        --syntax-string: #3548cf;
        --syntax-type: #005f5f;
        --syntax-variable: #005e8b;
        --syntax-added: #005000;
        --syntax-removed: #8f1313;
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

            /* Modus Vivendi foregrounds. */
            --syntax-text: #ffffff;
            --syntax-comment: #989898;
            --syntax-builtin: #f78fe7;
            --syntax-constant: #00bcff;
            --syntax-docstring: #9ac8e0;
            --syntax-function: #feacd0;
            --syntax-function-call: #d09dc0;
            --syntax-keyword: #b6a0ff;
            --syntax-preprocessor: #ff7f86;
            --syntax-property: #00d3d0;
            --syntax-regexp: #00c06f;
            --syntax-string: #79a8ff;
            --syntax-type: #6ae4b9;
            --syntax-variable: #00d3d0;
            --syntax-added: #a0e0a0;
            --syntax-removed: #ffbfbf;
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
        background: rgb(255 87 0 / 16%);
        color: var(--accent);
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
        padding: 24px 2px;
        overflow: auto;
        scrollbar-color: var(--border) transparent;
        scrollbar-width: thin;
        -webkit-mask-image: linear-gradient(to bottom, transparent, black 24px, black calc(100% - 24px), transparent);
        mask-image: linear-gradient(to bottom, transparent, black 24px, black calc(100% - 24px), transparent);
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

    .thinking-message {
        font-style: italic;
    }

    .tool,
    .tool-group {
        width: 100%;
        color: var(--muted);
        font-family: "Berkeley Mono", ui-monospace, monospace;
        font-size: 13px;
    }

    .tool-group {
        display: grid;
        gap: 3px;
    }

    .tool.direct-shell {
        padding: 10px 12px;
        border-radius: 10px;
        background: var(--surface);
        color: var(--text);
        font-size: 14px;
    }

    .tool-group-title {
        color: var(--text);
        font-weight: 600;
    }

    .tool-row {
        min-width: 0;
        padding: 1px 0;
    }

    button.tool-row {
        width: 100%;
        border: 0;
        background: transparent;
        color: inherit;
        font: inherit;
        text-align: left;
        cursor: pointer;
    }

    button.tool-row:hover {
        color: var(--text);
    }

    .tool-header {
        display: flex;
        align-items: center;
        gap: 7px;
        min-width: 0;
        overflow-wrap: anywhere;
    }

    .tool-dot {
        width: 6px;
        height: 6px;
        flex: 0 0 6px;
        border-radius: 50%;
        background: var(--accent);
    }

    .shell-prompt {
        color: var(--accent);
    }

    .tool.is-error,
    .tool-row.is-error,
    .tool-error,
    .error-message {
        color: var(--error);
    }

    .tool-error {
        display: block;
        margin-left: 13px;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .change-count {
        margin-left: auto;
        white-space: nowrap;
    }

    .added {
        color: var(--syntax-added);
    }

    .removed {
        color: var(--syntax-removed);
    }

    .tool-output {
        max-height: 15em;
        margin: 8px 0 0;
        overflow-y: auto;
        color: var(--muted);
        scrollbar-color: var(--border) transparent;
        scrollbar-width: thin;
        white-space: pre-wrap;
    }

    .detail-page {
        width: 100%;
        height: 100%;
        padding: 24px;
        overflow: auto;
        background: var(--background);
        scrollbar-color: var(--border) transparent;
        scrollbar-width: thin;
    }

    .detail-title,
    .detail-output {
        width: min(100%, 1100px);
        margin-right: auto;
        margin-left: auto;
    }

    .detail-title {
        margin-top: 0;
        margin-bottom: 16px;
        color: var(--muted);
        font-size: 13px;
        font-weight: 600;
    }

    .detail-output {
        margin-top: 0;
        margin-bottom: 0;
        overflow-wrap: anywhere;
        white-space: pre-wrap;
        font-family: "Berkeley Mono", ui-monospace, monospace;
        font-size: 13px;
        line-height: 1.55;
    }

    .highlighted-output,
    .highlighted-output .hljs-subst,
    .highlighted-output .hljs-number,
    .highlighted-output .hljs-operator,
    .highlighted-output .hljs-punctuation {
        color: var(--syntax-text);
    }

    .highlighted-output .hljs-comment {
        color: var(--syntax-comment);
        font-style: italic;
    }

    .highlighted-output .hljs-doctag,
    .highlighted-output .hljs-quote {
        color: var(--syntax-docstring);
        font-style: italic;
    }

    .highlighted-output .hljs-keyword,
    .highlighted-output .hljs-name,
    .highlighted-output .hljs-selector-tag {
        color: var(--syntax-keyword);
        font-weight: 700;
    }

    .highlighted-output .hljs-built_in {
        color: var(--syntax-builtin);
        font-weight: 700;
    }

    .highlighted-output .hljs-literal,
    .highlighted-output .hljs-symbol,
    .highlighted-output .hljs-template-tag,
    .highlighted-output .hljs-bullet {
        color: var(--syntax-constant);
    }

    .highlighted-output .hljs-string,
    .highlighted-output .hljs-meta-string,
    .highlighted-output .hljs-link {
        color: var(--syntax-string);
    }

    .highlighted-output .hljs-regexp {
        color: var(--syntax-regexp);
    }

    .highlighted-output .hljs-title,
    .highlighted-output .hljs-section,
    .highlighted-output .hljs-selector-id {
        color: var(--syntax-function);
    }

    .highlighted-output .hljs-function {
        color: var(--syntax-function-call);
    }

    .highlighted-output .hljs-type,
    .highlighted-output .hljs-class .hljs-title,
    .highlighted-output .hljs-selector-class,
    .highlighted-output .hljs-code {
        color: var(--syntax-type);
        font-weight: 700;
    }

    .highlighted-output .hljs-attr,
    .highlighted-output .hljs-attribute,
    .highlighted-output .hljs-selector-attr,
    .highlighted-output .hljs-selector-pseudo {
        color: var(--syntax-property);
    }

    .highlighted-output .hljs-variable,
    .highlighted-output .hljs-template-variable,
    .highlighted-output .hljs-params {
        color: var(--syntax-variable);
    }

    .highlighted-output .hljs-meta,
    .highlighted-output .hljs-meta-keyword {
        color: var(--syntax-preprocessor);
    }

    .highlighted-output .hljs-addition {
        color: var(--syntax-added);
    }

    .highlighted-output .hljs-deletion {
        color: var(--syntax-removed);
    }

    .highlighted-output .hljs-emphasis {
        font-style: italic;
    }

    .highlighted-output .hljs-strong {
        font-weight: 700;
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

#[derive(Clone, Debug, Eq, PartialEq)]
enum ToolKind {
    Exploration,
    Bash,
    Edit,
    Write,
    Other,
}

#[derive(Clone, Debug, PartialEq)]
enum ToolDetailContent {
    Plain(String),
    HighlightedHtml(String),
}

#[derive(Clone, Debug, PartialEq)]
struct ToolDetail {
    title: String,
    content: ToolDetailContent,
}

#[derive(Clone, Debug, PartialEq)]
struct ToolActivity {
    id: String,
    kind: ToolKind,
    summary: String,
    state: ToolState,
    error: Option<String>,
    detail: Option<ToolDetail>,
    added: Option<u64>,
    removed: Option<u64>,
    direct_shell: bool,
    output: String,
    highlighted_html: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
enum TranscriptItem {
    User(String),
    Assistant(String),
    Thinking { id: String, text: String },
    Exploration(Vec<ToolActivity>),
    Tool(ToolActivity),
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
            let mut summary = format!("Read {}", path_arg(args));
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
        "bash" => string_arg(args, "command").unwrap_or("...").to_owned(),
        "edit" | "write" => path_arg(args).to_owned(),
        "grep" => format!(
            "Search /{}/ in {}",
            string_arg(args, "pattern").unwrap_or(""),
            string_arg(args, "path")
                .filter(|path| !path.is_empty())
                .unwrap_or("."),
        ),
        "find" => format!(
            "Find {} in {}",
            string_arg(args, "pattern").unwrap_or("..."),
            string_arg(args, "path")
                .filter(|path| !path.is_empty())
                .unwrap_or("."),
        ),
        "ls" => format!(
            "List {}",
            string_arg(args, "path")
                .filter(|path| !path.is_empty())
                .unwrap_or("."),
        ),
        _ => name.to_owned(),
    }
}

fn tool_kind(name: &str) -> ToolKind {
    match name {
        "read" | "grep" | "find" | "ls" => ToolKind::Exploration,
        "bash" => ToolKind::Bash,
        "edit" => ToolKind::Edit,
        "write" => ToolKind::Write,
        _ => ToolKind::Other,
    }
}

fn tool_label(tool: &ToolActivity) -> String {
    if tool.direct_shell {
        return format!("$ {}", tool.summary);
    }

    let verb = match (&tool.kind, &tool.state) {
        (ToolKind::Bash | ToolKind::Other, ToolState::Active) => "Running",
        (ToolKind::Bash | ToolKind::Other, _) => "Ran",
        (ToolKind::Edit, ToolState::Active) => "Editing",
        (ToolKind::Edit, ToolState::Failed) => "Failed to edit",
        (ToolKind::Edit, ToolState::Complete) => "Edited",
        (ToolKind::Write, ToolState::Active) => "Writing",
        (ToolKind::Write, ToolState::Failed) => "Failed to write",
        (ToolKind::Write, ToolState::Complete) => "Wrote",
        (ToolKind::Exploration, _) => return tool.summary.clone(),
    };
    format!("{verb} {}", tool.summary)
}

fn detail_from(
    title: String,
    output: Option<String>,
    highlighted_html: Option<String>,
) -> Option<ToolDetail> {
    let content = if let Some(html) = highlighted_html.filter(|html| !html.is_empty()) {
        ToolDetailContent::HighlightedHtml(html)
    } else {
        ToolDetailContent::Plain(output.filter(|output| !output.is_empty())?)
    };
    Some(ToolDetail { title, content })
}

fn find_tool_mut<'a>(
    transcript: &'a mut [TranscriptItem],
    id: &str,
) -> Option<&'a mut ToolActivity> {
    transcript.iter_mut().rev().find_map(|item| match item {
        TranscriptItem::Tool(tool) if tool.id == id => Some(tool),
        TranscriptItem::Exploration(tools) => tools.iter_mut().rev().find(|tool| tool.id == id),
        _ => None,
    })
}

fn remove_trailing_empty_assistant(transcript: &mut Vec<TranscriptItem>) {
    if transcript
        .last()
        .is_some_and(|item| matches!(item, TranscriptItem::Assistant(text) if text.is_empty()))
    {
        transcript.pop();
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
        pi::StreamEvent::ThinkingStart { id } => {
            remove_trailing_empty_assistant(transcript);
            transcript.push(TranscriptItem::Thinking {
                id,
                text: String::new(),
            });
        }
        pi::StreamEvent::ThinkingDelta { id, delta } => {
            if let Some(TranscriptItem::Thinking { text, .. }) = transcript.iter_mut().rev().find(
                |item| matches!(item, TranscriptItem::Thinking { id: thinking_id, .. } if thinking_id == &id),
            ) {
                text.push_str(&delta);
            } else {
                transcript.push(TranscriptItem::Thinking { id, text: delta });
            }
        }
        pi::StreamEvent::ThinkingEnd { id, content } => {
            if content.is_empty() {
                if let Some(index) = transcript.iter().rposition(
                    |item| matches!(item, TranscriptItem::Thinking { id: thinking_id, text } if thinking_id == &id && text.is_empty()),
                ) {
                    transcript.remove(index);
                }
            } else if let Some(TranscriptItem::Thinking { text, .. }) = transcript
                .iter_mut()
                .rev()
                .find(|item| matches!(item, TranscriptItem::Thinking { id: thinking_id, .. } if thinking_id == &id))
            {
                *text = content;
            } else {
                transcript.push(TranscriptItem::Thinking { id, text: content });
            }
        }
        pi::StreamEvent::TextDelta(delta) => {
            if let Some(TranscriptItem::Assistant(text)) = transcript.last_mut() {
                text.push_str(&delta);
            } else {
                transcript.push(TranscriptItem::Assistant(delta));
            }
        }
        pi::StreamEvent::ToolStart { id, name, args } => {
            remove_trailing_empty_assistant(transcript);
            let kind = tool_kind(&name);
            let tool = ToolActivity {
                id,
                kind: kind.clone(),
                summary: tool_summary(&name, &args),
                state: ToolState::Active,
                error: None,
                detail: None,
                added: None,
                removed: None,
                direct_shell: false,
                output: String::new(),
                highlighted_html: None,
            };
            if kind == ToolKind::Exploration {
                if let Some(TranscriptItem::Exploration(tools)) = transcript.last_mut() {
                    tools.push(tool);
                } else {
                    transcript.push(TranscriptItem::Exploration(vec![tool]));
                }
            } else {
                transcript.push(TranscriptItem::Tool(tool));
            }
        }
        pi::StreamEvent::ToolEnd {
            id,
            is_error,
            error,
            output,
            highlighted_html,
            added,
            removed,
        } => {
            if let Some(tool) = find_tool_mut(transcript, &id) {
                tool.state = if is_error {
                    ToolState::Failed
                } else {
                    ToolState::Complete
                };
                tool.error = error;
                tool.detail = detail_from(tool_label(tool), output, highlighted_html);
                tool.added = added;
                tool.removed = removed;
            }
        }
        pi::StreamEvent::BashDelta(delta) => {
            if let Some(shell_id) = shell_id
                && let Some(tool) = find_tool_mut(transcript, shell_id)
            {
                tool.output.push_str(&delta);
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
        let tools: &mut [ToolActivity] = match item {
            TranscriptItem::Tool(tool) => std::slice::from_mut(tool),
            TranscriptItem::Exploration(tools) => tools,
            _ => continue,
        };
        for tool in tools {
            if tool.state == ToolState::Active {
                tool.state = ToolState::Failed;
                tool.error = Some(error.clone());
                marked = true;
            }
        }
    }

    marked
}

fn push_shell(transcript: &mut Vec<TranscriptItem>, command: &str) -> String {
    let id = format!("shell-{}", transcript.len());
    transcript.push(TranscriptItem::Tool(ToolActivity {
        id: id.clone(),
        kind: ToolKind::Bash,
        summary: command.to_owned(),
        state: ToolState::Active,
        error: None,
        detail: None,
        added: None,
        removed: None,
        direct_shell: true,
        output: String::new(),
        highlighted_html: None,
    }));
    id
}

fn update_shell(
    transcript: &mut [TranscriptItem],
    id: &str,
    state: ToolState,
    error: Option<String>,
    output: Option<String>,
    highlighted_html: Option<String>,
) {
    if let Some(tool) = find_tool_mut(transcript, id) {
        tool.state = state;
        tool.error = error;
        if let Some(output) = output {
            tool.output = output;
        }
        tool.highlighted_html = highlighted_html;
    }
}

fn escape_html(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            _ => escaped.push(character),
        }
    }
    escaped
}

fn render_bash_result(outcome: &pi::BashOutcome) -> (String, Option<String>) {
    let mut rendered = if outcome.output.is_empty() {
        "(no output)".to_owned()
    } else {
        outcome.output.clone()
    };
    let mut highlighted_html = if outcome.output.is_empty() {
        None
    } else {
        outcome.highlighted_html.clone()
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
        let needs_newline = !rendered.ends_with('\n');
        if needs_newline {
            rendered.push('\n');
        }
        rendered.push_str(&notice);
        rendered.push('\n');

        if let Some(html) = &mut highlighted_html {
            if needs_newline {
                html.push('\n');
            }
            html.push_str(&escape_html(&notice));
            html.push('\n');
        }
    }

    (rendered, highlighted_html)
}

fn plain_thinking(text: &str) -> &str {
    let text = text.trim();
    let text = text.strip_prefix("**").unwrap_or(text);
    let text = text.strip_suffix("**").unwrap_or(text);
    text.trim()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DetailOpenMode {
    FocusedTab,
    BackgroundTab,
    Window,
}

fn detail_open_mode(command: bool, shift: bool) -> DetailOpenMode {
    match (command, shift) {
        (true, true) => DetailOpenMode::Window,
        (true, false) => DetailOpenMode::BackgroundTab,
        _ => DetailOpenMode::FocusedTab,
    }
}

#[component]
fn ToolDetailView(detail: ToolDetail) -> Element {
    #[cfg(target_os = "macos")]
    use_native_tabs();

    rsx! {
        style { "@font-face {{ font-family: 'InterVariable'; font-style: normal; font-weight: 100 900; font-display: swap; src: url('{INTER_FONT}') format('woff2'); }}" }
        style { {APP_STYLE} }
        main { class: "detail-page", aria_label: "Tool detail",
            h1 { class: "detail-title", "{detail.title}" }
            match detail.content {
                ToolDetailContent::HighlightedHtml(html) => rsx! {
                    pre {
                        class: "detail-output highlighted-output",
                        // This HTML is generated locally by highlight.js, which escapes source text.
                        dangerous_inner_html: html,
                    }
                },
                ToolDetailContent::Plain(text) => rsx! {
                    pre { class: "detail-output", "{text}" }
                },
            }
        }
    }
}

fn open_tool_detail(
    source: dioxus::desktop::DesktopContext,
    detail: ToolDetail,
    mode: DetailOpenMode,
) {
    spawn(async move {
        let title = detail.title.clone();
        let tab = source
            .new_window(
                VirtualDom::new_with_props(ToolDetailView, ToolDetailViewProps { detail }),
                window_config(&title, mode != DetailOpenMode::BackgroundTab),
            )
            .await;

        #[cfg(target_os = "macos")]
        match mode {
            DetailOpenMode::FocusedTab => {
                group_window_as_tab(&source, &tab, true);
                tab.set_focus();
            }
            DetailOpenMode::BackgroundTab => {
                group_window_as_tab(&source, &tab, false);
            }
            DetailOpenMode::Window => tab.set_focus(),
        }

        #[cfg(not(target_os = "macos"))]
        if mode != DetailOpenMode::BackgroundTab {
            tab.set_focus();
        }
    });
}

#[component]
fn ToolRow(tool: ToolActivity) -> Element {
    let source = dioxus::desktop::use_window();
    let label = tool_label(&tool);
    let class = if tool.state == ToolState::Failed {
        "tool-row is-error"
    } else {
        "tool-row"
    };
    let detail = tool.detail.clone();
    let added = tool.added;
    let removed = tool.removed;

    let content = rsx! {
        span { class: "tool-header",
            if tool.state == ToolState::Active {
                span { class: "tool-dot", aria_hidden: "true" }
            }
            if tool.direct_shell {
                span {
                    span { class: "shell-prompt", "$" }
                    " {tool.summary}"
                }
            } else {
                span { "{label}" }
            }
            if added.is_some() || removed.is_some() {
                span { class: "change-count",
                    span { class: "added", "+{added.unwrap_or(0)}" }
                    " "
                    span { class: "removed", "-{removed.unwrap_or(0)}" }
                }
            }
        }
        if let Some(error) = &tool.error {
            span { class: "tool-error", "{error}" }
        }
        if tool.direct_shell {
            if let Some(highlighted_html) = &tool.highlighted_html {
                pre {
                    class: "tool-output highlighted-output",
                    // This HTML is generated locally by highlight.js, which escapes source text.
                    dangerous_inner_html: highlighted_html,
                }
            } else if !tool.output.is_empty() {
                pre { class: "tool-output", "{tool.output}" }
            }
        }
    };

    if let Some(detail) = detail {
        rsx! {
            button {
                class,
                type: "button",
                title: "Open detail",
                onclick: move |event| {
                    let modifiers = event.modifiers();
                    let mode = detail_open_mode(
                        modifiers.contains(Modifiers::META),
                        modifiers.contains(Modifiers::SHIFT),
                    );
                    open_tool_detail(source.clone(), detail.clone(), mode);
                },
                {content}
            }
        }
    } else {
        rsx! { div { class, {content} } }
    }
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
        TranscriptItem::Thinking { text, .. } => {
            let text = plain_thinking(&text);
            rsx! {
                pre { class: "assistant-message thinking-message", "{text}" }
            }
        }
        TranscriptItem::Exploration(tools) => {
            let title = if tools.iter().any(|tool| tool.state == ToolState::Active) {
                "Exploring"
            } else {
                "Explored"
            };
            rsx! {
                div { class: "tool-group",
                    div { class: "tool-group-title", "{title}" }
                    for tool in tools {
                        ToolRow { key: "{tool.id}", tool }
                    }
                }
            }
        }
        TranscriptItem::Tool(tool) => {
            let class = if tool.direct_shell {
                "tool direct-shell"
            } else {
                "tool"
            };
            rsx! {
                div { class, ToolRow { tool } }
            }
        }
        TranscriptItem::Error(text) => rsx! {
            div { class: "error-message", "{text}" }
        },
    }
}

fn window_builder(title: &str, focused: bool) -> dioxus::desktop::WindowBuilder {
    let builder = dioxus::desktop::WindowBuilder::new()
        .with_title(title)
        .with_focused(focused)
        .with_inner_size(LogicalSize::new(760.0, 760.0))
        .with_inner_size_constraints(WindowSizeConstraints::new(
            Some(LogicalUnit::new(420.0).into()),
            Some(LogicalUnit::new(360.0).into()),
            Some(LogicalUnit::new(760.0).into()),
            None,
        ));

    #[cfg(target_os = "macos")]
    let builder = {
        use dioxus::desktop::tao::platform::macos::WindowBuilderExtMacOS;

        builder
            .with_title_hidden(true)
            .with_automatic_window_tabbing(false)
            .with_tabbing_identifier(TABBING_IDENTIFIER)
    };

    builder
}

#[cfg(target_os = "macos")]
fn native_menu() -> dioxus::desktop::muda::Menu {
    use dioxus::desktop::muda::{
        Menu, MenuItem, PredefinedMenuItem, Submenu,
        accelerator::{Accelerator, CMD_OR_CTRL, Code},
    };

    let menu = Menu::new();

    let app_menu = Submenu::new("Spigot", true);
    app_menu
        .append_items(&[
            &PredefinedMenuItem::about(None, None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::services(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::hide(None),
            &PredefinedMenuItem::hide_others(None),
            &PredefinedMenuItem::show_all(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::quit(None),
        ])
        .expect("the native application menu should be valid");

    let file_menu = Submenu::new("File", true);
    file_menu
        .append_items(&[
            &MenuItem::with_id(
                NEW_TAB_MENU_ID,
                "New Tab",
                true,
                Some(Accelerator::new(Some(CMD_OR_CTRL), Code::KeyT)),
            ),
            &PredefinedMenuItem::separator(),
            &MenuItem::with_id(
                CLOSE_TAB_MENU_ID,
                "Close Tab",
                true,
                Some(Accelerator::new(Some(CMD_OR_CTRL), Code::KeyW)),
            ),
        ])
        .expect("the native File menu should be valid");

    let edit_menu = Submenu::new("Edit", true);
    edit_menu
        .append_items(&[
            &PredefinedMenuItem::undo(None),
            &PredefinedMenuItem::redo(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::cut(None),
            &PredefinedMenuItem::copy(None),
            &PredefinedMenuItem::paste(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::select_all(None),
        ])
        .expect("the native Edit menu should be valid");

    let window_menu = Submenu::new("Window", true);
    window_menu
        .append_items(&[
            &PredefinedMenuItem::minimize(None),
            &PredefinedMenuItem::maximize(None),
            &PredefinedMenuItem::fullscreen(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::bring_all_to_front(None),
        ])
        .expect("the native Window menu should be valid");

    menu.append_items(&[&app_menu, &file_menu, &edit_menu, &window_menu])
        .expect("the native menu bar should be valid");
    menu.init_for_nsapp();
    window_menu.set_as_windows_menu_for_nsapp();

    if cfg!(debug_assertions) {
        let help_menu = Submenu::new("Help", true);
        help_menu
            .append_items(&[
                &MenuItem::with_id(
                    "dioxus-toggle-dev-tools",
                    "Toggle Developer Tools",
                    true,
                    None,
                ),
                &MenuItem::with_id(
                    "dioxus-float-top",
                    "Float on Top (dev mode only)",
                    true,
                    None,
                ),
            ])
            .expect("the native Help menu should be valid");
        menu.append(&help_menu)
            .expect("the native Help menu should attach");
        help_menu.set_as_help_menu_for_nsapp();
    }

    menu
}

#[cfg(target_os = "macos")]
fn install_native_menu() {
    NATIVE_MENU.with(|_| {});
}

fn window_config(title: &str, focused: bool) -> dioxus::desktop::Config {
    let config = dioxus::desktop::Config::new().with_window(window_builder(title, focused));

    #[cfg(target_os = "macos")]
    let config = config.with_menu(None);

    config
}

#[cfg(target_os = "macos")]
fn hide_titlebar_separator(window: &dioxus::desktop::DesktopContext) {
    use dioxus::desktop::tao::platform::macos::WindowExtMacOS;
    use objc2_app_kit::{NSTitlebarSeparatorStyle, NSWindow};

    unsafe {
        // The context keeps the Tao window alive, and Dioxus runs components on the main thread.
        let window = &*window.window.ns_window().cast::<NSWindow>();
        window.setTitlebarSeparatorStyle(NSTitlebarSeparatorStyle::None);
    }
}

#[cfg(target_os = "macos")]
fn group_window_as_tab(
    source: &dioxus::desktop::DesktopContext,
    tab: &dioxus::desktop::DesktopContext,
    select_tab: bool,
) {
    use dioxus::desktop::tao::platform::macos::WindowExtMacOS;
    use objc2_app_kit::{NSWindow, NSWindowOrderingMode};

    unsafe {
        // Both contexts keep their Tao windows alive, and Dioxus polls this task on the main thread.
        let source_window = &*source.window.ns_window().cast::<NSWindow>();
        let tab_window = &*tab.window.ns_window().cast::<NSWindow>();
        source_window.addTabbedWindow_ordered(tab_window, NSWindowOrderingMode::Above);
        if let Some(group) = source_window.tabGroup() {
            group.setSelectedWindow(Some(if select_tab {
                tab_window
            } else {
                source_window
            }));
        }
    }

    hide_titlebar_separator(source);
    hide_titlebar_separator(tab);
}

#[cfg(target_os = "macos")]
fn use_native_tabs() {
    let source = dioxus::desktop::use_window();
    hide_titlebar_separator(&source);
    let source = Rc::downgrade(&source);

    dioxus::desktop::use_muda_event_handler(move |event| {
        let Some(source) = source.upgrade() else {
            return;
        };
        if !source.is_focused() {
            return;
        }

        match event.id().0.as_str() {
            NEW_TAB_MENU_ID => {
                let source = source.clone();
                spawn(async move {
                    let tab = source
                        .new_window(VirtualDom::new(App), window_config("Spigot", true))
                        .await;
                    group_window_as_tab(&source, &tab, true);
                    tab.set_focus();
                });
            }
            CLOSE_TAB_MENU_ID => source.close(),
            _ => {}
        }
    });
}

fn main() {
    #[cfg(target_os = "macos")]
    install_native_menu();

    dioxus::LaunchBuilder::desktop()
        .with_cfg(window_config("Spigot", true))
        .launch(App);
}

#[component]
fn App() -> Element {
    #[cfg(target_os = "macos")]
    use_native_tabs();

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
                                pi::StreamEvent::TextDelta(delta)
                                    | pi::StreamEvent::ThinkingDelta { delta, .. }
                                    | pi::StreamEvent::ThinkingEnd {
                                        content: delta, ..
                                    } if !delta.is_empty()
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
                                    let (output, highlighted_html) = render_bash_result(&outcome);
                                    update_shell(
                                        &mut transcript.write(),
                                        shell_id,
                                        state,
                                        None,
                                        Some(output),
                                        highlighted_html,
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
        APP_STYLE, DetailOpenMode, ToolDetailContent, ToolKind, ToolState, TranscriptItem,
        apply_stream_event, detail_open_mode, fail_active_tools, plain_thinking, push_shell,
        render_bash_result, tool_label, tool_summary, update_shell,
    };
    use crate::pi::{BashOutcome, StreamEvent};
    use serde_json::json;

    fn end(id: &str, output: Option<&str>, html: Option<&str>) -> StreamEvent {
        StreamEvent::ToolEnd {
            id: id.to_owned(),
            is_error: false,
            error: None,
            output: output.map(str::to_owned),
            highlighted_html: html.map(str::to_owned),
            added: None,
            removed: None,
        }
    }

    #[test]
    fn syntax_highlighting_does_not_set_a_background() {
        let start = APP_STYLE
            .find("    .highlighted-output")
            .expect("syntax rules should exist");
        let end = APP_STYLE[start..]
            .find("    .error-message")
            .map(|end| start + end)
            .expect("syntax rules should end before error styles");

        assert!(!APP_STYLE[start..end].contains("background"));
    }

    #[test]
    fn summarizes_builtin_tool_calls() {
        assert_eq!(
            tool_summary(
                "read",
                &json!({ "path": "src/main.rs", "offset": 4, "limit": 3 })
            ),
            "Read src/main.rs:4-6"
        );
        assert_eq!(
            tool_summary("bash", &json!({ "command": "cargo test" })),
            "cargo test"
        );
        assert_eq!(
            tool_summary("grep", &json!({ "pattern": "TODO", "path": "src" })),
            "Search /TODO/ in src"
        );
        assert_eq!(
            tool_summary("edit", &json!({ "path": "src/main.rs" })),
            "src/main.rs"
        );
        assert_eq!(
            tool_summary("write", &json!({ "path": "README.md" })),
            "README.md"
        );
        assert_eq!(
            tool_summary("find", &json!({ "pattern": "*.rs", "path": "src" })),
            "Find *.rs in src"
        );
        assert_eq!(tool_summary("ls", &json!({})), "List .");
        assert_eq!(tool_summary("custom", &json!({})), "custom");
    }

    #[test]
    fn groups_exploration_and_matches_parallel_completions_by_id() {
        let mut transcript = vec![TranscriptItem::User("Inspect it".to_owned())];
        apply_stream_event(&mut transcript, StreamEvent::AssistantStart, None);
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
            end("grep-1", Some("src/main.rs:1:TODO"), None),
            None,
        );
        apply_stream_event(
            &mut transcript,
            end("read-1", None, Some("<span>fn</span>")),
            None,
        );

        assert_eq!(transcript.len(), 2);
        let TranscriptItem::Exploration(tools) = &transcript[1] else {
            panic!("expected one exploration group")
        };
        assert_eq!(tools.len(), 2);
        assert_eq!(tools[0].id, "read-1");
        assert_eq!(tools[1].id, "grep-1");
        assert_eq!(tools[0].state, ToolState::Complete);
        assert!(matches!(
            tools[0].detail.as_ref().map(|detail| &detail.content),
            Some(ToolDetailContent::HighlightedHtml(html)) if html == "<span>fn</span>"
        ));
        assert!(matches!(
            tools[1].detail.as_ref().map(|detail| &detail.content),
            Some(ToolDetailContent::Plain(output)) if output == "src/main.rs:1:TODO"
        ));
    }

    #[test]
    fn exploration_groups_stop_at_text_and_non_exploration_tools() {
        let mut transcript = Vec::new();
        for (id, name) in [("read-1", "read"), ("grep-1", "grep")] {
            apply_stream_event(
                &mut transcript,
                StreamEvent::ToolStart {
                    id: id.to_owned(),
                    name: name.to_owned(),
                    args: json!({}),
                },
                None,
            );
        }
        apply_stream_event(
            &mut transcript,
            StreamEvent::TextDelta("Checked those.".to_owned()),
            None,
        );
        apply_stream_event(
            &mut transcript,
            StreamEvent::ToolStart {
                id: "find".to_owned(),
                name: "find".to_owned(),
                args: json!({}),
            },
            None,
        );
        apply_stream_event(
            &mut transcript,
            StreamEvent::ToolStart {
                id: "bash".to_owned(),
                name: "bash".to_owned(),
                args: json!({ "command": "pwd" }),
            },
            None,
        );
        apply_stream_event(
            &mut transcript,
            StreamEvent::ToolStart {
                id: "ls".to_owned(),
                name: "ls".to_owned(),
                args: json!({}),
            },
            None,
        );

        assert!(matches!(&transcript[0], TranscriptItem::Exploration(tools) if tools.len() == 2));
        assert_eq!(
            transcript[1],
            TranscriptItem::Assistant("Checked those.".to_owned())
        );
        assert!(matches!(&transcript[2], TranscriptItem::Exploration(tools) if tools.len() == 1));
        assert!(
            matches!(&transcript[3], TranscriptItem::Tool(tool) if tool.kind == ToolKind::Bash)
        );
        assert!(matches!(&transcript[4], TranscriptItem::Exploration(tools) if tools.len() == 1));
    }

    #[test]
    fn uses_compact_semantic_state_labels() {
        let mut transcript = Vec::new();
        for (id, name, args) in [
            ("bash", "bash", json!({ "command": "cargo test" })),
            ("edit", "edit", json!({ "path": "src/main.rs" })),
            ("write", "write", json!({ "path": "README.md" })),
        ] {
            apply_stream_event(
                &mut transcript,
                StreamEvent::ToolStart {
                    id: id.to_owned(),
                    name: name.to_owned(),
                    args,
                },
                None,
            );
        }

        let labels = transcript
            .iter()
            .map(|item| match item {
                TranscriptItem::Tool(tool) => tool_label(tool),
                _ => panic!("expected standalone tool"),
            })
            .collect::<Vec<_>>();
        assert_eq!(
            labels,
            [
                "Running cargo test",
                "Editing src/main.rs",
                "Writing README.md"
            ]
        );

        for id in ["bash", "edit", "write"] {
            apply_stream_event(&mut transcript, end(id, None, None), None);
        }
        let labels = transcript
            .iter()
            .map(|item| match item {
                TranscriptItem::Tool(tool) => tool_label(tool),
                _ => unreachable!(),
            })
            .collect::<Vec<_>>();
        assert_eq!(
            labels,
            ["Ran cargo test", "Edited src/main.rs", "Wrote README.md"]
        );
    }

    #[test]
    fn failed_edit_and_write_use_failure_labels() {
        let mut transcript = Vec::new();
        for (id, name, path) in [
            ("edit", "edit", "src/main.rs"),
            ("write", "write", "README.md"),
        ] {
            apply_stream_event(
                &mut transcript,
                StreamEvent::ToolStart {
                    id: id.to_owned(),
                    name: name.to_owned(),
                    args: json!({ "path": path }),
                },
                None,
            );
            apply_stream_event(
                &mut transcript,
                StreamEvent::ToolEnd {
                    id: id.to_owned(),
                    is_error: true,
                    error: Some("failed".to_owned()),
                    output: None,
                    highlighted_html: None,
                    added: None,
                    removed: None,
                },
                None,
            );
        }

        let labels = transcript
            .iter()
            .map(|item| match item {
                TranscriptItem::Tool(tool) => tool_label(tool),
                _ => unreachable!(),
            })
            .collect::<Vec<_>>();
        assert_eq!(
            labels,
            ["Failed to edit src/main.rs", "Failed to write README.md"]
        );
    }

    #[test]
    fn tool_failure_keeps_its_concise_error() {
        let mut transcript = Vec::new();
        apply_stream_event(
            &mut transcript,
            StreamEvent::ToolStart {
                id: "grep".to_owned(),
                name: "grep".to_owned(),
                args: json!({ "pattern": "missing" }),
            },
            None,
        );
        apply_stream_event(
            &mut transcript,
            StreamEvent::ToolEnd {
                id: "grep".to_owned(),
                is_error: true,
                error: Some("grep failed".to_owned()),
                output: None,
                highlighted_html: None,
                added: None,
                removed: None,
            },
            None,
        );

        let TranscriptItem::Exploration(tools) = &transcript[0] else {
            unreachable!()
        };
        assert_eq!(tools[0].state, ToolState::Failed);
        assert_eq!(tools[0].error.as_deref(), Some("grep failed"));
        assert!(tools[0].detail.is_none());
    }

    #[test]
    fn keeps_failures_visible_and_marks_all_active_tools() {
        let mut transcript = Vec::new();
        for (id, name) in [("read", "read"), ("bash", "bash")] {
            apply_stream_event(
                &mut transcript,
                StreamEvent::ToolStart {
                    id: id.to_owned(),
                    name: name.to_owned(),
                    args: json!({}),
                },
                None,
            );
        }

        assert!(fail_active_tools(
            &mut transcript,
            "request timed out\nmore details"
        ));
        for item in &transcript {
            let tool = match item {
                TranscriptItem::Exploration(tools) => &tools[0],
                TranscriptItem::Tool(tool) => {
                    assert_eq!(tool_label(tool), "Ran ...");
                    tool
                }
                _ => unreachable!(),
            };
            assert_eq!(tool.state, ToolState::Failed);
            assert_eq!(tool.error.as_deref(), Some("request timed out"));
        }
    }

    #[test]
    fn agent_output_is_detail_only_and_write_without_detail_is_static() {
        let mut transcript = Vec::new();
        apply_stream_event(
            &mut transcript,
            StreamEvent::ToolStart {
                id: "bash".to_owned(),
                name: "bash".to_owned(),
                args: json!({ "command": "printf hello" }),
            },
            None,
        );
        apply_stream_event(
            &mut transcript,
            end("bash", None, Some("<span>hello</span>")),
            None,
        );
        apply_stream_event(
            &mut transcript,
            StreamEvent::ToolStart {
                id: "write".to_owned(),
                name: "write".to_owned(),
                args: json!({ "path": "new.txt" }),
            },
            None,
        );
        apply_stream_event(&mut transcript, end("write", None, None), None);

        let TranscriptItem::Tool(bash) = &transcript[0] else {
            unreachable!()
        };
        assert!(bash.output.is_empty());
        assert!(bash.highlighted_html.is_none());
        assert!(matches!(
            bash.detail.as_ref().map(|detail| &detail.content),
            Some(ToolDetailContent::HighlightedHtml(html)) if html == "<span>hello</span>"
        ));
        let TranscriptItem::Tool(write) = &transcript[1] else {
            unreachable!()
        };
        assert!(write.detail.is_none());
    }

    #[test]
    fn records_edit_change_counts() {
        let mut transcript = Vec::new();
        apply_stream_event(
            &mut transcript,
            StreamEvent::ToolStart {
                id: "edit".to_owned(),
                name: "edit".to_owned(),
                args: json!({ "path": "src/main.rs" }),
            },
            None,
        );
        apply_stream_event(
            &mut transcript,
            StreamEvent::ToolEnd {
                id: "edit".to_owned(),
                is_error: false,
                error: None,
                output: None,
                highlighted_html: Some("<span>diff</span>".to_owned()),
                added: Some(4),
                removed: Some(2),
            },
            None,
        );
        let TranscriptItem::Tool(tool) = &transcript[0] else {
            unreachable!()
        };
        assert_eq!((tool.added, tool.removed), (Some(4), Some(2)));
        assert_eq!(tool.kind, ToolKind::Edit);
    }

    #[test]
    fn direct_shell_streams_and_remains_expanded_inline() {
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
            Some("<span>hello</span>".to_owned()),
        );

        let TranscriptItem::Tool(tool) = &transcript[0] else {
            unreachable!()
        };
        assert!(tool.direct_shell);
        assert_eq!(tool_label(tool), "$ printf hello");
        assert_eq!(tool.output, "hello");
        assert_eq!(tool.highlighted_html.as_deref(), Some("<span>hello</span>"));
        assert!(tool.detail.is_none());
    }

    #[test]
    fn maps_detail_open_modifiers() {
        assert_eq!(detail_open_mode(false, false), DetailOpenMode::FocusedTab);
        assert_eq!(detail_open_mode(false, true), DetailOpenMode::FocusedTab);
        assert_eq!(detail_open_mode(true, false), DetailOpenMode::BackgroundTab);
        assert_eq!(detail_open_mode(true, true), DetailOpenMode::Window);
    }

    #[test]
    fn strips_thinking_emphasis_markers() {
        assert_eq!(
            plain_thinking("**Checking the details.**"),
            "Checking the details."
        );
        assert_eq!(
            plain_thinking("Checking **literal** details."),
            "Checking **literal** details."
        );
    }

    #[test]
    fn streams_thinking_before_assistant_text() {
        let mut transcript = vec![TranscriptItem::User("Solve it".to_owned())];
        apply_stream_event(&mut transcript, StreamEvent::AssistantStart, None);
        apply_stream_event(
            &mut transcript,
            StreamEvent::ThinkingStart {
                id: "thinking-1".to_owned(),
            },
            None,
        );
        apply_stream_event(
            &mut transcript,
            StreamEvent::ThinkingDelta {
                id: "thinking-1".to_owned(),
                delta: "Checking".to_owned(),
            },
            None,
        );
        apply_stream_event(
            &mut transcript,
            StreamEvent::TextDelta("Done.".to_owned()),
            None,
        );
        apply_stream_event(
            &mut transcript,
            StreamEvent::ThinkingEnd {
                id: "thinking-1".to_owned(),
                content: "Checked the details.".to_owned(),
            },
            None,
        );

        assert_eq!(
            transcript,
            vec![
                TranscriptItem::User("Solve it".to_owned()),
                TranscriptItem::Thinking {
                    id: "thinking-1".to_owned(),
                    text: "Checked the details.".to_owned(),
                },
                TranscriptItem::Assistant("Done.".to_owned()),
            ]
        );
    }

    #[test]
    fn renders_bash_exit_and_truncation_details() {
        let (rendered, highlighted_html) = render_bash_result(&BashOutcome {
            output: "failed".to_owned(),
            exit_code: Some(7),
            cancelled: false,
            truncated: true,
            full_output_path: Some("/tmp/full<&>.log".to_owned()),
            highlighted_html: Some("<span>failed</span>".to_owned()),
        });

        assert_eq!(
            rendered,
            "failed\n[exit 7]\n[output truncated; full output: /tmp/full<&>.log]\n"
        );
        assert_eq!(
            highlighted_html.as_deref(),
            Some(
                "<span>failed</span>\n[exit 7]\n[output truncated; full output: /tmp/full&lt;&amp;&gt;.log]\n"
            )
        );
    }
}
