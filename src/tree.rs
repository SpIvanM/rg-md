use crate::cli::CliArgs;
use crate::formatter::{format_bytes, truncate_to_budget};
use crate::scanner::collect_files;
use serde::Serialize;
use serde_json::json;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct TreeFileItem {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TreeDirGroup {
    pub dir: String,
    pub total_files: usize,
    pub total_size_bytes: u64,
    pub files: Vec<TreeFileItem>,
}

pub fn execute_tree(repo_root: &Path, scope_rel: Option<&str>, args: &CliArgs) -> String {
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
    let mut dir_map: BTreeMap<String, Vec<TreeFileItem>> = BTreeMap::new();

    for abs_path in all_files {
        let rel_from_scan = match abs_path.strip_prefix(&scan_dir) {
            Ok(r) => r.to_string_lossy().replace('\\', "/"),
            Err(_) => abs_path.to_string_lossy().replace('\\', "/"),
        };
        let depth = rel_from_scan.split('/').count().saturating_sub(1);
        if let Some(max_d) = args.depth {
            if depth > max_d {
                continue;
            }
        }

        let rel_path = match abs_path.strip_prefix(repo_root) {
            Ok(r) => r.to_string_lossy().replace('\\', "/"),
            Err(_) => abs_path.to_string_lossy().replace('\\', "/"),
        };

        let parent_dir = if let Some(idx) = rel_path.rfind('/') {
            format!("{}/", &rel_path[..idx])
        } else {
            String::from("./")
        };

        let file_name = abs_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| rel_path.clone());

        let size = fs::metadata(&abs_path).map(|m| m.len()).unwrap_or(0);

        dir_map.entry(parent_dir).or_default().push(TreeFileItem {
            name: file_name,
            path: rel_path,
            size_bytes: size,
        });
    }

    let mut groups: Vec<TreeDirGroup> = Vec::new();
    for (dir, mut files) in dir_map {
        files.sort_by(|a, b| a.name.cmp(&b.name));
        let total_size = files.iter().map(|f| f.size_bytes).sum();
        let count = files.len();
        groups.push(TreeDirGroup {
            dir,
            total_files: count,
            total_size_bytes: total_size,
            files,
        });
    }

    if args.json {
        let res = json!({
            "command": "tree",
            "total_directories": groups.len(),
            "groups": groups
        });
        return truncate_to_budget(&res.to_string(), args.budget);
    }

    let mut out = String::new();
    for group in &groups {
        out.push_str(&format!(
            "### `{}` ({} files, {})\n",
            group.dir,
            group.total_files,
            format_bytes(group.total_size_bytes)
        ));
        for file in &group.files {
            out.push_str(&format!(
                "  - `{}` ({})\n",
                file.name,
                format_bytes(file.size_bytes)
            ));
        }
        out.push('\n');
    }

    truncate_to_budget(out.trim_end(), args.budget)
}
