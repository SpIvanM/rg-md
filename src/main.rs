use rg_md::run_from_args;
use std::env;
use std::io::{self, Write};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let output = run_from_args(args);
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let _ = handle.write_all(output.as_bytes());
    let _ = handle.flush();
}
