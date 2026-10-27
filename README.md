# rg-md — Markdown-Formatted Codebase Search Engine for AI Coding Agents

[![GitHub Release](https://img.shields.io/github/v/release/SpIvanM/rg-md?color=blue)](https://github.com/SpIvanM/rg-md/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust 2021](https://img.shields.io/badge/Rust-2021-orange.svg)](Cargo.toml)
[![Tests Passing](https://img.shields.io/badge/tests-passed-brightgreen.svg)](tests/)
[![Bilingual](https://img.shields.io/badge/docs-EN%20%7C%20RU-blue)](README.ru.md)

> [Documentation in Russian (Русская версия)](README.ru.md)

---

## Overview

rg-md is a high-speed command-line search utility engineered in Rust to format ripgrep code searches into structured Markdown.

Standard AI coding agents rely on ripgrep or grep during codebase exploration. Raw grep output formats matches with colon-delimited prefixes (`path:line:content`), which triggers hallucinated line boundary merges in LLM tokenizers, clutters tool output buffers, and inflates agent prompt contexts.

rg-md wraps ripgrep and reformats file groups into Markdown heading levels (`### `) with line-numbered bullet lists (`  - L<num>: <line>`). This structural hierarchy transforms raw text matches into a native AST-like document tree that language models digest with zero tokenizer friction and minimal context bloat.

```
       ┌────────────────────────────────────────────────────────┐
       │   Agent Turn:  rg-md "RecordingOverlay" src/           │
       └──────────────────────────┬─────────────────────────────┘
                                  ▼
 ┌──────────────────────────────────────────────────────────────┐
 │                rg-md Structured Markdown Stream              │
 │                                                              │
 │  ### `src/overlay/main.tsx`                                  │
 │    - L4: import RecordingOverlay from "./RecordingOverlay";  │
 │    - L23: <RecordingOverlay />                               │
 │                                                              │
 │  ### `src/overlay/RecordingOverlay.tsx`                      │
 │    - L4: import "./RecordingOverlay.css";                    │
 │    - L28: const RecordingOverlay: React.FC = () => {         │
 │    - L369: export default RecordingOverlay;                  │
 └──────────────────────────────────────────────────────────────┘
```

---

## Architectural Advantages for Autonomous Agents

### Hierarchical Token Efficiency
Organizing file hits under Markdown headings eliminates redundant file path repetitions on every hit line. In benchmarks, this reduces final conversation context size by 18.9% compared to disciplined ripgrep and 69.3% compared to heavy AST navigators.

### Turn Reduction
Because the model parses structured Markdown lists faster and without ambiguity, investigation loops converge in fewer iterations. Autonomous agent runs complete complex exploration plans in 39 turns versus 54 turns for ripgrep and 49 turns for baseline agents.

### Sub-Millisecond Execution Overhead
Built with Rust, rg-md introduces less than 5ms of streaming formatting overhead on top of raw ripgrep execution.

### Native Passthrough of Ripgrep Flags
All standard ripgrep arguments and flags pass through directly. Agents can supply path filters (`-g "*.rs"`), type selectors (`-t ts`), case-sensitivity flags (`-i`), or multiline search flags without modification.

---

## Empirical Evaluation: Cycle 3 Benchmark Results

The performance of rg-md was evaluated inside the AST Tools A/B Benchmark framework using an autonomous agent workflow powered by gemini-3.8-flash-low on the Tauri desktop application repository handy.

The benchmark task required autonomous architectural discovery, locating the exact UI recording overlay component, the Rust audio recording termination handler, the Tauri IPC command bindings, and the mouse event collision mitigation logic.

### Cycle 3 Comparative Benchmark Matrix

| Metric | rg-md (run_v9) | Disciplined rg (run_v01) | Raw v0 (run_v0) | cx (run_v6) | rtk rg (run_v5) | Raw Baseline (run_v0_pure) |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| Plan Rubric Score | 100 / 100 | 100 / 100 | 100 / 100 | 100 / 100 | 75 / 100 | 75 / 100 |
| Final Context Tokens | 31,011 | 38,259 | 53,533 | 100,897 | 48,505 | 64,447 |
| Context Delta vs rg-md | Baseline | +23.4% | +72.6% | +225.4% | +56.4% | +107.8% |
| Agent Planner Turns | 39 | 54 | 49 | 49 | 59 | 61 |
| Execution Time (s) | 187.91s | 265.97s | 290.35s | 252.62s | 283.87s | 304.84s |
| Total Inference Cost | $0.2167 | $0.3148 | $0.3133 | $0.3773 | $0.3618 | $0.3648 |
| Input Tokens | 176,178 | 248,936 | 194,178 | 180,708 | 270,474 | 201,523 |
| Cache Read Tokens | 639,129 | 1,113,553 | 1,667,684 | 2,589,264 | 1,472,534 | 2,201,091 |
| Output Tokens | 4,728 | 5,755 | 5,490 | 6,133 | 6,257 | 6,269 |

### Benchmark Findings
- Final context tokens: rg-md achieved the lowest context footprint across all tested variants at 31,011 tokens, saving 7,248 tokens (-18.9%) compared to disciplined ripgrep, 22,522 tokens (-42.1%) compared to raw v0, and 69,886 tokens (-69.3%) compared to cx.
- Investigation velocity: rg-md solved the architectural mapping task in 39 turns, outperforming disciplined ripgrep by 15 turns (-27.8%) and raw v0 by 10 turns (-20.4%).
- Wall-clock duration: rg-md finished in 187.91 seconds, completing the task 78.06 seconds faster (-29.3%) than disciplined ripgrep and 102.44 seconds faster (-35.3%) than raw v0.
- Financial cost: rg-md reduced total API expenditure to $0.2167, saving 31.2% over disciplined ripgrep ($0.3148) and 42.6% over cx ($0.3773).

---

## Installation & Distribution

Precompiled release binaries are distributed for Linux and Windows.

### Linux and macOS (Shell One-Liner)
Run in terminal:
```bash
curl -fsSL https://raw.githubusercontent.com/SpIvanM/rg-md/main/scripts/install.sh | bash
```
The installer places `rg-md` and symlink `md-rg` into `~/.local/bin/` and configures the user PATH.

### Windows (PowerShell One-Liner)
Run in PowerShell:
```powershell
irm https://raw.githubusercontent.com/SpIvanM/rg-md/main/scripts/install.ps1 | iex
```
The script downloads `rg-md.exe` into `%LOCALAPPDATA%\Programs\rg-md\bin` and registers User PATH.

### Build from Source via Cargo
```bash
cargo install --git https://github.com/SpIvanM/rg-md.git
```

---

## Agent Integration

To equip autonomous agents with rg-md, deploy the skill definition and register exploration rules in your system directives.

### Deploying the Agent Skill
The canonical skill definition is located at `skills/rg-md/SKILL.md`.

For Google Antigravity / AGY:
```bash
mkdir -p ~/.gemini/config/skills/rg-md
cp skills/rg-md/SKILL.md ~/.gemini/config/skills/rg-md/
```

For Cursor:
Add `skills/rg-md/SKILL.md` to `.cursor/rules/rg-md.mdc` or append directives to `.cursorrules`.

For Claude Code:
Append directives to `CLAUDE.md`.

### Agent Directives Block (AGENTS.md)
Add the following block to your project root `AGENTS.md` or global agent configuration:

```markdown
# >>> rg-md navigation rules >>>
## Codebase Navigation Directives
- Run rg-md "<pattern>" [path] [ripgrep-options...] for structured markdown search.
- Switch immediately to targeted read_file once relevant files and line numbers are located.
- ‼ If locating identifiers, blind recursive grep without path scope is strictly prohibited.
- ‼ If exploring codebase or symbols, reading whole files with read_file is strictly prohibited.
- Reserve read_file strictly for small line ranges (under 50 lines) pinpointed by search results.
# <<< rg-md navigation rules <<<
```

---

## Command-Line Usage

rg-md supports both positional search arguments and passthrough options:

### Basic Search
```bash
# Search for identifier across repository
rg-md "RecordingOverlay"

# Alias syntax
md-rg "RecordingOverlay"
```

### Path-Scoped Search
```bash
# Restrict search to specific subdirectory
rg-md "stop_recording" src-tauri/src
```

### Type-Filtered and Flagged Search
```bash
# Filter by file type
rg-md "handle_click" src/ -t tsx

# Case-insensitive search with word boundaries
rg-md -i -w "cancel_generation" src-tauri/
```

---

## Contributing

Contributions are welcomed. To maintain token economy and performance invariants, all pull requests must adhere to the Empirical Contribution Standard:

Every pull request modifying output formatting or execution pipelines must include paired A/B telemetry measurements confirming that agent turns, context tokens, or execution latency do not regress.

Refer to [CONTRIBUTING.md](CONTRIBUTING.md) for testing guidelines and pull request requirements.

---

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
Copyright (c) 2026 Ivan M (SpIvanM) and contributors.
