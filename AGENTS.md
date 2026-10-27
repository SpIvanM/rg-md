# AGENTS.md — Workspace Engineering Rules (rg-md)

## 1. Mission
rg-md is a high-speed Rust command-line tool that formats ripgrep code searches into structured Markdown headings and bullet points. Its purpose is to minimize agent turn counts, billed context tokens, and overall inference costs during codebase exploration.

## 2. Architecture & Module Boundaries
- `src/main.rs` — Command-line entrypoint for `rg-md` and alias `md-rg`.
- `src/lib.rs` — Library interface exposing parsing and formatting modules.
- `src/cli.rs` — Command-line argument parsing and delegation to ripgrep.
- `src/engine.rs` — Execution wrapper invoking ripgrep subprocess with heading flags.
- `src/formatter.rs` — Markdown parser and emitter converting raw ripgrep output into hierarchical Markdown.
- `tests/` — Automated integration tests validating formatting, exit codes, and empty results.
- `scripts/` — Installation automation and telemetry benchmark analyzers.

## 3. Engineering Standards
- Keep crate dependencies light and compile times under two seconds.
- Maintain sub-50ms execution overhead over raw ripgrep.
- Preserve zero-allocation or single-pass stream formatting where possible.
- Run test suite via `cargo test` before submitting changes.
- Store transient test artifacts in `tmp/` and design notes in `.work/`.

## 4. Codebase Navigation with rg-md
- Execute rg-md or md-rg directly without prerequisite verification steps.
- ‼ If invoking rg-md or md-rg, checking presence via --version or --help before use is strictly prohibited.
- Run rg-md "<pattern>" [path] [ripgrep-options...] for structured markdown search.
- Switch immediately to targeted read_file once relevant files and line numbers are located.
- ‼ If locating identifiers, blind recursive grep without path scope is strictly prohibited.
- ‼ If exploring codebase or symbols, reading whole files with read_file is strictly prohibited.
- Reserve read_file strictly for small line ranges (under 50 lines) pinpointed by search results.
- Conclude exploration once target symbols and modification sites are mapped.

# >>> rg-md navigation rules >>>
## Codebase Navigation Directives
- Run rg-md "<pattern>" [path] [ripgrep-options...] for structured markdown search.
- Switch immediately to targeted read_file once relevant files and line numbers are located.
- ‼ If locating identifiers, blind recursive grep without path scope is strictly prohibited.
- ‼ If exploring codebase or symbols, reading whole files with read_file is strictly prohibited.
- Reserve read_file strictly for small line ranges (under 50 lines) pinpointed by search results.
# <<< rg-md navigation rules <<<
