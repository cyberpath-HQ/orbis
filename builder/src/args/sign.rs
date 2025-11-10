use std::path::PathBuf;

use clap::Args;

#[derive(Args)]
pub struct SignArgs {
    /// Label of the key pair to use for signing
    #[arg(short, long, default_value = "default")]
    pub(crate) label:  String,
    
    /// Storage directory where key pairs are stored
    #[arg(short, long, default_value = "./keychains/")]
    pub(crate) storage: PathBuf,
    
    /// Input file to sign
    #[arg(value_name = "INPUT")]
    pub(crate) input:  PathBuf,
    
    /// Where to write the signature file, if not specified, no file is written
    #[arg(short, long)]
    pub(crate) output: Option<PathBuf>,
}
