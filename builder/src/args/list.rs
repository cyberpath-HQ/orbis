use std::path::PathBuf;
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct ListArgs {
    #[command(subcommand)]
    pub(crate) command: ListCommands
}

#[derive(Subcommand)]
pub enum ListCommands {
    /// List available key pairs in the specified storage directory
    Keys {
        /// Storage directory where key pairs are stored
        #[arg(short, long, default_value = "./keychains/")]
        storage: PathBuf,
    },
    /// List available plugins in the specified plugins directory
    ///
    /// Optionally include plugins from the cargo build directory.
    /// Notice that duplicates may occur if the same plugin exists in both locations.
    Plugins {
        /// Directory where plugins are stored
        #[arg(short, long, default_value = "./plugins/")]
        plugins_dir: PathBuf,

        /// Include plugins from the cargo build directory (as compiled binaries)
        #[arg(long, default_value_t = false)]
        with_cargo_build: bool,

        /// Include debug builds from the cargo target directory
        #[arg(long, default_value_t = false)]
        with_cargo_debug: bool,
    },
}