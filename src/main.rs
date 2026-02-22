use std::{
    io::Write,
    path::{Path, PathBuf},
};

use clap::Parser;
use log::{debug, error, info};

mod scanner;

#[derive(Debug, Parser)]
struct RloxArgs {
    /// Path to the script file to execute
    script: Option<String>,
}

fn main() {
    env_logger::builder().filter_level(log::LevelFilter::Trace).init();

    let args = RloxArgs::parse();
    if let Some(script) = args.script {
        info!("Executing script: {}", script);
        run_file(PathBuf::from(&script).as_path());
    } else {
        info!("Run in interactive mode.");
        repl();
    }
}

fn run_file(path: &Path) {
    match std::fs::read_to_string(path) {
        Ok(contents) => run(contents),
        Err(e) => error!("Failed to read file: {}", e),
    }
}

fn repl() {
    loop {
        let mut input = String::new();
        print!("> ");
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut input).unwrap();
        run(input);
    }
}

fn run(input: String) {
    for token in scanner::scan(&input) {
        debug!("{:?}", token);
    }
}
