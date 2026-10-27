# Rust Source Code (`src/`)

This directory contains the Rust implementation for `rg-md` and its CLI alias `md-rg`:
- `main.rs` - Application entrypoint and CLI dispatcher.
- `lib.rs` - Library interface exposing core parsing, execution, and formatting modules.
- `cli.rs` - Command-line argument parsing and option forwarding.
- `engine.rs` - Process execution wrapper calling the ripgrep binary with heading and line number flags.
- `formatter.rs` - Stream transformer converting raw ripgrep output into hierarchical Markdown.
