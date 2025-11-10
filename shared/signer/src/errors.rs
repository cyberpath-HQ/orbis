use ed25519_dalek::SignatureError;
use hex::FromHexError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SignerErrors {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid hex string: {0}")]
    HexFormatError(#[from] FromHexError),
    
    #[error("Format error: {0}")]
    FormatError(String),
    
    #[error("Invalid public key: {0}")]
    InvalidPublicKey(#[from] SignatureError),
}