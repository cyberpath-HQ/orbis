use tracing::{debug, error, info, warn};
use crate::args::hash::HashArgs;

/// Handle the 'hash' command to compute sha3-512 hashes of given files
///
/// # Arguments
///
/// * `filenames` - A vector of PathBuf representing the files to hash
pub fn handle(args: HashArgs) {
    debug!("Handling 'hash' command for {} file(s)", args.filenames.len());

    for filename in args.filenames {
        if filename.exists() && filename.is_file() {
            debug!(filename = %filename.display(), "Hashing file");
            let result = hasher::hash_file(&filename);

            if !result.is_ok() {
                error!(filename = %filename.display(), error = %result.unwrap_err(), "Failed to hash file");
            }
            else {
                let hash = result.unwrap();
                info!(filename = %filename.display(), hash = %hash, "Hashed file");
            }
        }
        else {
            warn!(filename = %filename.display(), "File does not exist or is not a regular file");
        }
    }
}