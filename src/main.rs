use anyhow::Result;
use clap::Parser;

use fc::cli::args::{FcArgs, Commands};
use fc::cli::commands;

fn main() -> Result<()> {
    let args = FcArgs::parse();

    match args.command {
        Commands::Compress { input, output, algorithm, benchmark, verbose, force } => {
            if algorithm != "huffman" {
                anyhow::bail!("unsupported algorithm: {} (only 'huffman' is supported)", algorithm);
            }
            commands::run_compress(&input, output.as_deref(), benchmark, verbose, force)
        }
        Commands::Decompress { input, output, verify, verbose } => {
            commands::run_decompress(&input, output.as_deref(), verify, verbose)
        }
        Commands::Benchmark { input, iterations, compare, output: _ } => {
            commands::run_benchmark(&input, iterations, compare)
        }
        Commands::Info { input } => {
            commands::run_info(&input)
        }
    }
}
