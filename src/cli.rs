use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandKind {
    Search,
    Slice,
    Map,
    Help,
    Version,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Budget {
    pub max_tokens: Option<usize>,
    pub max_chars: Option<usize>,
}

impl Budget {
    pub fn effective_char_limit(&self) -> usize {
        if let Some(chars) = self.max_chars {
            chars
        } else if let Some(tokens) = self.max_tokens {
            tokens.saturating_mul(4)
        } else {
            usize::MAX
        }
    }

    pub fn is_unlimited(&self) -> bool {
        self.max_tokens.is_none() && self.max_chars.is_none()
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
    pub ignore_case: bool,
    pub hidden: bool,
    pub context: usize,
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
            ignore_case: false,
            hidden: false,
            context: 0,
        }
    }
}

pub fn match_subcommand(arg: &str) -> Option<CommandKind> {
    match arg.to_ascii_lowercase().as_str() {
        "search" | "find" | "q" | "query" | "grep" => Some(CommandKind::Search),
        "slice" | "view" | "inspect" | "cat" | "show" | "def" => Some(CommandKind::Slice),
        "map" | "tree" | "ls" | "files" | "list" | "overview" => Some(CommandKind::Map),
        "help" | "-h" | "--help" => Some(CommandKind::Help),
        "version" | "-v" | "-V" | "--version" => Some(CommandKind::Version),
        _ => None,
    }
}

/// Translates WSL paths (`/home/...`, `/mnt/c/...`) to Windows UNC/drive paths when running on Windows,
/// and translates Windows UNC paths (`\\wsl.localhost\Ubuntu\...`) to `/...` when running on Linux.
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
        if arg == "-V" || arg == "--version" {
            cli.command = CommandKind::Version;
            return cli;
        }

        if arg.starts_with('-') {
            if let Some((flag, val)) = arg.split_once('=') {
                match flag {
                    "-p" | "--path" | "--dir" | "--repo" => {
                        cli.path = Some(PathBuf::from(translate_platform_path(val)));
                    }
                    "-b" | "--budget" | "--tokens" | "--max-tokens" => {
                        if let Ok(n) = val.parse::<usize>() {
                            cli.budget.max_tokens = Some(n);
                        }
                    }
                    "--max-chars" => {
                        if let Ok(n) = val.parse::<usize>() {
                            cli.budget.max_chars = Some(n);
                        }
                    }
                    "-C" | "--context" => {
                        if let Ok(n) = val.parse::<usize>() {
                            cli.context = n;
                        }
                    }
                    _ => {}
                }
            } else {
                match arg.as_str() {
                    "-i" | "--ignore-case" => {
                        cli.ignore_case = true;
                    }
                    "-s" | "--case-sensitive" => {
                        cli.ignore_case = false;
                    }
                    "--hidden" => {
                        cli.hidden = true;
                    }
                    "--json" => {
                        cli.json = true;
                    }
                    "-n" | "--line-number" | "--heading" => {
                        // Accepted benign compatibility flags
                    }
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
                                cli.budget.max_tokens = Some(n);
                            }
                        }
                    }
                    "--max-chars" => {
                        if i + 1 < args_vec.len() {
                            i += 1;
                            if let Ok(n) = args_vec[i].parse::<usize>() {
                                cli.budget.max_chars = Some(n);
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
                    _ => {}
                }
            }
        } else {
            positionals.push(arg.clone());
        }
        i += 1;
    }

    // Determine command from first positional if it matches an alias
    if !positionals.is_empty() {
        if let Some(cmd) = match_subcommand(&positionals[0]) {
            explicit_command = Some(cmd);
            positionals.remove(0);
        }
    }

    cli.command = explicit_command.unwrap_or(CommandKind::Search);

    // Order-independent positional path resolution
    match cli.command {
        CommandKind::Help | CommandKind::Version => {}
        CommandKind::Map => {
            if cli.path.is_none() && !positionals.is_empty() {
                cli.path = Some(PathBuf::from(translate_platform_path(&positionals[0])));
                positionals.remove(0);
            }
            cli.targets = positionals;
        }
        CommandKind::Slice => {
            if cli.path.is_none() && positionals.len() >= 2 {
                // If second positional is an existing directory or looks like a dir, treat as path
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
                        // Directory passed first: `rg-md /path "pattern"`
                        cli.path = Some(PathBuf::from(cand0));
                        cli.query = positionals[1].clone();
                        positionals = vec![positionals[1].clone()];
                    } else {
                        // Pattern passed first: `rg-md "pattern" /path`
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
