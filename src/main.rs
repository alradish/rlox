use std::{
    io::Write,
    path::{Path, PathBuf},
};

use clap::Parser;
use log::{error, info};
use rlox;

#[derive(Debug, Parser)]
struct RloxArgs {
    /// Path to the script file to execute
    script: Option<String>,

    /// Print tokens during execution
    #[arg(short, long, default_value_t = true)]
    print_tokens: bool,
}

fn main() {
    env_logger::builder().filter_level(log::LevelFilter::Trace).init();

    let args = RloxArgs::parse();
    if let Some(script) = args.script {
        info!("Executing script: {}", script);
        run_file(PathBuf::from(&script).as_path(), args.print_tokens);
    } else {
        info!("Run in interactive mode.");
        repl();
    }
}

fn run_file(path: &Path, print_tokens: bool) {
    match std::fs::read_to_string(path) {
        Ok(contents) => rlox::run(contents, print_tokens),
        Err(e) => error!("Failed to read file: {}", e),
    }
}

fn repl() {
    loop {
        let mut input = String::new();
        print!("> ");
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut input).unwrap();
        rlox::run(input, false);
    }
}
