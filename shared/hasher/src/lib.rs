use std::fs::read;
use std::path::Path;
use sha3::{Digest, Sha3_512};

/// Compute the SHA3-512 hash of the given data and return it as a hexadecimal string
///
/// # Arguments
///
/// * `data` - A reference to the data to hash
pub fn hash<D: AsRef<[u8]>>(data: D) -> String {
    let mut hasher = Sha3_512::new();
    hasher.update(&data);
    format!("{:x}", hasher.finalize())
}

/// Compute the SHA3-512 hash of a file specified by its path
///
/// # Arguments
///
/// * `file` - A reference to a path of the file to hash
pub fn hash_file<F: AsRef<Path>>(file: F) -> Result<String, std::io::Error> {
    let data = read(file)?;
    Ok(hash(data))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_hash() {
        let result = hash("hello");
        assert_eq!(
            result.as_str(),
            "75d527c368f2efe848ecf6b073a36767800805e9eef2b1857d5f984f036eb6df891d75f72d9b154518c1cd58835286d1da9a38deba3de98b5a53e5ed78a84976"
        );
    }

    #[test]
    fn can_hash_file() {
        let test_file = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/test.txt");
        let result = hash_file(test_file).unwrap();
        assert_eq!(
            result.as_str(),
            "28130646fb5c71337a700c5eb7060ff87c367b6c40f970f22937f36a6f0fed6ee746755c3185c602ac8c3b7faeb0a695841649f3ff83c1efe24b02c6d259ba08"
        );
    }
}
