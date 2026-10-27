# Contributing to rg-md

> [Read in Russian (Русская версия)](CONTRIBUTING.ru.md)

Thank you for your interest in contributing to rg-md.
The mission of this project is to minimize agent turn counts, wall-clock latency, and billed context tokens during automated codebase exploration.

To preserve these efficiency guarantees, all contributions follow the Empirical Contribution Standard.

---

## Core Empirical Contribution Rule

Every pull request modifying output formatting, CLI parsing, or process execution must include reproducible A/B measurements demonstrating a confirmed positive effect without regressions.

Submissions based purely on subjective aesthetics are rejected. Every change must be validated against quantitative metrics:
- Zero latency regression: Execution overhead over ripgrep must remain under 10ms.
- Zero context token inflation: Formatting must not introduce redundant whitespace, prefixes, or verbose metadata.
- Confirmed agent economy: Pull requests must demonstrate equal or reduced agent turns and token consumption on benchmark tasks.

---

## Contribution Workflow

### Step 1: Create a Feature Branch
```bash
git checkout -b feat/my-improvement
```

### Step 2: Implement Rust Changes
- Adhere to module boundaries:
  - `src/main.rs`: CLI dispatch and binary alias support.
  - `src/cli.rs`: Argument extraction and argument passthrough.
  - `src/engine.rs`: Subprocess spawning and output stream capturing.
  - `src/formatter.rs`: Stream transformation and Markdown rendering.
- Keep dependencies minimal and avoid external heavy runtime dependencies.

### Step 3: Run Automated Test Suites
Add integration tests for any new formatting flag or edge case:
```bash
cargo test
```
All unit and integration tests must pass with zero failures.

### Step 4: Measure Latency
Ensure the release build overhead is minimal:
```bash
cargo build --release
time target/release/rg-md "MyPattern" /path/to/repo
```

### Step 5: Verify A/B Telemetry
Run the telemetry analyzer to compare baseline ripgrep or previous rg-md against your changes:
```bash
python scripts/bench_telemetry.py
```
Include the comparison table in your pull request description.

---

## Pull Request Template

Every pull request description must include the following verification section:

```markdown
### Empirical A/B Telemetry Impact

| Metric | Previous rg-md | Modified rg-md | Delta (%) |
| :--- | :---: | :---: | :---: |
| Agent Planner Turns | 39 | 38 | -2.6% |
| Final Context Tokens | 31,011 | 30,500 | -1.6% |
| Active Wall Time (s) | 187.91s | 184.20s | -2.0% |
| Total Inference Cost | $0.2167 | $0.2130 | -1.7% |
| Command Latency (ms) | 4.2ms | 4.1ms | -0.1ms |

- Target Repository: owner/repo
- Model Evaluated: gemini-3.8-flash-low / flash / pro
- Change Summary: Detailed explanation of modified parsing or output logic
```

---

## Architectural Invariants

- Streaming Markdown output: Never buffer unbounded gigabytes of output in memory before writing to stdout.
- Exact line numbers: Never alter or drop line numbers produced by ripgrep.
- Dual platform compatibility: Ensure identical behavior across Linux, macOS, and Windows.
- UTF-8 safety: Prevent panics when encountering non-ASCII or multibyte character boundaries.

---

## License

By contributing to rg-md, you agree that your contributions are licensed under the MIT License.
