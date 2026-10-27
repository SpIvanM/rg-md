use crate::cli::CliArgs;
use crate::formatter::format_search_results;
use crate::scanner::collect_files;
use rayon::prelude::*;
use regex::RegexBuilder;
use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct LineMatch {
    pub line: usize,
    pub content: String,
    pub is_match: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileSearchResult {
    pub path: String,
    pub matches: Vec<LineMatch>,
}

pub fn format_display_path(
    file_path: &Path,
    repo_root: &Path,
    explicit_path: Option<&Path>,
) -> String {
    if let Some(exp) = explicit_path {
        if exp.is_absolute() {
            return file_path.to_string_lossy().replace('\\', "/");
        }
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if let Ok(rel) = file_path.strip_prefix(&cwd) {
        let s = rel.to_string_lossy().replace('\\', "/");
        if !s.is_empty() {
            return s;
        }
    }
    if let Ok(rel) = file_path.strip_prefix(repo_root) {
        let s = rel.to_string_lossy().replace('\\', "/");
        if !s.is_empty() {
            return s;
        }
    }
    file_path.to_string_lossy().replace('\\', "/")
}

fn is_binary_file(bytes: &[u8]) -> bool {
    let check_len = bytes.len().min(1024);
    bytes[..check_len].contains(&0x00)
}

fn search_single_file(
    file_path: &Path,
    repo_root: &Path,
    explicit_path: Option<&Path>,
    re: &regex::Regex,
    context: usize,
) -> Option<FileSearchResult> {
    let bytes = fs::read(file_path).ok()?;
    if is_binary_file(&bytes) {
        return None;
    }

    let text = String::from_utf8_lossy(&bytes);
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return None;
    }

    let mut match_line_indices: Vec<usize> = Vec::new();
    for (idx, line) in lines.iter().enumerate() {
        if re.is_match(line) {
            match_line_indices.push(idx);
        }
    }

    if match_line_indices.is_empty() {
        return None;
    }

    let mut line_matches: Vec<LineMatch> = Vec::new();

    if context == 0 {
        for idx in match_line_indices {
            line_matches.push(LineMatch {
                line: idx + 1,
                content: lines[idx].trim().to_string(),
                is_match: true,
            });
        }
    } else {
        let total = lines.len();
        let match_set: HashSet<usize> = match_line_indices.iter().copied().collect();

        // Build intervals [start, end]
        let mut intervals: Vec<(usize, usize)> = Vec::new();
        for &m in &match_line_indices {
            let start = m.saturating_sub(context);
            let end = (m + context).min(total.saturating_sub(1));
            intervals.push((start, end));
        }

        // Merge overlapping intervals
        let mut merged: Vec<(usize, usize)> = Vec::new();
        for (start, end) in intervals {
            if let Some(last) = merged.last_mut() {
                if start <= last.1 + 1 {
                    last.1 = last.1.max(end);
                    continue;
                }
            }
            merged.push((start, end));
        }

        for (start, end) in merged {
            for (idx, line) in lines.iter().enumerate().take(end + 1).skip(start) {
                line_matches.push(LineMatch {
                    line: idx + 1,
                    content: line.trim().to_string(),
                    is_match: match_set.contains(&idx),
                });
            }
        }
    }

    let display_path = format_display_path(file_path, repo_root, explicit_path);

    Some(FileSearchResult {
        path: display_path,
        matches: line_matches,
    })
}

pub fn execute_search(repo_root: &Path, scope_rel: Option<&str>, args: &CliArgs) -> String {
    if args.query.is_empty() {
        return "Usage: rg-md [flags] <pattern> [path]\n".to_string();
    }

    let re = match RegexBuilder::new(&args.query)
        .case_insensitive(args.ignore_case)
        .build()
    {
        Ok(r) => r,
        Err(_) => {
            let escaped = regex::escape(&args.query);
            match RegexBuilder::new(&escaped)
                .case_insensitive(args.ignore_case)
                .build()
            {
                Ok(r) => r,
                Err(e) => return format!("Error compiling regex: {}\n", e),
            }
        }
    };

    let target_path = if let Some(scope) = scope_rel {
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

    let files = if target_path.is_file() {
        vec![target_path]
    } else {
        collect_files(&target_path, args.hidden)
    };

    let explicit_ref = args.path.as_deref();

    let mut results: Vec<FileSearchResult> = files
        .par_iter()
        .filter_map(|fp| search_single_file(fp, repo_root, explicit_ref, &re, args.context))
        .collect();

    results.sort_by(|a, b| a.path.cmp(&b.path));

    format_search_results(&results, args)
}
