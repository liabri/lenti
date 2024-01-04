//! A static site generator for photo galleries.
mod error;
mod model;
mod input;
mod output;

use anyhow::{ Context, Result };
use clap::Parser;
use std::path::PathBuf;
use model::Gallery;

/// Commandline arguments.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// The gallery, the directory containing albums
    #[clap(long)]
    input: String,

    /// The output directory.
    #[clap(long)]
    output: String,

    /// The resources directory.
    #[clap(long)]
    resources: String,
}

impl Cli {
    fn output_config(&self) -> output::Config {
        output::Config {
            input_path: PathBuf::from(&self.input),
            output_path: PathBuf::from(&self.output),
            resources_path: PathBuf::from(&self.resources)
        }
    }
}

fn run_on_args(args: impl Iterator<Item = std::ffi::OsString>) -> Result<()> {
    let args = Cli::parse_from(args);
    let gallery = Gallery::from_path(&PathBuf::from(&args.input)).with_context(|| "Failed to read gallery")?;
    output::write_files(&gallery, &args.output_config()).with_context(|| "Failed to write gallery")
}

fn main() {
    if let Err(e) = run_on_args(std::env::args_os()) {
        println!("Error: {:?}", e);
    }
}