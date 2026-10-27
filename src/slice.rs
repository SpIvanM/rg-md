use crate::cli::CliArgs;
use crate::formatter::format_slice_results;
use crate::scanner::collect_files;
use crate::search::format_display_path;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct SliceLine {
    pub line: usize,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SliceResult {
    pub target: String,
    pub file_path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub symbol: Option<String>,
    pub lines: Vec<SliceLine>,
}

pub fn parse_line_range(spec: &str) -> Option<(usize, usize)> {
    let clean = spec.trim().trim_start_matches('L').trim_start_matches('l');
    if let Some((s, e)) = clean.split_once('-') {
        let start = s
            .trim()
            .trim_start_matches('L')
            .trim_start_matches('l')
            .parse::<usize>()
            .ok()?;
        let end = e
            .trim()
            .trim_start_matches('L')
            .trim_start_matches('l')
            .parse::<usize>()
            .ok()?;
        Some((start.max(1), end.max(start)))
    } else if let Some((s, len)) = clean.split_once('+') {
        let start = s
            .trim()
            .trim_start_matches('L')
            .trim_start_matches('l')
            .parse::<usize>()
            .ok()?;
        let count = len.trim().parse::<usize>().ok()?;
        Some((start.max(1), start.saturating_add(count)))
    } else if let Ok(line) = clean.parse::<usize>() {
        Some((line.saturating_sub(10).max(1), line.saturating_add(10)))
    } else {
        None
    }
}

pub fn find_candidate_file(
    repo_root: &Path,
    scope_rel: Option<&str>,
    explicit_path: Option<&Path>,
    file_spec: &str,
) -> Option<PathBuf> {
    let direct = PathBuf::from(file_spec);
    if direct.is_file() {
        return Some(direct);
    }

    if let Some(scope) = scope_rel {
        let scoped = repo_root.join(scope).join(file_spec);
        if scoped.is_file() {
            return Some(scoped);
        }
    }

    if let Some(exp) = explicit_path {
        let candidate = exp.join(file_spec);
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    let root_cand = repo_root.join(file_spec);
    if root_cand.is_file() {
        return Some(root_cand);
    }

    // Suffix match on collected files
    let norm_spec = file_spec.replace('\\', "/");
    let norm_lower = norm_spec.to_ascii_lowercase();
    let all_files = collect_files(repo_root, false);

    for f in all_files {
        let rel = f
            .strip_prefix(repo_root)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| f.to_string_lossy().replace('\\', "/"));
        let rel_lower = rel.to_ascii_lowercase();
        if rel_lower == norm_lower || rel_lower.ends_with(&format!("/{}", norm_lower)) {
            return Some(f);
        }
    }

    None
}

fn locate_symbol_in_lines(lines: &[&str], symbol: &str) -> Option<usize> {
    let sym_lower = symbol.to_ascii_lowercase();

    // Prefer definition patterns
    let def_keywords = [
        "fn ",
        "function ",
        "const ",
        "class ",
        "let ",
        "struct ",
        "enum ",
        "type ",
        "def ",
        "pub fn ",
        "interface ",
        "trait ",
    ];

    for (idx, line) in lines.iter().enumerate() {
        let l_lower = line.to_ascii_lowercase();
        for kw in def_keywords {
            if l_lower.contains(kw) && l_lower.contains(&sym_lower) {
                return Some(idx);
            }
        }
    }

    // Fallback: any occurrence
    for (idx, line) in lines.iter().enumerate() {
        if line.to_ascii_lowercase().contains(&sym_lower) {
            return Some(idx);
        }
    }

    None
}

pub fn execute_slice(repo_root: &Path, scope_rel: Option<&str>, args: &CliArgs) -> String {
    let mut targets_to_process = args.targets.clone();
    if targets_to_process.is_empty() && !args.query.is_empty() {
        targets_to_process.push(args.query.clone());
    }

    if targets_to_process.is_empty() {
        return "Usage: rg-md slice <target...> [path]\nTargets: file:start-end, file:line, file::symbol, or symbol\n".to_string();
    }

    let mut results: Vec<SliceResult> = Vec::new();
    let explicit_ref = args.path.as_deref();

    for target in &targets_to_process {
        let mut file_spec = target.as_str();
        let mut range_spec: Option<&str> = None;
        let mut symbol_spec: Option<&str> = None;

        if let Some((f, s)) = target.split_once("::") {
            file_spec = f;
            symbol_spec = Some(s);
        } else if let Some((f, r)) = target.rsplit_once(':') {
            if let Some(_range) = parse_line_range(r) {
                file_spec = f;
                range_spec = Some(r);
            }
        }

        // Try locating file
        let resolved_file = find_candidate_file(repo_root, scope_rel, explicit_ref, file_spec);

        if let Some(file_path) = resolved_file {
            if let Ok(bytes) = fs::read(&file_path) {
                let text = String::from_utf8_lossy(&bytes);
                let lines: Vec<&str> = text.lines().collect();
                let total = lines.len();

                let (start_l, end_l, sym) = if let Some(sym_name) = symbol_spec {
                    if let Some(idx) = locate_symbol_in_lines(&lines, sym_name) {
                        let s = (idx + 1).saturating_sub(2).max(1);
                        let e = (idx + 35).min(total);
                        (s, e, Some(sym_name.to_string()))
                    } else {
                        (1, total.min(50), Some(sym_name.to_string()))
                    }
                } else if let Some(r) = range_spec {
                    let (s, e) = parse_line_range(r).unwrap_or((1, total));
                    (
                        s.min(total.max(1)),
                        e.min(total).max(s.min(total.max(1))),
                        None,
                    )
                } else {
                    (1, total, None)
                };

                let mut slice_lines = Vec::new();
                if total > 0 && start_l <= total {
                    for lnum in start_l..=end_l.min(total) {
                        slice_lines.push(SliceLine {
                            line: lnum,
                            content: lines[lnum - 1].trim().to_string(),
                        });
                    }
                }

                let display_p = format_display_path(&file_path, repo_root, explicit_ref);

                results.push(SliceResult {
                    target: target.clone(),
                    file_path: display_p,
                    start_line: start_l,
                    end_line: end_l.min(total),
                    symbol: sym,
                    lines: slice_lines,
                });
            }
        } else {
            // Target might be a symbol name alone (e.g. `RecordingOverlay`)
            let sym_name = target;
            let all_files = collect_files(repo_root, false);
            let mut found_sym = false;

            for f in all_files {
                if let Ok(bytes) = fs::read(&f) {
                    let text = String::from_utf8_lossy(&bytes);
                    let lines: Vec<&str> = text.lines().collect();
                    if let Some(idx) = locate_symbol_in_lines(&lines, sym_name) {
                        let total = lines.len();
                        let s = (idx + 1).saturating_sub(2).max(1);
                        let e = (idx + 35).min(total);

                        let mut slice_lines = Vec::new();
                        for lnum in s..=e {
                            slice_lines.push(SliceLine {
                                line: lnum,
                                content: lines[lnum - 1].trim().to_string(),
                            });
                        }

                        let display_p = format_display_path(&f, repo_root, explicit_ref);
                        results.push(SliceResult {
                            target: target.clone(),
                            file_path: display_p,
                            start_line: s,
                            end_line: e,
                            symbol: Some(sym_name.to_string()),
                            lines: slice_lines,
                        });
                        found_sym = true;
                        break;
                    }
                }
            }

            if !found_sym {
                results.push(SliceResult {
                    target: target.clone(),
                    file_path: target.clone(),
                    start_line: 0,
                    end_line: 0,
                    symbol: Some(sym_name.to_string()),
                    lines: Vec::new(),
                });
            }
        }
    }

    format_slice_results(&results, args)
}
