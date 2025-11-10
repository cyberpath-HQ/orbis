use std::path::PathBuf;
use clap::Args;

#[derive(Args)]
pub struct HashArgs {
    /// Path to the file(s) to hash
    #[arg(value_name = "FILES", required = true)]
    pub(crate) filenames: Vec<PathBuf>
}