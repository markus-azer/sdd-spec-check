mod check;
mod collect;
mod config;
mod report;

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Result;
use clap::Parser;

use crate::config::{Config, Severity};

#[derive(Parser)]
#[command(name = "sdd-spec-check", version, about)]
struct Cli {
    #[arg(long, value_name = "PATH")]
    config: Option<PathBuf>,
    #[arg(long = "specs", value_name = "GLOB")]
    specs: Vec<String>,
    #[arg(long = "tests", value_name = "GLOB")]
    tests: Vec<String>,
    #[arg(long = "on-empty-glob")]
    on_empty_glob: Option<Severity>,
}

fn main() -> ExitCode {
    match run() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(1),
        Err(err) => {
            eprintln!("sdd-spec-check: {err:#}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<bool> {
    let cli = Cli::parse();
    let config = Config::load(
        cli.config.as_deref(),
        &cli.specs,
        &cli.tests,
        cli.on_empty_glob,
    )?;
    let collected = collect::collect(&config)?;
    let outcome = check::run(&collected, &config);
    report::print(&outcome);
    Ok(outcome.ok())
}
