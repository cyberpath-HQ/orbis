mod handler;
mod args;

use std::fmt::Display;
use clap::{Parser, Subcommand};
use tracing::trace;
use crate::args::hash::HashArgs;
use crate::args::keygen::KeygenArgs;
use crate::args::list::ListArgs;
use crate::args::sign::SignArgs;
use crate::args::verify::VerifyArgs;

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
        /// Clean previous build artifacts
        #[arg(short, long, default_value_t = false)]
        clean: bool,

        /// Build in release mode
        #[arg(short, long, default_value_t = false)]
        release: bool,

        /// Additional plugins to include
        ///
        /// Comma-separated list of plugin names to build (without path or extension)
        /// e.g., --plugins plugin1,plugin2
        ///
        /// Plugins should be located in the plugins/ directory
        ///
        /// Special values:
        /// - "all" or "*": Build all plugins found in the plugins/ directory
        #[arg(short, long, value_delimiter = ',')]
        plugins: Vec<String>,

        /// Build all components, including all plugins in the plugins/ directory
        ///
        /// Note: This overrides the --plugins option
        #[arg(short, long, default_value_t = false)]
        all: bool,
    },
    /// Compute the sha3-512 hash of (plugin) files
    Hash(HashArgs),
    /// Sign a file using a stored key pair
    Sign(SignArgs),
    /// Verify a file's signature using a stored key pair or provided public key
    Verify(VerifyArgs),
    /// Generate a new key pair and store it in the specified storage
    Keygen(KeygenArgs),
    /// List available keys and plugins
    List(ListArgs),
}

impl Display for Commands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Commands::Build {..} => {
                write!(f, "Build")
            }
            Commands::Hash(_) => {
                write!(f, "Hash")
            }
            Commands::Sign(_) => {
                write!(f, "Sign")
            }
            Commands::Keygen(_) => {
                write!(f, "Keygen")
            }
            Commands::Verify(_) => {
                write!(f, "Verify")
            }
            Commands::List(_) => {
                write!(f, "List")
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
        Commands::Hash(a) => {
            handler::hash::handle(a)
        }
        Commands::Sign(a) => {
            handler::sign::handle(a)
        }
        Commands::Verify(a) => {
            handler::verify::handle(a)
        }
        Commands::Keygen(a) => {
            handler::keygen::handle(a)
        }
        Commands::List(a) => {
            handler::list::handle(a, args.json)
        }
    }
}
