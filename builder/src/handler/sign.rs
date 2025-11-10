use tracing::{debug, error, info, trace};
use signer::SigningKeyPair;
use crate::args::sign::SignArgs;

pub fn handle(args: SignArgs) {
    debug!("Signing file '{}' using key label '{}' from storage '{}'", args.input.display(), args.label, args.storage.display());
    
    let private_key_path = args.storage.join(format!("{}.key", args.label));
    debug!("Loading private key from '{}'", private_key_path.display());
    
    if !private_key_path.exists() {
        error!("Private key with label '{}' not found in storage '{}'", args.label, args.storage.display());
        return
    }
    
    trace!("Reading private key file '{}'", private_key_path.display());
    let private_key_kext = match std::fs::read_to_string(&private_key_path) {
        Ok(content) => content,
        Err(e) => {
            error!("Failed to read private key file '{}': {}", private_key_path.display(), e);
            return
        }
    };
    trace!("Private key file read successfully");
    
    let private_key_hex = private_key_kext.trim();
    trace!("Deriving keypair from private key");
    let keypair = match SigningKeyPair::from_private_key_hex(private_key_hex) {
        Ok(kp) => kp,
        Err(e) => {
            error!("Failed to load private key from '{}': {}", private_key_path.display(), e);
            return
        }
    };
    trace!("Keypair derived successfully");
    
    debug!("Checking if input file '{}' exists", args.input.display());
    if !args.input.exists() {
        error!("Input file '{}' does not exist", args.input.display());
        return
    }
    
    trace!("Signing file '{}'", args.input.display());
    let signature = match keypair.sign_file(&args.input) {
        Ok(sig) => sig,
        Err(e) => {
            error!("Failed to sign file '{}': {}", args.input.display(), e);
            return
        }
    };
    trace!("File signed successfully");
    
    if let Some(output_path) = args.output {
        debug!("Output path specified: '{}'", output_path.display());
        
        match std::fs::write(&output_path, signature.to_hex()) {
            Ok(_) => {
                debug!("Signature written to '{}'", output_path.display());
            }
            Err(e) => {
                error!("Failed to write signature to '{}': {}", output_path.display(), e);
            }
        }
    } else {
        trace!("No output path specified, skipping writing signature to file");
        info!(signature = %signature.to_hex(), public_key = %signature.public_key().to_hex(), "File signed successfully");
    }
}