# Spigot

A minimal desktop interface for the [Pi agent harness](https://pi.dev), built with [Dioxus](https://dioxuslabs.com/).

The Rust UI keeps a Node.js sidecar alive over JSONL/stdin. The sidecar uses Pi's `@earendil-works/pi-coding-agent` SDK directly and streams assistant text and shell output back to Dioxus. It uses your existing Pi credentials and settings from `~/.pi/agent` and keeps the conversation in memory for the lifetime of the app.

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

The agent works in the Spigot project directory by default. Override that directory or the Node executable when needed:

```sh
SPIGOT_AGENT_CWD=/path/to/project cargo run
SPIGOT_NODE=/absolute/path/to/node cargo run
SPIGOT_PROMPT_TIMEOUT_SECS=3600 cargo run
```

Type a normal message to prompt Pi. Prefix input with `!` to run a shell command in the agent working directory; its output is added to Pi's context. Use `!!` to run a command without adding its output to the context. Commands run with the same permissions as Spigot.

## Validate

```sh
cargo fmt --check
cargo check
cargo clippy --all-targets -- -D warnings
```

## Acknowledgments

Spigot uses the neutral palette from [Signal by UNMS](https://signal.un.ms/) with `#FF5700` as its accent. The bundled Inter font is licensed under the [SIL Open Font License](assets/InterVariable-LICENSE.txt).

Spigot's development philosophy is adapted from the [suckless.org philosophy](https://suckless.org/philosophy/).
