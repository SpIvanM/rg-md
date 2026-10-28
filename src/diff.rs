use crate::cli::CliArgs;
use crate::formatter::truncate_to_budget;
use serde_json::json;
use similar::{ChangeTag, TextDiff};
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn execute_diff(repo_root: &Path, args: &CliArgs) -> String {
    if args.targets.len() >= 2 {
        let p1 = if Path::new(&args.targets[0]).is_absolute() {
            std::path::PathBuf::from(&args.targets[0])
        } else {
            repo_root.join(&args.targets[0])
        };
        let p2 = if Path::new(&args.targets[1]).is_absolute() {
            std::path::PathBuf::from(&args.targets[1])
        } else {
            repo_root.join(&args.targets[1])
        };

        let text1 = match fs::read_to_string(&p1) {
            Ok(t) => t,
            Err(e) => return format!("Error reading {}: {}", p1.display(), e),
        };
        let text2 = match fs::read_to_string(&p2) {
            Ok(t) => t,
            Err(e) => return format!("Error reading {}: {}", p2.display(), e),
        };

        return format_two_files_diff(&args.targets[0], &text1, &args.targets[1], &text2, args);
    }

    if args.targets.len() == 1 {
        let t = &args.targets[0];
        if t.contains("--- ") || t.contains("@@ ") || t.contains("diff --git") {
            return format_git_diff(t, args);
        } else if let Ok(content) = fs::read_to_string(repo_root.join(t)) {
            if content.contains("--- ") || content.contains("@@ ") || content.contains("diff --git") {
                return format_git_diff(&content, args);
            }
        }
    }

    if !args.query.is_empty() && (args.query.contains("--- ") || args.query.contains("@@ ")) {
        return format_git_diff(&args.query, args);
    }

    // Otherwise, execute git diff in repo_root
    let git_output = Command::new("git")
        .arg("diff")
        .current_dir(repo_root)
        .output();

    match git_output {
        Ok(out) if !out.stdout.is_empty() => {
            let raw_diff = String::from_utf8_lossy(&out.stdout);
            format_git_diff(&raw_diff, args)
        }
        _ => String::from("No diff changes detected."),
    }
}

pub fn format_two_files_diff(
    name1: &str,
    text1: &str,
    name2: &str,
    text2: &str,
    args: &CliArgs,
) -> String {
    let diff = TextDiff::from_lines(text1, text2);
    let mut added = 0;
    let mut removed = 0;

    let mut lines_out = Vec::new();
    let mut l1 = 1;
    let mut l2 = 1;

    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Delete => {
                removed += 1;
                lines_out.push(format!("  - L{}: - {}", l1, change.value().trim_end()));
                l1 += 1;
            }
            ChangeTag::Insert => {
                added += 1;
                lines_out.push(format!("  - L{}: + {}", l2, change.value().trim_end()));
                l2 += 1;
            }
            ChangeTag::Equal => {
                l1 += 1;
                l2 += 1;
            }
        }
    }

    if args.json {
        let res = json!({
            "command": "diff",
            "file1": name1,
            "file2": name2,
            "added": added,
            "removed": removed,
            "changes": lines_out
        });
        return truncate_to_budget(&res.to_string(), args.budget);
    }

    let mut out = String::new();
    out.push_str(&format!(
        "### `{}` -> `{}` (+{} -{})\n",
        name1, name2, added, removed
    ));
    for line in lines_out {
        out.push_str(&line);
        out.push('\n');
    }

    truncate_to_budget(out.trim_end(), args.budget)
}

pub fn format_git_diff(raw_diff: &str, args: &CliArgs) -> String {
    let mut out = String::new();
    let mut current_file: Option<String> = None;
    let mut current_lines: Vec<String> = Vec::new();
    let mut added = 0;
    let mut removed = 0;
    let mut old_line = 0;
    let mut new_line = 0;

    for line in raw_diff.lines() {
        if line.starts_with("diff --git ") {
            if let Some(f) = current_file.take() {
                out.push_str(&format!("### `{}` (+{} -{})\n", f, added, removed));
                for l in &current_lines {
                    out.push_str(l);
                    out.push('\n');
                }
                out.push('\n');
                current_lines.clear();
                added = 0;
                removed = 0;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let fname = parts[3].trim_start_matches("b/");
                current_file = Some(fname.to_string());
            }
        } else if line.starts_with("@@ ") {
            // Parse @@ -A,B +C,D @@
            if let Some(plus_idx) = line.find('+') {
                let rest = &line[plus_idx + 1..];
                let comma_or_space = rest.find([',', ' ']).unwrap_or(rest.len());
                new_line = rest[..comma_or_space].parse::<usize>().unwrap_or(1);
            }
            if let Some(minus_idx) = line.find('-') {
                let rest = &line[minus_idx + 1..];
                let comma_or_space = rest.find([',', ' ']).unwrap_or(rest.len());
                old_line = rest[..comma_or_space].parse::<usize>().unwrap_or(1);
            }
        } else if line.starts_with('+') && !line.starts_with("+++") {
            added += 1;
            current_lines.push(format!("  - L{}: + {}", new_line, &line[1..].trim_end()));
            new_line += 1;
        } else if line.starts_with('-') && !line.starts_with("---") {
            removed += 1;
            current_lines.push(format!("  - L{}: - {}", old_line, &line[1..].trim_end()));
            old_line += 1;
        } else if !line.starts_with("index ")
            && !line.starts_with("---")
            && !line.starts_with("+++")
        {
            old_line += 1;
            new_line += 1;
        }
    }

    if let Some(f) = current_file {
        out.push_str(&format!("### `{}` (+{} -{})\n", f, added, removed));
        for l in current_lines {
            out.push_str(&l);
            out.push('\n');
        }
    }

    truncate_to_budget(out.trim_end(), args.budget)
}
