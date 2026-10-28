use crate::cli::CliArgs;
use crate::search::FileSearchResult;
use crate::slice::SliceResult;
use serde_json::json;

pub fn render_version() -> String {
    format!("rg-md {}\n", env!("CARGO_PKG_VERSION"))
}

pub fn render_help() -> String {
    r#"rg-md — Sub-100ms Markdown-formatted ripgrep search & code extraction CLI for AI agents

USAGE:
  rg-md <pattern> [path]                  Fast ripgrep search in markdown list format (default)
  rg-md search <pattern> [path]           Explicit search command
  rg-md find <glob> [path]                Fast file discovery and path inventory
  rg-md tree [path]                       Hierarchical directory topology and module scoping
  rg-md slice <target...> [path]          Surgical range/window extractor (e.g. file:10-50, symbol)
  rg-md diff [file1 file2]                Compact unified diff without raw patch noise
  rg-md err <command...>                  Execute build/lint command and filter compiler errors
  rg-md test <command...>                 Execute test suite and isolate failures
  rg-md pipe [text]                       Filter and format raw piped stdin or text streams

ALIASES:
  search, q, query, grep                  Search code patterns
  find, files                             Find matching files
  tree, ls, map, overview                 Hierarchical directory structure
  slice, view, inspect, cat, show, def   Inspect surgical line ranges or symbol definitions
  diff                                    Unified diff or working tree changes
  err, errors                             Filter compiler/linter error output
  test, tests                             Filter test suite failure output
  pipe                                    Parse and format piped stdin

FLAGS:
  -p, --dir, --path <path>                Directory or file scope
  -b, --budget <tokens>                   Token budget (default: 2000 tokens / 8000 chars; auto-truncates output)
      --unlimited                         Disable budget constraints entirely (alias: --no-budget, -b 0)
      --max-chars <chars>                 Character budget override
      --raw                               Print clean raw code snippet without Markdown list prefixes (slice)
      --head <n>                          Show first n items/lines
      --tail <n>                          Show last n items/lines
  -d, --depth <n>                         Max directory traversal depth (tree)
  -C, --context <lines>                   Context lines before and after match
  -i, --ignore-case                       Case-insensitive search
      --hidden                            Search hidden files
      --json                              Emit structured JSON output
  -h, --help                              Show this help message
  -V, --version                           Show version
"#
    .to_string()
}

pub fn truncation_notice(budget: crate::cli::Budget) -> String {
    format!(
        "\n  ... [truncated: budget limit reached ({}). Use -b <N> or --unlimited to expand] ...\n",
        budget.limit_description()
    )
}

pub fn format_search_results(results: &[FileSearchResult], args: &CliArgs) -> String {
    if args.json {
        let total_m: usize = results.iter().map(|r| r.matches.len()).sum();
        let val = json!({
            "command": "search",
            "query": &args.query,
            "files_matched": results.len(),
            "total_matches": total_m,
            "results": results,
        });
        return serde_json::to_string_pretty(&val).unwrap_or_default();
    }

    let mut out = String::new();
    let limit = args.budget.effective_char_limit();
    let mut truncated = false;

    for res in results {
        if res.matches.is_empty() {
            continue;
        }

        let header = format!("\n### `{}`\n", res.path);
        if out.len() + header.len() > limit {
            truncated = true;
            break;
        }
        out.push_str(&header);

        for m in &res.matches {
            let line = format!("  - L{}: {}\n", m.line, m.content);
            if out.len() + line.len() > limit {
                truncated = true;
                break;
            }
            out.push_str(&line);
        }

        if truncated {
            break;
        }
    }

    if truncated {
        out.push_str(&truncation_notice(args.budget));
    }

    out
}

pub fn format_slice_results(results: &[SliceResult], args: &CliArgs) -> String {
    if args.json {
        let val = json!({
            "command": "slice",
            "targets_count": results.len(),
            "results": results,
        });
        return serde_json::to_string_pretty(&val).unwrap_or_default();
    }

    if args.raw {
        let mut out = String::new();
        let limit = args.budget.effective_char_limit();
        let mut truncated = false;

        for res in results {
            if res.lines.is_empty() {
                let msg = format!(
                    "Target `{}` could not be resolved or file is empty\n",
                    res.target
                );
                if out.len() + msg.len() > limit {
                    truncated = true;
                    break;
                }
                out.push_str(&msg);
                continue;
            }

            for l in &res.lines {
                let line = format!("{}\n", l.raw);
                if out.len() + line.len() > limit {
                    truncated = true;
                    break;
                }
                out.push_str(&line);
            }

            if truncated {
                break;
            }
        }

        if truncated {
            out.push_str(&truncation_notice(args.budget));
        }

        return out;
    }

    let mut out = String::new();
    let limit = args.budget.effective_char_limit();
    let mut truncated = false;

    for res in results {
        let header = if let Some(sym) = &res.symbol {
            if res.start_line > 0 && res.end_line > 0 {
                format!(
                    "\n### `{}:{}-{}` (symbol: {})\n",
                    res.file_path, res.start_line, res.end_line, sym
                )
            } else {
                format!("\n### `{}` (symbol: {})\n", res.file_path, sym)
            }
        } else if res.start_line > 0 && res.end_line > 0 {
            format!(
                "\n### `{}:{}-{}`\n",
                res.file_path, res.start_line, res.end_line
            )
        } else {
            format!("\n### `{}`\n", res.file_path)
        };

        if out.len() + header.len() > limit {
            truncated = true;
            break;
        }
        out.push_str(&header);

        if res.lines.is_empty() {
            let msg = format!(
                "  - Target `{}` could not be resolved or file is empty\n",
                res.target
            );
            if out.len() + msg.len() > limit {
                truncated = true;
                break;
            }
            out.push_str(&msg);
            continue;
        }

        for l in &res.lines {
            let line = format!("  - L{}: {}\n", l.line, l.content);
            if out.len() + line.len() > limit {
                truncated = true;
                break;
            }
            out.push_str(&line);
        }

        if truncated {
            break;
        }
    }

    if truncated {
        out.push_str(&truncation_notice(args.budget));
    }

    out
}

pub fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

pub fn truncate_to_budget(text: &str, budget: crate::cli::Budget) -> String {
    let limit = budget.effective_char_limit();
    if text.len() <= limit {
        return text.to_string();
    }

    let mut out = String::new();
    for line in text.lines() {
        if out.len() + line.len() + 1 > limit {
            break;
        }
        out.push_str(line);
        out.push('\n');
    }

    out.push_str(&truncation_notice(budget));
    out
}
