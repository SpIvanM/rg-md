---
name: rg-md
description: "Use when searching codebase, locating symbol usages, or finding file occurrences with structured Markdown output"
origin: IAM
version: 1.0.0
---

# Codebase Search with rg-md

Goal: minimize agent turns, context token bloat, and exploration cost using rg-md.

## Execution Directives
- Execute rg-md or md-rg directly without prerequisite verification steps.
- ‼ If invoking rg-md or md-rg, checking presence via --version or --help before use is strictly prohibited.
- If rg-md execution fails or is not found, see [installation.md](./references/installation.md).
- Run rg-md "<pattern>" [path] [ripgrep-options...] for structured markdown search.
- Switch immediately to targeted read_file once relevant files and line numbers are located.
- Prefer targeted subdirectories over repository root when searching domain-specific identifiers.
- Pass standard ripgrep flags directly to refine matches: -t for file types, -i for case insensitivity, -w for word matches.
- Conclude exploration immediately once target change sites and file paths are identified.
- ‼ If exploring codebase or symbols, reading whole files with read_file is strictly prohibited.
- ‼ If locating identifiers, blind recursive search without path scope is strictly prohibited.
- Reserve read_file strictly for small line ranges (under 50 lines) pinpointed by search results.
