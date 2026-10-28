use crate::cli::CliArgs;
use crate::formatter::truncate_to_budget;
use regex::Regex;
use serde_json::json;
use std::process::Command;

pub fn execute_err_or_test(args: &CliArgs) -> String {
    let raw_text = if !args.query.is_empty() && (args.query.contains('\n') || args.query.contains("error[") || args.query.contains("FAILED")) {
        Some(args.query.clone())
    } else if !args.targets.is_empty() {
        if args.targets[0].contains('\n') || args.targets[0].contains("error[") || args.targets[0].contains("FAILED") {
            Some(args.targets.join("\n"))
        } else if let Ok(content) = std::fs::read_to_string(&args.targets[0]) {
            Some(content)
        } else {
            None
        }
    } else {
        None
    };

    if let Some(raw) = raw_text {
        return filter_compiler_or_test_output(&raw, 1, args);
    }

    if args.targets.is_empty() {
        return String::from("No command specified to run.");
    }

    let program = &args.targets[0];
    let prog_args = &args.targets[1..];

    let output = Command::new(program)
        .args(prog_args)
        .current_dir(args.path.as_deref().unwrap_or_else(|| std::path::Path::new(".")))
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            let combined = format!("{}\n{}", stdout, stderr);

            let status_code = out.status.code().unwrap_or(if out.status.success() { 0 } else { 1 });
            filter_compiler_or_test_output(&combined, status_code, args)
        }
        Err(e) => format!("Failed to execute '{}': {}", program, e),
    }
}

pub fn filter_compiler_or_test_output(raw: &str, status_code: i32, args: &CliArgs) -> String {
    let mut errors: Vec<String> = Vec::new();
    let mut failures: Vec<String> = Vec::new();

    let re_cargo_err = Regex::new(r"error(\[[^\]]+\])?: (.*)").unwrap();
    let re_tsc_err = Regex::new(r"^(.*?):(\d+):(\d+) - error (TS\d+): (.*)").unwrap();
    let re_test_fail = Regex::new(r"(FAILED|test [^\s]+ \.\.\. FAILED|FAIL [^\s]+)").unwrap();

    let mut in_failure_block = false;
    let mut block_lines: Vec<String> = Vec::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if in_failure_block && !block_lines.is_empty() {
                failures.push(block_lines.join("\n"));
                block_lines.clear();
                in_failure_block = false;
            }
            continue;
        }

        if let Some(caps) = re_cargo_err.captures(trimmed) {
            let code = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let msg = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            errors.push(format!("### Error {} \n  - {}", code, msg));
        } else if let Some(caps) = re_tsc_err.captures(trimmed) {
            let file = &caps[1];
            let l = &caps[2];
            let c = &caps[3];
            let code = &caps[4];
            let msg = &caps[5];
            errors.push(format!("### Error: `{}:{}:{}` [{}]\n  - {}", file, l, c, code, msg));
        } else if re_test_fail.is_match(trimmed) || trimmed.starts_with("assertion failed") {
            in_failure_block = true;
            block_lines.push(format!("  - {}", trimmed));
        } else if in_failure_block {
            block_lines.push(format!("    {}", trimmed));
            if block_lines.len() > 6 {
                failures.push(block_lines.join("\n"));
                block_lines.clear();
                in_failure_block = false;
            }
        }
    }

    if in_failure_block && !block_lines.is_empty() {
        failures.push(block_lines.join("\n"));
    }

    if args.json {
        let res = json!({
            "status_code": status_code,
            "errors_count": errors.len(),
            "failures_count": failures.len(),
            "errors": errors,
            "failures": failures
        });
        return truncate_to_budget(&res.to_string(), args.budget);
    }

    let mut out = String::new();

    if errors.is_empty() && failures.is_empty() {
        if status_code == 0 {
            return String::from("### Command Succeeded (0 errors, 0 failures)\n");
        } else {
            out.push_str(&format!("### Command Failed with exit code {}\n", status_code));
            for l in raw.lines().rev().take(10).collect::<Vec<_>>().into_iter().rev() {
                if !l.trim().is_empty() {
                    out.push_str(&format!("  - {}\n", l.trim()));
                }
            }
            return truncate_to_budget(out.trim_end(), args.budget);
        }
    }

    for err in errors {
        out.push_str(&err);
        out.push('\n');
    }

    if !failures.is_empty() {
        out.push_str(&format!("\n### Test Failures ({} failed)\n", failures.len()));
        for fail in failures {
            out.push_str(&fail);
            out.push('\n');
        }
    }

    truncate_to_budget(out.trim_end(), args.budget)
}
