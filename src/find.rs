use crate::cli::CliArgs;
use crate::formatter::{format_bytes, truncate_to_budget};
use crate::scanner::collect_files;
use serde::Serialize;
use serde_json::json;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct FindFileItem {
    pub path: String,
    pub size_bytes: u64,
}

pub fn glob_to_regex(pattern: &str) -> Result<regex::Regex, regex::Error> {
    let mut regex_str = String::from("^");
    let mut chars = pattern.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '*' => {
                if chars.peek() == Some(&'*') {
                    chars.next();
                    if chars.peek() == Some(&'/') {
                        chars.next();
                        regex_str.push_str("(?:.*/)?");
                    } else {
                        regex_str.push_str(".*");
                    }
                } else {
                    regex_str.push_str("[^/]*");
                }
            }
            '?' => regex_str.push_str("[^/]"),
            '.' | '(' | ')' | '+' | '|' | '^' | '$' | '@' | '%' => {
                regex_str.push('\\');
                regex_str.push(c);
            }
            _ => regex_str.push(c),
        }
    }
    regex_str.push('$');
    regex::RegexBuilder::new(&regex_str)
        .case_insensitive(true)
        .build()
}

pub fn execute_find(repo_root: &Path, scope_rel: Option<&str>, args: &CliArgs) -> String {
    let glob_pat = if !args.query.is_empty() {
        &args.query
    } else if !args.targets.is_empty() {
        &args.targets[0]
    } else {
        "*"
    };

    let matcher = match glob_to_regex(glob_pat) {
        Ok(r) => r,
        Err(_) => {
            // Fallback to substring matching regex
            let escaped = regex::escape(glob_pat);
            regex::RegexBuilder::new(&escaped)
                .case_insensitive(true)
                .build()
                .unwrap()
        }
    };

    let scan_dir = if let Some(scope) = scope_rel {
        repo_root.join(scope)
    } else if let Some(p) = &args.path {
        if p.is_absolute() {
            p.clone()
        } else {
            repo_root.join(p)
        }
    } else {
        repo_root.to_path_buf()
    };

    let all_files = collect_files(&scan_dir, args.hidden);
    let mut matched_items: Vec<FindFileItem> = Vec::new();

    for abs_path in all_files {
        let rel = match abs_path.strip_prefix(repo_root) {
            Ok(r) => r.to_string_lossy().replace('\\', "/"),
            Err(_) => abs_path.to_string_lossy().replace('\\', "/"),
        };

        let file_name = abs_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        if matcher.is_match(&file_name) || matcher.is_match(&rel) {
            let size = fs::metadata(&abs_path).map(|m| m.len()).unwrap_or(0);
            matched_items.push(FindFileItem {
                path: rel,
                size_bytes: size,
            });
        }
    }

    matched_items.sort_by(|a, b| a.path.cmp(&b.path));

    if args.json {
        let res = json!({
            "command": "find",
            "pattern": glob_pat,
            "total_matches": matched_items.len(),
            "matches": matched_items
        });
        return truncate_to_budget(&res.to_string(), args.budget);
    }

    let scope_display = scope_rel.unwrap_or_else(|| {
        if scan_dir == repo_root {
            "."
        } else {
            scan_dir
                .strip_prefix(repo_root)
                .map(|p| p.to_str().unwrap_or("."))
                .unwrap_or(".")
        }
    });

    let mut out = String::new();
    out.push_str(&format!(
        "### find \"{}\" ({} matches in `{}`)\n",
        glob_pat,
        matched_items.len(),
        scope_display
    ));

    for item in &matched_items {
        out.push_str(&format!(
            "  - `{}` ({})\n",
            item.path,
            format_bytes(item.size_bytes)
        ));
    }

    truncate_to_budget(&out, args.budget)
}
