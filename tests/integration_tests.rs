use rg_md::cli::{parse_args, translate_platform_path, CommandKind};
use rg_md::run_from_args;
use rg_md::scanner::discover_root_and_scope;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn create_temp_workspace(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp_dir = std::env::temp_dir().join(format!("rg_md_test_{}_{}", prefix, nanos));
    fs::create_dir_all(&temp_dir).unwrap();

    // Standard root marker
    fs::write(
        temp_dir.join("Cargo.toml"),
        "[package]\nname = \"sample\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();

    // Source files
    let src_dir = temp_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();

    fs::write(
        src_dir.join("main.rs"),
        "fn main() {\n    println!(\"Hello World\");\n}\n\npub fn calculate_score(val: i32) -> i32 {\n    val * 2\n}\n",
    )
    .unwrap();

    fs::write(
        src_dir.join("lib.rs"),
        "pub mod utils;\npub fn helper() -> bool {\n    true\n}\n",
    )
    .unwrap();

    let tests_dir = temp_dir.join("tests");
    fs::create_dir_all(&tests_dir).unwrap();
    fs::write(
        tests_dir.join("test_app.rs"),
        "#[test]\nfn test_score() {\n    assert_eq!(2 + 2, 4);\n}\n",
    )
    .unwrap();

    temp_dir
}

#[test]
fn test_search_markdown_formatting() {
    let ws = create_temp_workspace("search");

    // 1. Basic search with markdown header and L-prefixes
    let out = run_from_args(&["calculate_score", ws.to_str().unwrap()]);
    assert!(
        out.contains("### `") && out.contains("src/main.rs`"),
        "Output should contain markdown header: {}",
        out
    );
    assert!(
        out.contains("  - L5: pub fn calculate_score(val: i32) -> i32 {"),
        "Output should contain formatted L-prefix: {}",
        out
    );

    // 2. Explicit search command with regex
    let out_regex = run_from_args(&["search", r"fn calculate.*score", ws.to_str().unwrap()]);
    assert!(
        out_regex.contains("### `") && out_regex.contains("src/main.rs`"),
        "Regex search should find main.rs: {}",
        out_regex
    );
    assert!(
        out_regex.contains("  - L5:"),
        "Regex match should include line number: {}",
        out_regex
    );

    // 3. Case-insensitive search (-i)
    let out_case = run_from_args(&["-i", "CALCULATE_SCORE", ws.to_str().unwrap()]);
    assert!(
        out_case.contains("calculate_score"),
        "Case-insensitive search should match: {}",
        out_case
    );

    // 4. Context lines (-C 1)
    let out_ctx = run_from_args(&["-C", "1", "calculate_score", ws.to_str().unwrap()]);
    assert!(
        out_ctx.contains("  - L5:"),
        "Target match should be present: {}",
        out_ctx
    );

    let _ = fs::remove_dir_all(ws);
}

#[test]
fn test_find_file_discovery() {
    let ws = create_temp_workspace("find");
    let comp_dir = ws.join("src").join("components");
    fs::create_dir_all(&comp_dir).unwrap();
    fs::write(
        comp_dir.join("RecordingOverlay.tsx"),
        "export const RecordingOverlay = () => <div>Recording...</div>;\n",
    )
    .unwrap();

    // 1. Find Rust files (*.rs)
    let out_rs = run_from_args(&["find", "*.rs", ws.to_str().unwrap()]);
    assert!(
        out_rs.contains("### find \"*.rs\""),
        "Header must match find glob format: {}",
        out_rs
    );
    assert!(
        out_rs.contains("src/main.rs"),
        "Must discover main.rs: {}",
        out_rs
    );
    assert!(
        out_rs.contains("src/lib.rs"),
        "Must discover lib.rs: {}",
        out_rs
    );
    assert!(
        out_rs.contains("tests/test_app.rs"),
        "Must discover test_app.rs: {}",
        out_rs
    );
    assert!(
        !out_rs.contains("RecordingOverlay.tsx"),
        "Must not match tsx files: {}",
        out_rs
    );
    // Byte sizes verified
    assert!(
        out_rs.contains(" B)") || out_rs.contains(" KB)"),
        "Must include formatted byte sizes: {}",
        out_rs
    );

    // 2. Find TypeScript files (*.tsx)
    let out_tsx = run_from_args(&["find", "*.tsx", ws.to_str().unwrap()]);
    assert!(
        out_tsx.contains("### find \"*.tsx\""),
        "Header must match find glob format: {}",
        out_tsx
    );
    assert!(
        out_tsx.contains("RecordingOverlay.tsx"),
        "Must discover tsx file: {}",
        out_tsx
    );
    assert!(
        !out_tsx.contains("src/main.rs"),
        "Must not match rs files: {}",
        out_tsx
    );
    assert!(
        out_tsx.contains(" B)") || out_tsx.contains(" KB)"),
        "Must include formatted byte size: {}",
        out_tsx
    );

    let _ = fs::remove_dir_all(ws);
}

#[test]
fn test_tree_directory_topology() {
    let ws = create_temp_workspace("tree");
    let deep_dir = ws.join("src").join("nested").join("deep");
    fs::create_dir_all(&deep_dir).unwrap();
    fs::write(deep_dir.join("deep_module.rs"), "// deep nested module\n").unwrap();

    // 1. Full tree directory topology
    let out = run_from_args(&["tree", ws.to_str().unwrap()]);
    assert!(
        out.contains("### `src/`") || out.contains("### src/"),
        "Should contain grouped directory header: {}",
        out
    );
    assert!(
        out.contains("files,") || out.contains("file,"),
        "Header must show file count and byte size: {}",
        out
    );
    assert!(
        out.contains("main.rs"),
        "Should show file items under directory: {}",
        out
    );

    // 2. Tree with depth limit (--depth 1)
    let out_depth1 = run_from_args(&["tree", ws.to_str().unwrap(), "--depth=1"]);
    assert!(
        out_depth1.contains("src/"),
        "Depth 1 should list top level folders: {}",
        out_depth1
    );
    assert!(
        !out_depth1.contains("deep_module.rs"),
        "Depth 1 should constrain deeper directory topology: {}",
        out_depth1
    );

    let _ = fs::remove_dir_all(ws);
}

#[test]
fn test_diff_formatting() {
    let ws = create_temp_workspace("diff");
    let f1 = ws.join("service_v1.rs");
    let f2 = ws.join("service_v2.rs");

    fs::write(
        &f1,
        "fn start() {\n    init();\n    run();\n}\n",
    )
    .unwrap();

    fs::write(
        &f2,
        "fn start() {\n    setup();\n    init();\n    run();\n    cleanup();\n}\n",
    )
    .unwrap();

    // 1. Two files diff
    let out_two = run_from_args(&[
        "diff",
        f1.to_str().unwrap(),
        f2.to_str().unwrap(),
        "-p",
        ws.to_str().unwrap(),
    ]);

    assert!(
        out_two.contains("###") && (out_two.contains("(+2 -0)") || out_two.contains("(+")),
        "Should contain formatted diff header: {}",
        out_two
    );
    assert!(
        out_two.contains("+") && out_two.contains("setup();"),
        "Should contain added lines with +: {}",
        out_two
    );
    assert!(
        out_two.contains("+") && out_two.contains("cleanup();"),
        "Should contain second added line: {}",
        out_two
    );

    // 2. Git diff ingestion
    let git_patch = r#"--- a/src/audio.rs
+++ b/src/audio.rs
@@ -10,3 +10,4 @@
 fn stop_recording() {
-    recorder.abort();
+    recorder.flush();
+    recorder.stop();
 }
"#;

    let out_git = run_from_args(&["diff", git_patch]);
    assert!(
        out_git.contains("###") && out_git.contains("audio.rs"),
        "Git diff header must contain audio.rs: {}",
        out_git
    );
    assert!(
        out_git.contains("(+2 -1)"),
        "Diff header must specify (+2 -1): {}",
        out_git
    );
    assert!(
        out_git.contains("+ recorder.flush();") || out_git.contains("+ recorder.stop();"),
        "Added lines should have +: {}",
        out_git
    );
    assert!(
        out_git.contains("- recorder.abort();"),
        "Removed lines should have -: {}",
        out_git
    );

    let _ = fs::remove_dir_all(ws);
}

#[test]
fn test_slice_advanced() {
    let ws = create_temp_workspace("slice");
    let lines_file = ws.join("lines.txt");
    let mut content = String::new();
    for i in 1..=30 {
        content.push_str(&format!("Line content number {}\n", i));
    }
    fs::write(&lines_file, &content).unwrap();

    // 1. Line range (file:5-10)
    let range_target = format!("{}:5-10", lines_file.display());
    let out_range = run_from_args(&["slice", &range_target]);
    assert!(
        out_range.contains("L5: Line content number 5"),
        "Should include line 5: {}",
        out_range
    );
    assert!(
        out_range.contains("L10: Line content number 10"),
        "Should include line 10: {}",
        out_range
    );
    assert!(
        !out_range.contains("L4:"),
        "Should not include line 4: {}",
        out_range
    );
    assert!(
        !out_range.contains("L11:"),
        "Should not include line 11: {}",
        out_range
    );

    // 2. --head (first N lines)
    let out_head = run_from_args(&["slice", lines_file.to_str().unwrap(), "--head=4"]);
    assert!(
        out_head.contains("L1: Line content number 1"),
        "Head should include line 1: {}",
        out_head
    );
    assert!(
        out_head.contains("L4: Line content number 4"),
        "Head should include line 4: {}",
        out_head
    );
    assert!(
        !out_head.contains("L5:"),
        "Head should not include line 5: {}",
        out_head
    );

    // 3. --tail (last N lines)
    let out_tail = run_from_args(&["slice", lines_file.to_str().unwrap(), "--tail=3"]);
    assert!(
        out_tail.contains("L28: Line content number 28"),
        "Tail should include line 28: {}",
        out_tail
    );
    assert!(
        out_tail.contains("L30: Line content number 30"),
        "Tail should include line 30: {}",
        out_tail
    );
    assert!(
        !out_tail.contains("L27:"),
        "Tail should not include line 27: {}",
        out_tail
    );

    // 4. Center window (file:15)
    let center_target = format!("{}:15", lines_file.display());
    let out_center = run_from_args(&["slice", &center_target]);
    assert!(
        out_center.contains("L15: Line content number 15"),
        "Center window must contain line 15: {}",
        out_center
    );

    let _ = fs::remove_dir_all(ws);
}

#[test]
fn test_pipe_stdin_ingestion() {
    let raw_input = "alpha item\nbeta item\ngamma item\ndelta item\n";
    let out = run_from_args(&["pipe", raw_input]);

    assert!(
        out.contains("### pipe") || out.contains("### Standard Input Stream"),
        "Output must have Markdown AST header: {}",
        out
    );
    assert!(
        out.contains("alpha item"),
        "Must format alpha item: {}",
        out
    );
    assert!(
        out.contains("delta item"),
        "Must format delta item: {}",
        out
    );
    assert!(
        out.contains("  - L1:") || out.contains("  - alpha"),
        "Must format items as Markdown bullet points: {}",
        out
    );
}

#[test]
fn test_pipe_raw_ingestion() {
    let mut args = parse_args(vec!["pipe"]);
    args.query = "line one\nline two\nline three".to_string();
    let out = rg_md::pipe::execute_pipe(&args);
    assert!(out.contains("### pipe"), "Header should be pipe: {}", out);
    assert!(out.contains("  - L1: line one"), "Should have L1: {}", out);
    assert!(out.contains("  - L2: line two"), "Should have L2: {}", out);
    assert!(out.contains("  - L3: line three"), "Should have L3: {}", out);

    // Also test with diff stream through pipe
    let diff_input = "diff --git a/src/lib.rs b/src/lib.rs\n--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1,3 +1,3 @@\n-old line\n+new line\n";
    let mut diff_args = parse_args(vec!["pipe"]);
    diff_args.query = diff_input.to_string();
    let out_diff = rg_md::pipe::execute_pipe(&diff_args);
    assert!(out_diff.contains("### `src/lib.rs`"), "Diff header should be formatted: {}", out_diff);
    assert!(out_diff.contains("- old line"), "Hunk removal formatted: {}", out_diff);
    assert!(out_diff.contains("+ new line"), "Hunk addition formatted: {}", out_diff);
}

#[test]
fn test_err_and_test_filtering() {
    let raw_log = r#"
   Compiling sample v0.1.0 (/path)
error[E0425]: cannot find value `foo` in this scope
  --> src/main.rs:14:9
   |
14 |         foo.bar()
   |         ^^^ not found in this scope
warning: unused variable `x`
test tests::test_one ... ok
test tests::test_two ... FAILED
failures:
---- tests::test_two stdout ----
thread 'tests::test_two' panicked at 'assertion failed: `(left == right)`', tests.rs:42:9
test result: FAILED. 1 passed; 1 failed
"#;

    let out = run_from_args(&["err", raw_log]);

    assert!(
        out.contains("### Error"),
        "Must extract compilation error header: {}",
        out
    );
    assert!(
        out.contains("E0425") || out.contains("cannot find value"),
        "Must extract compiler error details: {}",
        out
    );
    assert!(
        out.contains("### Test Failures"),
        "Must extract test failure header: {}",
        out
    );
    assert!(
        out.contains("test tests::test_two ... FAILED") || out.contains("assertion failed"),
        "Must extract failure details: {}",
        out
    );
    assert!(
        !out.contains("Compiling sample"),
        "Must filter out non-error compiler noise: {}",
        out
    );
    assert!(
        !out.contains("test tests::test_one ... ok"),
        "Must filter out passing test lines: {}",
        out
    );
}

#[test]
fn test_budget_token_capping() {
    let ws = create_temp_workspace("budget");

    // 1. Search with character budget
    let out_search = run_from_args(&[
        "calculate_score",
        ws.to_str().unwrap(),
        "--max-chars=40",
    ]);
    assert!(
        out_search.len() <= 120,
        "Search length should be capped: {}",
        out_search.len()
    );
    assert!(
        out_search.contains("[truncated: budget limit reached]"),
        "Search output must contain budget marker: {}",
        out_search
    );

    // 2. Find with token budget (-b)
    let out_find = run_from_args(&[
        "find",
        "*.rs",
        ws.to_str().unwrap(),
        "-b=15",
    ]);
    assert!(
        out_find.len() <= 150,
        "Find length should be capped: {}",
        out_find.len()
    );
    assert!(
        out_find.contains("[truncated: budget limit reached]"),
        "Find output must contain budget marker: {}",
        out_find
    );

    // 3. Tree with character budget
    let out_tree = run_from_args(&[
        "tree",
        ws.to_str().unwrap(),
        "--max-chars=50",
    ]);
    assert!(
        out_tree.len() <= 150,
        "Tree length should be capped: {}",
        out_tree.len()
    );
    assert!(
        out_tree.contains("[truncated: budget limit reached]"),
        "Tree output must contain budget marker: {}",
        out_tree
    );

    // 4. Pipe with token budget
    let large_pipe_input = (1..=50)
        .map(|i| format!("Pipeline item number {} with extra content", i))
        .collect::<Vec<_>>()
        .join("\n");
    let out_pipe = run_from_args(&[
        "pipe",
        &large_pipe_input,
        "-b=20",
    ]);
    assert!(
        out_pipe.len() <= 200,
        "Pipe length should be capped: {}",
        out_pipe.len()
    );
    assert!(
        out_pipe.contains("[truncated: budget limit reached]"),
        "Pipe output must contain budget marker: {}",
        out_pipe
    );

    let _ = fs::remove_dir_all(ws);
}

#[test]
fn test_json_output() {
    let ws = create_temp_workspace("json");
    let out = run_from_args(&["calculate_score", ws.to_str().unwrap(), "--json"]);

    assert!(out.starts_with('{'), "Should be valid JSON object: {}", out);
    assert!(out.contains("\"matches\":"), "JSON should contain matches key: {}", out);

    let _ = fs::remove_dir_all(ws);
}

#[test]
fn test_forgiving_cli_argument_parsing() {
    let args1 = parse_args(vec!["search", "foo"]);
    assert_eq!(args1.command, CommandKind::Search);
    assert_eq!(args1.query, "foo");

    let args2 = parse_args(vec!["find", "bar"]);
    assert_eq!(args2.command, CommandKind::Find);
    assert_eq!(args2.query, "bar");

    let args3 = parse_args(vec!["q", "baz"]);
    assert_eq!(args3.command, CommandKind::Search);
    assert_eq!(args3.query, "baz");

    let args4 = parse_args(vec!["slice", "main.rs:1-10"]);
    assert_eq!(args4.command, CommandKind::Slice);
    assert_eq!(args4.query, "main.rs:1-10");

    let args5 = parse_args(vec!["view", "lib.rs"]);
    assert_eq!(args5.command, CommandKind::Slice);

    let args6 = parse_args(vec!["inspect", "MyStruct"]);
    assert_eq!(args6.command, CommandKind::Slice);

    let args7 = parse_args(vec!["map"]);
    assert_eq!(args7.command, CommandKind::Map);

    let args8 = parse_args(vec!["tree"]);
    assert_eq!(args8.command, CommandKind::Tree);

    let args9 = parse_args(vec!["diff", "f1", "f2"]);
    assert_eq!(args9.command, CommandKind::Diff);

    let args10 = parse_args(vec!["pipe"]);
    assert_eq!(args10.command, CommandKind::Pipe);

    let args11 = parse_args(vec!["err"]);
    assert_eq!(args11.command, CommandKind::Err);

    let wsl_path = r"\\wsl.localhost\Ubuntu\home\ivanm\project";
    assert_eq!(translate_platform_path(wsl_path), "/home/ivanm/project");

    let wsl_slash = "//wsl.localhost/Ubuntu/var/log";
    assert_eq!(translate_platform_path(wsl_slash), "/var/log");
}

#[test]
fn test_root_discovery_and_scoping() {
    let root_dir = create_temp_workspace("root");
    let nested_dir = root_dir.join("src").join("components").join("deep");
    fs::create_dir_all(&nested_dir).unwrap();

    let (discovered_root, scope_rel) = discover_root_and_scope(Some(&nested_dir), &[]);

    let canon_root = fs::canonicalize(&root_dir).unwrap();
    assert_eq!(discovered_root, canon_root);
    assert_eq!(scope_rel, Some("src/components/deep".to_string()));

    let _ = fs::remove_dir_all(&root_dir);
}

#[test]
fn test_err_command_execution() {
    let out = run_from_args(&["err", "rustc", "/tmp/nonexistent_test_file_rg_md.rs"]);

    assert!(
        out.contains("### Error"),
        "Must contain structured Error header: {}",
        out
    );
    assert!(
        out.contains("couldn't read") && out.contains("nonexistent_test_file_rg_md.rs"),
        "Must capture actual compiler error details from stderr: {}",
        out
    );
    assert!(
        !out.contains("Command Failed with exit code"),
        "Must not fall back to generic command failed when error is parsed: {}",
        out
    );
    assert!(
        !out.contains("aborting due to"),
        "Must filter out rustc aborting summary boilerplate: {}",
        out
    );
}

#[test]
fn test_test_command_execution() {
    // 1. Successful command execution
    let out_success = run_from_args(&["test", "rustc", "--version"]);
    assert!(
        out_success.contains("### Command Succeeded (0 errors, 0 failures)"),
        "Successful command must report Command Succeeded: {}",
        out_success
    );

    // 2. Failed command execution with error
    let out_fail = run_from_args(&["test", "rustc", "--invalid-flag-rg-md-test"]);
    assert!(
        out_fail.contains("### Error"),
        "Failed command must report Error: {}",
        out_fail
    );
    assert!(
        out_fail.contains("Unrecognized option"),
        "Failed command error details must be captured: {}",
        out_fail
    );
}
