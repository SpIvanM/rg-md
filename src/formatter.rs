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
  rg-md slice <target...> [path]          Surgical range/window extractor (e.g. file:10-50, symbol)
  rg-md map [path]                        Repository file tree & path inventory

ALIASES:
  search, find, q                         Search code patterns
  slice, view, inspect                    Inspect surgical line ranges or symbol definitions
  map, tree, ls                           List files and structure

FLAGS:
  -p, --dir, --path <path>                Directory or file scope to search
  -b, --budget <tokens>                   Token budget (1 token ~ 4 chars; auto-truncates output)
      --max-chars <chars>                 Character budget override
  -C, --context <lines>                   Context lines before and after match
  -i, --ignore-case                       Case-insensitive search
      --hidden                            Search hidden files
      --json                              Emit structured JSON output
  -h, --help                              Show this help message
  -V, --version                           Show version
"#
    .to_string()
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
        out.push_str("\n  ... [truncated: budget limit reached] ...\n");
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
        out.push_str("\n  ... [truncated: budget limit reached] ...\n");
    }

    out
}
