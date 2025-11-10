use std::fs::create_dir_all;
use tracing::{debug, error, info, trace};
use crate::args::keygen::KeygenArgs;

pub fn handle(args: KeygenArgs) {
    debug!("Creating key pair with label: {}", args.label);
    
    trace!("Constructing output directory tree");
    if let Err(e) = create_dir_all(&args.output) {
        error!("Failed to create output directory: {}", e);
        return;
    }

    let private_key_file = args.output.join(format!("{}.key", args.label));
    let public_key_file = args.output.join(format!("{}.pub", args.label));
    debug!("Private key file will be at: {}", private_key_file.display());
    debug!("Public key file will be at: {}", public_key_file.display());

    if private_key_file.exists() {
        error!("Cannot generate, private key file already exists at {}", private_key_file.display());
        return;
    }

    trace!("Generating Ed25519 key pair");
    let keypair = signer::SigningKeyPair::generate();
    trace!("Key pair generated successfully");
    
    debug!("Saving private and public keys to files");
    let private_hex = keypair.private_key_hex();
    let public_hex = keypair.public_key_hex();

    if let Err(e) = std::fs::write(&private_key_file, &private_hex) {
        error!("Failed to write private key file: {}", e);
        return;
    }
    if let Err(e) = std::fs::write(&public_key_file, &public_hex) {
        error!("Failed to write public key file: {}", e);
        return;
    }

    info!(public_hex, label = %args.label, "Key pair generated successfully");
}