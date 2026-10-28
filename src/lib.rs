pub mod cli;
pub mod diff;
pub mod exec;
pub mod find;
pub mod formatter;
pub mod pipe;
pub mod scanner;
pub mod search;
pub mod slice;
pub mod tree;

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
        CommandKind::Tree => {
            let (repo_root, scope_rel) =
                discover_root_and_scope(args.path.as_deref(), &args.targets);
            tree::execute_tree(&repo_root, scope_rel.as_deref(), args)
        }
        CommandKind::Find => {
            let (repo_root, scope_rel) =
                discover_root_and_scope(args.path.as_deref(), &args.targets);
            find::execute_find(&repo_root, scope_rel.as_deref(), args)
        }
        CommandKind::Diff => {
            let (repo_root, _) =
                discover_root_and_scope(args.path.as_deref(), &args.targets);
            diff::execute_diff(&repo_root, args)
        }
        CommandKind::Err | CommandKind::Test => {
            exec::execute_err_or_test(args)
        }
        CommandKind::Pipe => {
            pipe::execute_pipe(args)
        }
    }
}
