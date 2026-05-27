# http-tui

[![CI](https://github.com/axs15/http-tui/actions/workflows/ci.yml/badge.svg)](https://github.com/axs15/http-tui/actions/workflows/ci.yml)

A terminal UI for browsing HTTP API responses, built with [Ratatui](https://ratatui.rs).

## Features

- Polls a JSON API endpoint every 5 seconds and displays results in a table
- Non-blocking async I/O via Tokio
- Keyboard-driven interface

## Usage

```
cargo run
```

Press `q` or `Esc` to quit.

## Keybindings

| Key | Action |
|-----|--------|
| `q` / `Esc` | Quit |
| `Ctrl+C` | Quit |

## Building

```
cargo build --release
```
