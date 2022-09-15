//! A static site generator for photo galleries.
mod error;
mod input;
mod model;
mod output;

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;

/// Commandline arguments.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// If set, then don't write any files.
    #[clap(long = "dry_run")]
    dry_run: bool,

    /// The source directory.
    #[clap(long)]
    input: String,

    /// The output directory.
    #[clap(long)]
    output: String,
}

impl Cli {
    fn output_config(&self) -> output::Config {
        output::Config {
            output_path: PathBuf::from(&self.output),
        }
    }
}

fn run_on_args(args: impl Iterator<Item = std::ffi::OsString>) -> Result<()> {
    let args = Cli::parse_from(args);
    let input_path = PathBuf::from(&args.input);
    let gallery = input::gallery_from_dir(&input_path).with_context(|| "Failed to read gallery")?;
    output::write_files(&gallery, &args.output_config()).with_context(|| "Failed to write gallery")
}

fn main() {
    if let Err(e) = run_on_args(std::env::args_os()) {
        println!("Error: {:?}", e);
    }
}