use std::path::PathBuf;
use clap::Args;

#[derive(Args)]
pub struct KeygenArgs {
    /// Label for the generated key pair (e.g., "release-signing")
    #[arg(value_name = "LABEL", default_value = "default")]
    pub(crate) label: String,

    /// Output path for the private key file
    #[arg(short, long, default_value = "./keychains/")]
    pub(crate) output: PathBuf,
}