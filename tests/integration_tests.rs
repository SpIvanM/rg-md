use rg_md::cli::{parse_args, translate_platform_path, CommandKind};
use rg_md::run_from_args;
use rg_md::scanner::discover_root_and_scope;
use serde_json::Value;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

fn create_temp_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "{}_{}_{}",
        prefix,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn test_forgiving_cli_argument_parsing() {
    // 1. Alias recognition
    let args1 = parse_args(vec!["search", "foo"]);
    assert_eq!(args1.command, CommandKind::Search);
    assert_eq!(args1.query, "foo");

    let args2 = parse_args(vec!["find", "bar"]);
    assert_eq!(args2.command, CommandKind::Search);
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
    assert_eq!(args8.command, CommandKind::Map);

    let args9 = parse_args(vec!["ls"]);
    assert_eq!(args9.command, CommandKind::Map);

    // 2. Order independence and flags
    let args_flags = parse_args(vec![
        "-i",
        "-C",
        "3",
        "--hidden",
        "my_query",
        "-p",
        "/tmp/test",
    ]);
    assert_eq!(args_flags.command, CommandKind::Search);
    assert_eq!(args_flags.query, "my_query");
    assert!(args_flags.ignore_case);
    assert_eq!(args_flags.context, 3);
    assert!(args_flags.hidden);
    assert_eq!(args_flags.path, Some(PathBuf::from("/tmp/test")));

    // Flags after query
    let args_flags2 = parse_args(vec!["my_query", "/tmp/test", "-i", "-b", "500", "--json"]);
    assert_eq!(args_flags2.command, CommandKind::Search);
    assert!(args_flags2.ignore_case);
    assert!(args_flags2.json);
    assert_eq!(args_flags2.budget.max_tokens, Some(500));

    // 3. WSL translation
    let wsl_path = r"\\wsl.localhost\Ubuntu\home\ivanm\project";
    assert_eq!(translate_platform_path(wsl_path), "/home/ivanm/project");

    let wsl_slash = "//wsl.localhost/Ubuntu/var/log";
    assert_eq!(translate_platform_path(wsl_slash), "/var/log");
}

