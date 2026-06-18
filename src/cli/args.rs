use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "fc", about = "File compression tool", version)]
pub struct FcArgs {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Compress a file
    Compress {
        /// Input file
        #[arg(short, long)]
        input: String,
        /// Output file
        #[arg(short, long)]
        output: Option<String>,
        /// Algorithm
        #[arg(short, long, default_value = "huffman")]
        algorithm: String,
        /// Show benchmark stats
        #[arg(long)]
        benchmark: bool,
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
        /// Overwrite output
        #[arg(short, long)]
        force: bool,
    },
    /// Decompress a file
    Decompress {
        /// Input file
        #[arg(short, long)]
        input: String,
        /// Output file
        #[arg(short, long)]
        output: Option<String>,
        /// Verify CRC32
        #[arg(long, default_value_t = true)]
        verify: bool,
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Run benchmarks
    Benchmark {
        /// Input file
        #[arg(short, long)]
        input: String,
        /// Number of iterations
        #[arg(short = 'n', long, default_value_t = 5)]
        iterations: usize,
        /// Compare against gzip
        #[arg(long)]
        compare: bool,
        /// Save results as JSON
        #[arg(long)]
        output: Option<String>,
    },
    /// Show compressed file info
    Info {
        /// Input file
        #[arg(short, long)]
        input: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_compress_subcommand() {
        let cmd = FcArgs::command();
        let result = cmd.try_get_matches_from(["fc", "compress", "-i", "test.txt", "-o", "test.huff"]);
        assert!(result.is_ok());
    }

    #[test]
    fn cli_decompress_subcommand() {
        let cmd = FcArgs::command();
        let result = cmd.try_get_matches_from(["fc", "decompress", "-i", "test.huff", "-o", "test.txt"]);
        assert!(result.is_ok());
    }

    #[test]
    fn cli_benchmark_subcommand() {
        let cmd = FcArgs::command();
        let result = cmd.try_get_matches_from(["fc", "benchmark", "-i", "test.txt"]);
        assert!(result.is_ok());
    }

    #[test]
    fn cli_info_subcommand() {
        let cmd = FcArgs::command();
        let result = cmd.try_get_matches_from(["fc", "info", "-i", "test.huff"]);
        assert!(result.is_ok());
    }

    #[test]
    fn compress_requires_input() {
        let cmd = FcArgs::command();
        let result = cmd.try_get_matches_from(["fc", "compress"]);
        assert!(result.is_err());
    }
}
