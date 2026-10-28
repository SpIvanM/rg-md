use crate::cli::{translate_platform_path, CliArgs};
use ignore::WalkBuilder;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

pub const ROOT_MARKERS: &[&str] = &[
    ".git",
    "Cargo.toml",
    "package.json",
    "pyproject.toml",
    "go.mod",
    "setup.py",
    "pom.xml",
    "build.gradle",
    "composer.json",
];

pub const EXCLUDED_DIRS: &[&str] = &[
    ".git",
    ".hg",
    ".svn",
    "node_modules",
    "target",
    "dist",
    "build",
    "out",
    ".next",
    ".nuxt",
    ".venv",
    "venv",
    "env",
    "__pycache__",
    ".pytest_cache",
    ".mypy_cache",
    ".ruff_cache",
    ".tox",
    ".nox",
    ".cache",
    ".tmp",
    "tmp",
    "vendor",
    ".idea",
    ".vscode",
    "coverage",
    "bower_components",
];

pub fn is_stop_dir(p: &Path) -> bool {
    let s = p.to_string_lossy().replace('\\', "/");
    let clean = s
        .strip_prefix("//?/UNC/wsl.localhost/Ubuntu")
        .or_else(|| s.strip_prefix("//wsl.localhost/Ubuntu"))
        .unwrap_or(&s);
    matches!(
        clean,
        "" | "/" | "/tmp" | "/var" | "/var/tmp" | "/home" | "/Users" | "C:/"
    ) || clean.ends_with("/projects")
}

pub fn discover_root_and_scope(
    explicit_path: Option<&Path>,
    targets: &[String],
) -> (PathBuf, Option<String>) {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    let (start_path, was_explicit) = if let Some(p) = explicit_path {
        (
            PathBuf::from(translate_platform_path(&p.to_string_lossy())),
            true,
        )
    } else {
        let mut candidate = None;
        for t in targets {
            let clean = translate_platform_path(t);
            let p = Path::new(&clean);
            if p.exists() || clean.contains('/') || clean.contains('\\') {
                candidate = Some(PathBuf::from(clean));
                break;
            }
        }
        if let Some(c) = candidate {
            (c, false)
        } else {
            (cwd, false)
        }
    };

    let abs_start = fs::canonicalize(&start_path).unwrap_or_else(|_| {
        if start_path.is_absolute() {
            start_path.clone()
        } else {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(&start_path)
        }
    });

    let start_dir = if abs_start.is_file() {
        abs_start.parent().unwrap_or(&abs_start).to_path_buf()
    } else {
        abs_start.clone()
    };

    let mut curr = start_dir.as_path();
    let mut found_root = None;

    loop {
        if is_stop_dir(curr) {
            break;
        }
        for marker in ROOT_MARKERS {
            if curr.join(marker).exists() {
                found_root = Some(curr.to_path_buf());
                break;
            }
        }
        if found_root.is_some() {
            break;
        }
        match curr.parent() {
            Some(parent) if parent != curr && !is_stop_dir(parent) => {
                curr = parent;
            }
            _ => break,
        }
    }

    let repo_root = found_root.unwrap_or_else(|| start_dir.clone());

    let scope_rel = if was_explicit && abs_start != repo_root {
        abs_start
            .strip_prefix(&repo_root)
            .ok()
            .map(|p| {
                p.to_string_lossy()
                    .replace('\\', "/")
                    .trim_start_matches("./")
                    .to_string()
            })
            .filter(|s| !s.is_empty() && s != ".")
    } else {
        None
    };

    (repo_root, scope_rel)
}

pub fn collect_files(root: &Path, include_hidden: bool) -> Vec<PathBuf> {
    let mut builder = WalkBuilder::new(root);
    builder
        .hidden(!include_hidden)
        .parents(true)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .filter_entry(|entry| {
            if entry.file_type().is_some_and(|ft| ft.is_dir()) {
                if let Some(name) = entry.file_name().to_str() {
                    if EXCLUDED_DIRS.contains(&name) {
                        return false;
                    }
                }
            }
            true
        });

    let mut files = Vec::new();
    for entry in builder.build().flatten() {
        if entry.file_type().is_some_and(|ft| ft.is_file()) {
            files.push(entry.into_path());
        }
    }
    files.sort();
    files
}

pub fn execute_map(repo_root: &Path, scope_rel: Option<&str>, args: &CliArgs) -> String {
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

    let files = collect_files(&scan_dir, args.hidden);
    let mut rel_files = Vec::new();
    for f in &files {
        let rel = f
            .strip_prefix(repo_root)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| f.to_string_lossy().replace('\\', "/"));
        rel_files.push(rel);
    }

    if args.json {
        let val = json!({
            "command": "map",
            "root": repo_root.to_string_lossy().replace('\\', "/"),
            "scope": scope_rel,
            "total_files": rel_files.len(),
            "files": rel_files,
        });
        return serde_json::to_string_pretty(&val).unwrap_or_default();
    }

    let mut out = String::new();
    let display_root = repo_root.to_string_lossy().replace('\\', "/");
    out.push_str(&format!(
        "### `{}` ({} files)\n",
        display_root,
        rel_files.len()
    ));

    let limit = args.budget.effective_char_limit();
    for file in &rel_files {
        let line = format!("  - `{}`\n", file);
        if out.len() + line.len() > limit {
            out.push_str(&crate::formatter::truncation_notice(args.budget));
            break;
        }
        out.push_str(&line);
    }

    out
}
