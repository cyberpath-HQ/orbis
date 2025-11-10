use std::path::PathBuf;

use clap::Args;

#[derive(Args)]
pub struct VerifyArgs {
    /// Label of the key pair to use for signing
    ///
    /// If a public key is provided, this is not used
    #[arg(short, long, default_value = "default")]
    pub(crate) label:  String,

    /// Public key in hex format to use for verification
    ///
    /// If not provided, the public key from the key pair with the given label will be used
    #[arg(short = 'k', long)]
    pub(crate) public_key: Option<String>,
    
    /// Storage directory where key pairs are stored
    #[arg(short, long, default_value = "./keychains/")]
    pub(crate) storage: PathBuf,
    
    /// Input file to verify signature sign
    #[arg(value_name = "INPUT")]
    pub(crate) input:  PathBuf,

    /// Signature in hex format to verify
    ///
    /// If not provided, the signature will be read from a file with the same name as the input file, followed by a ".sig" extension
    #[arg(value_name = "SIGNATURE")]
    pub(crate) signature: Option<String>,
}
