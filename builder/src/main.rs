mod handler;

use std::fmt::Display;
use std::path::PathBuf;
use clap::{Parser, Subcommand};
use tracing::trace;

/// Orbis Builder CLI, allows flexible and advanced project building and management
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Verbosity level (0-3)
    ///
    /// Use multiple times for increased verbosity
    /// -v for DEBUG, -vv for TRACE
    ///
    /// Default verbosity is INFO
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Output in JSON format
    #[arg(short, long, default_value_t = false)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build the Orbis project
    Build {
        /// Build in release mode
        #[arg(short, long, default_value_t = false)]
        release: bool,

        /// Additional plugins to include
        #[arg(short, long, value_delimiter = ',')]
        plugins: Vec<String>,

        /// Build all components, including all plugins in the plugins/ directory
        ///
        /// Note: This overrides the --plugins option
        #[arg(short, long, default_value_t = false)]
        all: bool,
    },
    /// Compute the sha3-512 hash of (plugin) files
    Hash {
        /// Path to the file(s) to hash
        #[arg(value_name = "FILES", required = true)]
        filenames: Vec<PathBuf>
    },
    Sign {

    },
    Keygen {

    }
}

impl Display for Commands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Commands::Build { release: _, plugins: _, all: _ } => {
                write!(f, "Build")
            }
            Commands::Hash { filenames: _ } => {
                write!(f, "Hash")
            }
        }
    }
}

fn main() {
    let args = Args::parse();

    if args.json {
        tracing_subscriber::fmt()
            .with_ansi(true)
            .with_max_level(match args.verbose {
                0 => tracing::Level::INFO,
                1 => tracing::Level::DEBUG,
                _ => tracing::Level::TRACE,
            })
            .json()
            .with_target(false)
            .without_time()
            .init();
    }
    else {
        tracing_subscriber::fmt()
            .with_ansi(true)
            .with_max_level(match args.verbose {
                0 => tracing::Level::INFO,
                1 => tracing::Level::DEBUG,
                _ => tracing::Level::TRACE,
            })
            .with_target(false)
            .without_time()
            .init();
    }


    trace!("Running for command: {}", args.command);
    match args.command {
        Commands::Build { .. } => {}
        Commands::Hash { filenames } => {
            handler::hash::handle(filenames)
        }
    }
}
