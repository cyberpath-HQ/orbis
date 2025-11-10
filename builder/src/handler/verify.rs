use tracing::{debug, error, info, trace};
use signer::{verify_signature, PublicKey, Signature};
use crate::args::verify::VerifyArgs;

pub fn handle(args: VerifyArgs) {
    debug!("Verifying signature for file '{}' using key label '{}' from storage '{}'", args.input.display(), args.label, args.storage.display());

    let public_key = if let Some(pub_key_hex) = args.public_key {
        debug!("Using provided public key for verification");
        match PublicKey::from_hex(&pub_key_hex) {
            Ok(pk) => pk,
            Err(e) => {
                error!("Failed to load provided public key: {}", e);
                return
            }
        }
    } else {
        let public_key_path = args.storage.join(format!("{}.pub", args.label));
        debug!("Loading public key from '{}'", public_key_path.display());

        if !public_key_path.exists() {
            error!("Public key with label '{}' not found in storage '{}'", args.label, args.storage.display());
            return
        }

        trace!("Reading public key file '{}'", public_key_path.display());
        let public_key_kext = match std::fs::read_to_string(&public_key_path) {
            Ok(content) => content,
            Err(e) => {
                error!("Failed to read public key file '{}': {}", public_key_path.display(), e);
                return
            }
        };
        trace!("Public key file read successfully");

        let public_key_hex = public_key_kext.trim();
        match PublicKey::from_hex(public_key_hex) {
            Ok(pk) => pk,
            Err(e) => {
                error!("Failed to load public key from '{}': {}", public_key_path.display(), e);
                return
            }
        }
    };

    debug!("Checking if input file '{}' exists", args.input.display());
    if !args.input.exists() {
        error!("Input file '{}' does not exist", args.input.display());
        return
    }

    let signature_hex = if let Some(sig) = args.signature {
        debug!("Using provided signature for verification");
        sig
    } else {
        let signature_path = args.input.clone().to_str().unwrap().to_string() + ".sig";
        let signature_path = std::path::PathBuf::from(signature_path);
        debug!("Loading signature from file '{}'", signature_path.display());

        if !signature_path.exists() {
            error!("Signature file '{}' does not exist", signature_path.display());
            return
        }

        trace!("Reading signature file '{}'", signature_path.display());
        match std::fs::read_to_string(&signature_path) {
            Ok(content) => content.trim().to_string(),
            Err(e) => {
                error!("Failed to read signature file '{}': {}", signature_path.display(), e);
                return
            }
        }
    };

    let signature = if let Ok(sig) = Signature::from_hex(signature_hex.as_str(), public_key) {
        sig
    } else {
        error!("Failed to parse signature from hex");
        return
    };

    trace!("Verifying signature for file '{}'", args.input.display());
    match verify_signature(&args.input, &signature) {
        Ok(_) => {
            info!(valid = true, "Signature is valid for file '{}'", args.input.display());
        }
        Err(e) => {
            error!(valid = false, "Signature verification failed for file '{}': {}", args.input.display(), e);
        }
    }
}