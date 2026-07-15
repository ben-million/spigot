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

    .shell-prompt {
        color: var(--accent);
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
        max-height: 15em;
        margin-top: 8px;
        overflow-y: auto;
        color: var(--muted);
        scrollbar-color: var(--border) transparent;
        scrollbar-width: thin;
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

#[derive(Clone, Debug, PartialEq)]
enum TranscriptItem {
    User(String),
    Assistant(String),
    Thinking {
        id: String,
        text: String,
    },
    Tool {
        id: String,
        summary: String,
        state: ToolState,
        error: Option<String>,
        output: String,
        highlighted_html: Option<String>,
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
        pi::StreamEvent::ThinkingStart { id } => {
            if transcript.last().is_some_and(
                |item| matches!(item, TranscriptItem::Assistant(text) if text.is_empty()),
            ) {
                transcript.pop();
            }
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
            transcript.push(TranscriptItem::Tool {
                id,
                summary: tool_summary(&name, &args),
                state: ToolState::Active,
                error: None,
                output: String::new(),
                highlighted_html: None,
            });
        }
        pi::StreamEvent::ToolEnd {
            id,
            is_error,
            error,
            output,
            highlighted_html,
        } => {
            if let Some(TranscriptItem::Tool {
                state,
                error: tool_error,
                output: tool_output,
                highlighted_html: tool_highlighted_html,
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
                if let Some(output) = output {
                    *tool_output = output;
                }
                *tool_highlighted_html = highlighted_html;
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
        highlighted_html: None,
    });
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
    if let Some(TranscriptItem::Tool {
        state: tool_state,
        error: tool_error,
        output: tool_output,
        highlighted_html: tool_highlighted_html,
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
        *tool_highlighted_html = highlighted_html;
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
        TranscriptItem::Tool {
            summary,
            state,
            error,
            output,
            highlighted_html,
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
                        if let Some(command) = summary.strip_prefix("$ ") {
                            span {
                                span { class: "shell-prompt", "$" }
                                " {command}"
                            }
                        } else {
                            span { "{summary}" }
                        }
                    }
                    if let Some(error) = error {
                        div { class: "tool-error", "{error}" }
                    }
                    if let Some(highlighted_html) = highlighted_html {
                        // This HTML is generated locally by highlight.js, which escapes source text.
                        pre {
                            class: "tool-output highlighted-output",
                            dangerous_inner_html: highlighted_html,
                        }
                    } else if !output.is_empty() {
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

fn window_builder() -> dioxus::desktop::WindowBuilder {
    let builder = dioxus::desktop::WindowBuilder::new()
        .with_title("Spigot")
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

fn window_config() -> dioxus::desktop::Config {
    let config = dioxus::desktop::Config::new().with_window(window_builder());

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
) {
    use dioxus::desktop::tao::platform::macos::WindowExtMacOS;
    use objc2_app_kit::{NSWindow, NSWindowOrderingMode};

    unsafe {
        // Both contexts keep their Tao windows alive, and Dioxus polls this task on the main thread.
        let source_window = &*source.window.ns_window().cast::<NSWindow>();
        let tab_window = &*tab.window.ns_window().cast::<NSWindow>();
        source_window.addTabbedWindow_ordered(tab_window, NSWindowOrderingMode::Above);
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
                        .new_window(VirtualDom::new(App), window_config())
                        .await;
                    group_window_as_tab(&source, &tab);
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
        .with_cfg(window_config())
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
        APP_STYLE, ToolState, TranscriptItem, apply_stream_event, fail_active_tools,
        plain_thinking, push_shell, render_bash_result, tool_summary, update_shell,
    };
    use crate::pi::{BashOutcome, StreamEvent};
    use serde_json::json;

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
                output: None,
                highlighted_html: Some("<span class=\"hljs-keyword\">fn</span>".to_owned()),
            },
            None,
        );
        apply_stream_event(
            &mut transcript,
            StreamEvent::ToolEnd {
                id: "grep-1".to_owned(),
                is_error: true,
                error: Some("grep failed".to_owned()),
                output: None,
                highlighted_html: None,
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
                highlighted_html: Some(highlighted_html),
                ..
            } if highlighted_html == "<span class=\"hljs-keyword\">fn</span>"
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
            StreamEvent::ThinkingDelta {
                id: "thinking-1".to_owned(),
                delta: " the details.".to_owned(),
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
    fn request_failure_marks_active_tools_without_a_duplicate_error() {
        let mut transcript = vec![
            TranscriptItem::Tool {
                id: "active".to_owned(),
                summary: "read src/main.rs".to_owned(),
                state: ToolState::Active,
                error: None,
                output: String::new(),
                highlighted_html: None,
            },
            TranscriptItem::Tool {
                id: "complete".to_owned(),
                summary: "ls .".to_owned(),
                state: ToolState::Complete,
                error: None,
                output: String::new(),
                highlighted_html: None,
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
    fn shows_agent_shell_output() {
        let mut transcript = Vec::new();
        apply_stream_event(
            &mut transcript,
            StreamEvent::ToolStart {
                id: "bash-1".to_owned(),
                name: "bash".to_owned(),
                args: json!({ "command": "printf hello" }),
            },
            None,
        );
        apply_stream_event(
            &mut transcript,
            StreamEvent::ToolEnd {
                id: "bash-1".to_owned(),
                is_error: false,
                error: None,
                output: Some("hello".to_owned()),
                highlighted_html: Some("<span>hello</span>".to_owned()),
            },
            None,
        );

        assert!(matches!(
            &transcript[0],
            TranscriptItem::Tool {
                summary,
                state: ToolState::Complete,
                output,
                highlighted_html: Some(highlighted_html),
                ..
            } if summary == "$ printf hello"
                && output == "hello"
                && highlighted_html == "<span>hello</span>"
        ));
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
            Some("<span>hello</span>".to_owned()),
        );

        assert!(matches!(
            &transcript[0],
            TranscriptItem::Tool {
                summary,
                state: ToolState::Complete,
                output,
                highlighted_html: Some(highlighted_html),
                ..
            } if summary == "$ printf hello"
                && output == "hello"
                && highlighted_html == "<span>hello</span>"
        ));
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
