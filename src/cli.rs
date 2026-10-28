use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandKind {
    Search,
    Slice,
    Map,
    Tree,
    Find,
    Diff,
    Err,
    Test,
    Pipe,
    Help,
    Version,
}

pub const DEFAULT_TOKEN_BUDGET: usize = 2000;
pub const DEFAULT_CHAR_BUDGET: usize = DEFAULT_TOKEN_BUDGET * 4; // 8000

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Budget {
    pub max_tokens: Option<usize>,
    pub max_chars: Option<usize>,
    pub unlimited: bool,
}

impl Default for Budget {
    fn default() -> Self {
        Self {
            max_tokens: Some(DEFAULT_TOKEN_BUDGET),
            max_chars: None,
            unlimited: false,
        }
    }
}

impl Budget {
    pub fn unlimited() -> Self {
        Self {
            max_tokens: None,
            max_chars: None,
            unlimited: true,
        }
    }

    pub fn is_unlimited(&self) -> bool {
        self.unlimited || (self.max_tokens.is_none() && self.max_chars.is_none())
    }

    pub fn effective_char_limit(&self) -> usize {
        if self.is_unlimited() {
            usize::MAX
        } else if let Some(chars) = self.max_chars {
            chars
        } else if let Some(tokens) = self.max_tokens {
            tokens.saturating_mul(4)
        } else {
            usize::MAX
        }
    }

    pub fn limit_description(&self) -> String {
        if self.is_unlimited() {
            "unlimited".to_string()
        } else if let Some(chars) = self.max_chars {
            format!("{} chars", chars)
        } else if let Some(tokens) = self.max_tokens {
            format!("{} tokens", tokens)
        } else {
            format!("{} tokens", DEFAULT_TOKEN_BUDGET)
        }
    }
}

#[derive(Debug, Clone)]
pub struct CliArgs {
    pub command: CommandKind,
    pub query: String,
    pub targets: Vec<String>,
    pub path: Option<PathBuf>,
    pub budget: Budget,
    pub json: bool,
    pub raw: bool,
    pub ignore_case: bool,
    pub hidden: bool,
    pub context: usize,
    pub head: Option<usize>,
    pub tail: Option<usize>,
    pub depth: Option<usize>,
}

impl Default for CliArgs {
    fn default() -> Self {
        Self {
            command: CommandKind::Search,
            query: String::new(),
            targets: Vec::new(),
            path: None,
            budget: Budget::default(),
            json: false,
            raw: false,
            ignore_case: false,
            hidden: false,
            context: 0,
            head: None,
            tail: None,
            depth: None,
        }
    }
}

pub fn match_subcommand(arg: &str) -> Option<CommandKind> {
    match arg.to_ascii_lowercase().as_str() {
        "search" | "q" | "query" | "grep" => Some(CommandKind::Search),
        "slice" | "view" | "inspect" | "cat" | "show" | "def" => Some(CommandKind::Slice),
        "map" | "overview" => Some(CommandKind::Map),
        "tree" | "ls" => Some(CommandKind::Tree),
        "find" | "files" => Some(CommandKind::Find),
        "diff" => Some(CommandKind::Diff),
        "err" | "errors" => Some(CommandKind::Err),
        "test" | "tests" => Some(CommandKind::Test),
        "pipe" => Some(CommandKind::Pipe),
        "help" | "-h" | "--help" => Some(CommandKind::Help),
        "version" | "-v" | "-V" | "--version" => Some(CommandKind::Version),
        _ => None,
    }
}

