mod adapters;
mod audit;
mod commands;
mod constants;
mod context;
mod model;
mod output;
mod receipt;
mod repl;
mod research;
mod sanitizer;
mod snapshot;
mod tui;
mod util;

use clap::Parser;
use commands::{execute, Cli};
use context::Context;
use std::process::ExitCode;

pub type RavenResult<T> = Result<T, Box<dyn std::error::Error>>;

fn main() -> ExitCode {
    let cli = Cli::parse();
    let ctx = match Context::load() {
        Ok(ctx) => ctx,
        Err(err) => {
            eprintln!("RAVEN_ERROR: {err}");
            return ExitCode::from(1);
        }
    };

    match execute(cli, &ctx) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("RAVEN_ERROR: {err}");
            ExitCode::from(1)
        }
    }
}