#[test]
fn test_markdown_search_formatting() {
    let temp_dir = create_temp_dir("rg_md_test_md");
    let file1 = temp_dir.join("service.ts");
    let mut f1 = File::create(&file1).unwrap();
    writeln!(f1, "import {{ Config }} from './config';").unwrap();
    writeln!(f1, "export class RecordingService {{").unwrap();
    writeln!(f1, "    private active: boolean = false;").unwrap();
    writeln!(f1, "    public startRecording() {{}}").unwrap();
    writeln!(f1, "}}").unwrap();

    let output = run_from_args(vec![
        "RecordingService".to_string(),
        temp_dir.to_string_lossy().to_string(),
    ]);

    assert!(
        output.contains("### `"),
        "Must contain Markdown header: {}",
        output
    );
    assert!(output.contains("service.ts"), "Must mention service.ts");
    assert!(
        output.contains("  - L2: export class RecordingService {"),
        "Must contain formatted line: {}",
        output
    );

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_budget_and_token_truncation() {
    let temp_dir = create_temp_dir("rg_md_test_budget");
    let file = temp_dir.join("large.txt");
    let mut f = File::create(&file).unwrap();
    for i in 1..=200 {
        writeln!(
            f,
            "line {} TARGET_TOKEN with extra padding content for length testing",
            i
        )
        .unwrap();
    }

    // Run with small char budget
    let output = run_from_args(vec![
        "--max-chars".to_string(),
        "300".to_string(),
        "TARGET_TOKEN".to_string(),
        temp_dir.to_string_lossy().to_string(),
    ]);

    assert!(
        output.contains("[truncated: budget limit reached]"),
        "Must indicate truncation: {}",
        output
    );
    assert!(
        output.len() <= 450,
        "Output length ({}) should not exceed budget + notice",
        output.len()
    );

    // Run with token budget
    let output_tok = run_from_args(vec![
        "-b".to_string(),
        "50".to_string(), // 50 tokens ~ 200 chars
        "TARGET_TOKEN".to_string(),
        temp_dir.to_string_lossy().to_string(),
    ]);

    assert!(
        output_tok.contains("[truncated: budget limit reached]"),
        "Must truncate with -b: {}",
        output_tok
    );

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_slice_extraction() {
    let temp_dir = create_temp_dir("rg_md_test_slice");
    let file = temp_dir.join("math.rs");
    let mut f = File::create(&file).unwrap();
    writeln!(f, "// Math module").unwrap();
    writeln!(f, "pub fn add(a: i32, b: i32) -> i32 {{").unwrap();
    writeln!(f, "    a + b").unwrap();
    writeln!(f, "}}").unwrap();
    writeln!(f, "pub fn sub(a: i32, b: i32) -> i32 {{").unwrap();
    writeln!(f, "    a - b").unwrap();
    writeln!(f, "}}").unwrap();

    // 1. Slice by line range
    let slice_spec = format!("{}:2-4", file.to_string_lossy());
    let output = run_from_args(vec!["slice", &slice_spec]);

    assert!(
        output.contains("### `"),
        "Must contain markdown header: {}",
        output
    );
    assert!(
        output.contains(":2-4`"),
        "Header must include line range: {}",
        output
    );
    assert!(
        output.contains("  - L2: pub fn add(a: i32, b: i32) -> i32 {"),
        "Line 2 present: {}",
        output
    );
    assert!(
        output.contains("  - L3: a + b"),
        "Line 3 present: {}",
        output
    );
    assert!(output.contains("  - L4: }"), "Line 4 present: {}", output);
    assert!(
        !output.contains("  - L5:"),
        "Line 5 should not be in 2-4 slice"
    );

    // 2. Slice by symbol
    let sym_spec = format!("{}::sub", file.to_string_lossy());
    let output_sym = run_from_args(vec!["slice", &sym_spec]);
    assert!(
        output_sym.contains("sub"),
        "Symbol sub output: {}",
        output_sym
    );
    assert!(
        output_sym.contains("  - L5: pub fn sub(a: i32, b: i32) -> i32 {"),
        "Line 5 present: {}",
        output_sym
    );

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_root_discovery_and_scoping() {
    let root_dir = create_temp_dir("rg_md_test_root");
    let cargo_toml = root_dir.join("Cargo.toml");
    File::create(&cargo_toml).unwrap();

    let nested_dir = root_dir.join("src").join("components").join("deep");
    fs::create_dir_all(&nested_dir).unwrap();

    let (discovered_root, scope_rel) = discover_root_and_scope(Some(&nested_dir), &[]);

    let canon_root = fs::canonicalize(&root_dir).unwrap();
    assert_eq!(discovered_root, canon_root);
    assert_eq!(scope_rel, Some("src/components/deep".to_string()));

    let _ = fs::remove_dir_all(&root_dir);
}

#[test]
fn test_json_output() {
    let temp_dir = create_temp_dir("rg_md_test_json");
    let file = temp_dir.join("item.rs");
    let mut f = File::create(&file).unwrap();
    writeln!(f, "pub struct Item {{ pub id: u64 }}").unwrap();
    writeln!(f, "pub fn get_item() -> Item {{ Item {{ id: 1 }} }}").unwrap();

    // 1. Search JSON
    let search_json = run_from_args(vec![
        "--json".to_string(),
        "Item".to_string(),
        temp_dir.to_string_lossy().to_string(),
    ]);

    let parsed: Value =
        serde_json::from_str(&search_json).expect("Search output should be valid JSON");
    assert_eq!(parsed["command"], "search");
    assert_eq!(parsed["query"], "Item");
    assert!(parsed["total_matches"].as_u64().unwrap() >= 2);
    assert!(parsed["results"].as_array().unwrap().len() >= 1);

    // 2. Slice JSON
    let slice_spec = format!("{}:1-2", file.to_string_lossy());
    let slice_json = run_from_args(vec!["--json", "slice", &slice_spec]);
    let parsed_slice: Value =
        serde_json::from_str(&slice_json).expect("Slice output should be valid JSON");
    assert_eq!(parsed_slice["command"], "slice");
    assert_eq!(parsed_slice["results"][0]["start_line"], 1);
    assert_eq!(parsed_slice["results"][0]["end_line"], 2);

    // 3. Map JSON
    let map_json = run_from_args(vec![
        "--json",
        "map",
        temp_dir.to_string_lossy().to_string().as_str(),
    ]);
    let parsed_map: Value =
        serde_json::from_str(&map_json).expect("Map output should be valid JSON");
    assert_eq!(parsed_map["command"], "map");
    assert!(parsed_map["total_files"].as_u64().unwrap() >= 1);

    let _ = fs::remove_dir_all(&temp_dir);
}
