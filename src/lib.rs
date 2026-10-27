pub mod cli;
pub mod formatter;
pub mod scanner;
pub mod search;
pub mod slice;

use cli::{parse_args, CliArgs, CommandKind};
use formatter::{render_help, render_version};
use scanner::discover_root_and_scope;

pub fn run_from_args<I, S>(raw_args: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args = parse_args(raw_args);
    run_cli(&args)
}

pub fn run_cli(args: &CliArgs) -> String {
    match args.command {
        CommandKind::Help => render_help(),
        CommandKind::Version => render_version(),
        CommandKind::Search => {
            let (repo_root, scope_rel) =
                discover_root_and_scope(args.path.as_deref(), &args.targets);
            search::execute_search(&repo_root, scope_rel.as_deref(), args)
        }
        CommandKind::Slice => {
            let (repo_root, scope_rel) =
                discover_root_and_scope(args.path.as_deref(), &args.targets);
            slice::execute_slice(&repo_root, scope_rel.as_deref(), args)
        }
        CommandKind::Map => {
            let (repo_root, scope_rel) =
                discover_root_and_scope(args.path.as_deref(), &args.targets);
            scanner::execute_map(&repo_root, scope_rel.as_deref(), args)
        }
    }
}
