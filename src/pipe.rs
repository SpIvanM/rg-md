use crate::cli::CliArgs;
use crate::diff::format_git_diff;
use crate::exec::filter_compiler_or_test_output;
use crate::formatter::truncate_to_budget;
use std::io::{self, Read};

pub fn execute_pipe(args: &CliArgs) -> String {
    let buffer = if !args.query.is_empty() {
        args.query.clone()
    } else if !args.targets.is_empty() {
        args.targets.join("\n")
    } else {
        let mut b = String::new();
        let _ = io::stdin().read_to_string(&mut b);
        b
    };

    if buffer.trim().is_empty() {
        return String::from("No input received on stdin.");
    }

    // Auto-detect diff or compiler/test output
    if buffer.starts_with("diff --git ") || buffer.contains("\n@@ ") {
        format_git_diff(&buffer, args)
    } else if buffer.contains("error[") || buffer.contains("FAILED") || buffer.contains("error TS") {
        filter_compiler_or_test_output(&buffer, 1, args)
    } else {
        // Generic plain text Markdown AST wrapper
        let mut out = String::from("### pipe\n");
        for (idx, line) in buffer.lines().enumerate() {
            if !line.trim().is_empty() {
                out.push_str(&format!("  - L{}: {}\n", idx + 1, line.trim()));
            }
        }
        truncate_to_budget(out.trim_end(), args.budget)
    }
}