pub fn translate_platform_path(raw: &str) -> String {
    let trimmed = raw.trim();
    if cfg!(windows) {
        if let Some(rest) = trimmed.strip_prefix("/mnt/") {
            let mut chars = rest.chars();
            if let (Some(drive), Some('/')) = (chars.next(), chars.next()) {
                if drive.is_ascii_alphabetic() {
                    let remainder: String = chars.collect();
                    return format!(
                        "{}:\\{}",
                        drive.to_ascii_uppercase(),
                        remainder.replace('/', "\\")
                    );
                }
            }
        }
        if trimmed.starts_with("/home/")
            || trimmed.starts_with("/root/")
            || trimmed.starts_with("/tmp/")
            || trimmed.starts_with("/etc/")
            || trimmed.starts_with("/usr/")
            || trimmed.starts_with("/var/")
            || trimmed.starts_with("/opt/")
        {
            return format!(r"\\wsl.localhost\Ubuntu{}", trimmed.replace('/', "\\"));
        }
    } else {
        if let Some(rest) = trimmed
            .strip_prefix(r"\\wsl.localhost\Ubuntu")
            .or_else(|| trimmed.strip_prefix("//wsl.localhost/Ubuntu"))
            .or_else(|| trimmed.strip_prefix(r"\\wsl$\Ubuntu"))
            .or_else(|| trimmed.strip_prefix("//wsl$/Ubuntu"))
        {
            let norm = rest.replace('\\', "/");
            return if norm.starts_with('/') {
                norm
            } else {
                format!("/{}", norm)
            };
        }
        if trimmed.len() >= 2
            && trimmed
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic())
            && trimmed.chars().nth(1) == Some(':')
        {
            let drive = trimmed.chars().next().unwrap().to_ascii_lowercase();
            let remainder = &trimmed[2..].replace('\\', "/");
            let norm_rem = remainder.trim_start_matches('/');
            return format!("/mnt/{}/{}", drive, norm_rem);
        }
    }
    trimmed.to_string()
}

