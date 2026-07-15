# Crust

A minimal desktop interface for the [Pi agent harness](https://pi.dev), built with [Dioxus](https://dioxuslabs.com/).

The Rust UI keeps a Node.js sidecar alive over JSONL/stdin. The sidecar uses Pi's `@earendil-works/pi-coding-agent` SDK directly and streams assistant text back to Dioxus. It uses your existing Pi credentials and settings from `~/.pi/agent` and keeps the conversation in memory for the lifetime of the app.

## Requirements

- Rust 1.85 or newer (edition 2024)
- Node.js 22.19 or newer
- A configured Pi installation with an available model

Dioxus Desktop needs no additional system packages on macOS. See the Dioxus desktop prerequisites when building on Linux or Windows.

## Run

```sh
npm install
cargo run
```

The agent works in the Crust project directory by default. Override that directory or the Node executable when needed:

```sh
CRUST_AGENT_CWD=/path/to/project cargo run
CRUST_NODE=/absolute/path/to/node cargo run
CRUST_PROMPT_TIMEOUT_SECS=3600 cargo run
```

## Validate

```sh
cargo fmt --check
cargo check
cargo clippy --all-targets -- -D warnings
```

## Acknowledgments

Crust's development philosophy is adapted from the [suckless.org philosophy](https://suckless.org/philosophy/).
