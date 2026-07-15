# Spigot

Spigot is a small desktop interface for the [Pi agent harness](https://pi.dev). It provides a focused conversation buffer, a prompt, and direct access to shell commands without trying to become an editor, terminal emulator, or general-purpose IDE.

The interface is written in Rust with Dioxus and communicates with Pi's Node.js SDK over a plain JSONL stream. It reuses Pi's existing credentials and configuration, works in a chosen project directory, and keeps the conversation in memory only for the lifetime of the app.