pub fn parse_args<I, S>(raw_args: I) -> CliArgs
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args_vec: Vec<String> = raw_args
        .into_iter()
        .map(|s| s.as_ref().to_string())
        .collect();

    let mut cli = CliArgs::default();
    let mut explicit_command: Option<CommandKind> = None;
    let mut positionals: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args_vec.len() {
        let arg = &args_vec[i];

        if arg == "-h" || arg == "--help" {
            cli.command = CommandKind::Help;
            return cli;
        }
        if (arg == "-v" || arg == "-V" || arg == "--version") && explicit_command.is_none() {
            cli.command = CommandKind::Version;
            return cli;
        }

        if arg == "--" {
            for rem in &args_vec[i + 1..] {
                positionals.push(rem.clone());
            }
            break;
        }

        if arg.starts_with('-') && !arg.contains('\n') && !arg.starts_with("--- ") && !arg.starts_with("@@ ") {
            if arg == "--json" {
                cli.json = true;
            } else if arg == "--raw" {
                cli.raw = true;
            } else if arg == "--unlimited" || arg == "--no-budget" {
                cli.budget.unlimited = true;
                cli.budget.max_tokens = None;
                cli.budget.max_chars = None;
            } else if arg == "-i" || arg == "--ignore-case" {
                cli.ignore_case = true;
            } else if arg == "--hidden" {
                cli.hidden = true;
            } else if let Some(val) = arg.strip_prefix("-p=")
                .or_else(|| arg.strip_prefix("--path="))
                .or_else(|| arg.strip_prefix("--dir="))
                .or_else(|| arg.strip_prefix("--repo="))
            {
                cli.path = Some(PathBuf::from(translate_platform_path(val)));
            } else if let Some(val) = arg.strip_prefix("-b=")
                .or_else(|| arg.strip_prefix("--budget="))
                .or_else(|| arg.strip_prefix("--tokens="))
                .or_else(|| arg.strip_prefix("--max-tokens="))
                .or_else(|| {
                    if arg.starts_with("-b") && arg.len() > 2 && arg.as_bytes()[2].is_ascii_digit() {
                        Some(&arg[2..])
                    } else {
                        None
                    }
                })
            {
                if let Ok(n) = val.parse::<usize>() {
                    if n == 0 {
                        cli.budget.unlimited = true;
                        cli.budget.max_tokens = None;
                        cli.budget.max_chars = None;
                    } else {
                        cli.budget.unlimited = false;
                        cli.budget.max_tokens = Some(n);
                        cli.budget.max_chars = None;
                    }
                }
            } else if let Some(val) = arg.strip_prefix("--max-chars=") {
                if let Ok(n) = val.parse::<usize>() {
                    if n == 0 {
                        cli.budget.unlimited = true;
                        cli.budget.max_tokens = None;
                        cli.budget.max_chars = None;
                    } else {
                        cli.budget.unlimited = false;
                        cli.budget.max_chars = Some(n);
                        cli.budget.max_tokens = None;
                    }
                }
            } else if let Some(val) = arg.strip_prefix("--head=") {
                if let Ok(n) = val.parse::<usize>() {
                    cli.head = Some(n);
                }
            } else if let Some(val) = arg.strip_prefix("--tail=") {
                if let Ok(n) = val.parse::<usize>() {
                    cli.tail = Some(n);
                }
            } else if let Some(val) = arg.strip_prefix("-d=").or_else(|| arg.strip_prefix("--depth=")) {
                if let Ok(n) = val.parse::<usize>() {
                    cli.depth = Some(n);
                }
            } else if let Some(val) = arg.strip_prefix("-C=")
                .or_else(|| arg.strip_prefix("--context="))
            {
                if let Ok(n) = val.parse::<usize>() {
                    cli.context = n;
                }
            } else {
                match arg.as_str() {
                    "-p" | "--path" | "--dir" | "--repo" => {
                        if i + 1 < args_vec.len() {
                            i += 1;
                            cli.path = Some(PathBuf::from(translate_platform_path(&args_vec[i])));
                        }
                    }
                    "-b" | "--budget" | "--tokens" | "--max-tokens" => {
                        if i + 1 < args_vec.len() {
                            i += 1;
                            if let Ok(n) = args_vec[i].parse::<usize>() {
                                if n == 0 {
                                    cli.budget.unlimited = true;
                                    cli.budget.max_tokens = None;
                                    cli.budget.max_chars = None;
                                } else {
                                    cli.budget.unlimited = false;
                                    cli.budget.max_tokens = Some(n);
                                    cli.budget.max_chars = None;
                                }
                            }
                        }
                    }
                    "--max-chars" => {
                        if i + 1 < args_vec.len() {
                            i += 1;
                            if let Ok(n) = args_vec[i].parse::<usize>() {
                                if n == 0 {
                                    cli.budget.unlimited = true;
                                    cli.budget.max_tokens = None;
                                    cli.budget.max_chars = None;
                                } else {
                                    cli.budget.unlimited = false;
                                    cli.budget.max_chars = Some(n);
                                    cli.budget.max_tokens = None;
                                }
                            }
                        }
                    }
                    "--unlimited" | "--no-budget" => {
                        cli.budget.unlimited = true;
                        cli.budget.max_tokens = None;
                        cli.budget.max_chars = None;
                    }
                    "--raw" => {
                        cli.raw = true;
                    }
                    "--head" => {
                        if i + 1 < args_vec.len() {
                            i += 1;
                            if let Ok(n) = args_vec[i].parse::<usize>() {
                                cli.head = Some(n);
                            }
                        }
                    }
                    "--tail" => {
                        if i + 1 < args_vec.len() {
                            i += 1;
                            if let Ok(n) = args_vec[i].parse::<usize>() {
                                cli.tail = Some(n);
                            }
                        }
                    }
                    "-d" | "--depth" => {
                        if i + 1 < args_vec.len() {
                            i += 1;
                            if let Ok(n) = args_vec[i].parse::<usize>() {
                                cli.depth = Some(n);
                            }
                        }
                    }
                    "-C" | "--context" => {
                        if i + 1 < args_vec.len() {
                            i += 1;
                            if let Ok(n) = args_vec[i].parse::<usize>() {
                                cli.context = n;
                            }
                        }
                    }
                    _ => {
                        if explicit_command == Some(CommandKind::Err) || explicit_command == Some(CommandKind::Test) {
                            positionals.push(arg.clone());
                        }
                    }
                }
            }
        } else {
            if explicit_command.is_none() && positionals.is_empty() {
                if let Some(cmd) = match_subcommand(arg) {
                    explicit_command = Some(cmd);
                    i += 1;
                    continue;
                }
            }
            positionals.push(arg.clone());
        }
        i += 1;
    }

    if explicit_command.is_none() && !positionals.is_empty() {
        if let Some(cmd) = match_subcommand(&positionals[0]) {
            explicit_command = Some(cmd);
            positionals.remove(0);
        }
    }

    cli.command = explicit_command.unwrap_or(CommandKind::Search);

    match cli.command {
        CommandKind::Help | CommandKind::Version => {}
        CommandKind::Pipe => {
            if !positionals.is_empty() {
                cli.query = positionals.join("\n");
            }
            cli.targets = positionals;
        }
        CommandKind::Err | CommandKind::Test => {
            if !positionals.is_empty() {
                cli.query = positionals.join(" ");
            }
            cli.targets = positionals;
        }
        CommandKind::Diff => {
            cli.targets = positionals;
        }
        CommandKind::Tree | CommandKind::Map => {
            if cli.path.is_none() && !positionals.is_empty() {
                cli.path = Some(PathBuf::from(translate_platform_path(&positionals[0])));
                positionals.remove(0);
            }
            cli.targets = positionals;
        }
        CommandKind::Find => {
            if cli.path.is_none() && positionals.len() >= 2 {
                let cand1 = translate_platform_path(&positionals[1]);
                let p1 = Path::new(&cand1);
                if p1.is_dir() || cand1.contains('/') || cand1.contains('\\') || cand1 == "." {
                    cli.path = Some(PathBuf::from(cand1));
                    cli.query = positionals[0].clone();
                    positionals = vec![positionals[0].clone()];
                }
            } else if !positionals.is_empty() {
                cli.query = positionals[0].clone();
            }
            cli.targets = positionals;
        }
        CommandKind::Slice => {
            if cli.path.is_none() && positionals.len() >= 2 {
                let candidate = translate_platform_path(&positionals[1]);
                let p = Path::new(&candidate);
                if p.is_dir() || candidate.ends_with('/') || candidate.ends_with('\\') {
                    cli.path = Some(PathBuf::from(candidate));
                    positionals.remove(1);
                }
            }
            if !positionals.is_empty() {
                cli.query = positionals[0].clone();
            }
            cli.targets = positionals;
        }
        CommandKind::Search => {
            if cli.path.is_none() {
                if positionals.len() == 2 {
                    let cand0 = translate_platform_path(&positionals[0]);
                    let cand1 = translate_platform_path(&positionals[1]);
                    let p0 = Path::new(&cand0);
                    let p1 = Path::new(&cand1);

                    if p0.is_dir() && !p1.is_dir() {
                        cli.path = Some(PathBuf::from(cand0));
                        cli.query = positionals[1].clone();
                        positionals = vec![positionals[1].clone()];
                    } else {
                        cli.query = positionals[0].clone();
                        cli.path = Some(PathBuf::from(cand1));
                        positionals = vec![positionals[0].clone()];
                    }
                } else if positionals.len() > 2 {
                    let last_idx = positionals.len() - 1;
                    let cand = translate_platform_path(&positionals[last_idx]);
                    let p = Path::new(&cand);
                    if p.is_dir() || cand.contains('/') || cand.contains('\\') || cand == "." {
                        cli.path = Some(PathBuf::from(cand));
                        positionals.remove(last_idx);
                    }
                    if !positionals.is_empty() {
                        cli.query = positionals[0].clone();
                    }
                } else if positionals.len() == 1 {
                    cli.query = positionals[0].clone();
                }
            } else if !positionals.is_empty() {
                cli.query = positionals[0].clone();
            }
            cli.targets = positionals;
        }
    }

    cli
}
