/// Plugin signature verification system using Ed25519
#[cfg(feature = "signing")]
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

use serde::{Deserialize, Serialize};

#[cfg(feature = "signing")]
use std::path::Path;

#[cfg(feature = "signing")]
use crate::PluginError;

/// Ed25519 public key for signature verification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[derive(bincode::Encode, bincode::Decode)]
pub struct PublicKey {
    /// Raw public key bytes (32 bytes) - Vec for serde compatibility
    #[serde(with = "serde_bytes")]
    pub key_bytes: Vec<u8>,
    /// Optional label for this key (e.g., "official", "partner-acme")
    pub label: Option<String>,
}

impl PublicKey {
    /// Create from raw bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self {
            key_bytes: bytes.to_vec(),
            label: None,
        }
    }

    /// Get as array
    pub fn as_bytes(&self) -> [u8; 32] {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&self.key_bytes[..32]);
        arr
    }

    /// Create with label
    pub fn with_label(bytes: [u8; 32], label: String) -> Self {
        Self {
            key_bytes: bytes.to_vec(),
            label: Some(label),
        }
    }

    /// Create from hex string
    #[cfg(feature = "signing")]
    pub fn from_hex(hex_str: &str) -> Result<Self, PluginError> {
        let bytes = hex::decode(hex_str)
            .map_err(|e| PluginError::SignatureError(format!("Invalid hex string: {}", e)))?;

        if bytes.len() != 32 {
            return Err(PluginError::SignatureError(
                format!("Public key must be 32 bytes, got {}", bytes.len())
            ));
        }

        Ok(Self {
            key_bytes: bytes,
            label: None,
        })
    }

    /// Convert to hex string
    #[cfg(feature = "signing")]
    pub fn to_hex(&self) -> String {
        hex::encode(&self.key_bytes)
    }

    /// Get verifying key
    #[cfg(feature = "signing")]
    fn to_verifying_key(&self) -> Result<VerifyingKey, PluginError> {
        let arr = self.as_bytes();
        VerifyingKey::from_bytes(&arr)
            .map_err(|e| PluginError::SignatureError(format!("Invalid public key: {}", e)))
    }
}

/// Plugin signature
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(bincode::Encode, bincode::Decode)]
pub struct PluginSignature {
    /// Ed25519 signature bytes (64 bytes) - Vec for serde compatibility
    #[serde(with = "serde_bytes")]
    pub signature_bytes: Vec<u8>,

    /// Public key that created this signature
    public_key: PublicKey,
    /// Optional metadata
    pub metadata: Option<SignatureMetadata>,
}


/// Signature metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(bincode::Encode, bincode::Decode)]
pub struct SignatureMetadata {
    /// When the signature was created
    pub created_at: String,
    /// Signer identity
    pub signer: Option<String>,
    /// Any additional notes
    pub notes: Option<String>,
}

impl PluginSignature {
    /// Create from raw bytes
    pub fn from_bytes(signature_bytes: [u8; 64], public_key: PublicKey) -> Self {
        Self {
            signature_bytes: signature_bytes.to_vec(),
            public_key,
            metadata: None,
        }
    }

    /// Get signature as array
    pub fn as_bytes(&self) -> [u8; 64] {
        let mut arr = [0u8; 64];
        arr.copy_from_slice(&self.signature_bytes[..64]);
        arr
    }

    /// Get public key
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    /// Create from hex string
    #[cfg(feature = "signing")]
    pub fn from_hex(hex_str: &str, public_key: PublicKey) -> Result<Self, PluginError> {
        let bytes = hex::decode(hex_str)
            .map_err(|e| PluginError::SignatureError(format!("Invalid hex string: {}", e)))?;

        if bytes.len() != 64 {
            return Err(PluginError::SignatureError(
                format!("Signature must be 64 bytes, got {}", bytes.len())
            ));
        }

        Ok(Self {
            signature_bytes: bytes,
            public_key,
            metadata: None,
        })
    }

    /// Convert to hex string
    #[cfg(feature = "signing")]
    pub fn to_hex(&self) -> String {
        hex::encode(&self.signature_bytes)
    }

    /// Verify signature against file content
    #[cfg(feature = "signing")]
    pub fn verify(&self, content: &[u8]) -> Result<bool, PluginError> {
        let verifying_key = self.public_key.to_verifying_key()?;
        let sig_bytes = self.as_bytes();
        let signature = Signature::from_bytes(&sig_bytes);

        match verifying_key.verify(content, &signature) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

/// Signing key pair (for plugin developers to sign their plugins)
#[cfg(feature = "signing")]
pub struct SigningKeyPair {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

#[cfg(feature = "signing")]
impl SigningKeyPair {
    /// Generate a new random key pair
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
        let verifying_key = signing_key.verifying_key();

        Self {
            signing_key,
            verifying_key,
        }
    }

    /// Get the public key
    pub fn public_key(&self) -> PublicKey {
        PublicKey::from_bytes(self.verifying_key.to_bytes())
    }

    /// Get public key as hex
    pub fn public_key_hex(&self) -> String {
        hex::encode(self.verifying_key.to_bytes())
    }

    /// Sign a file
    pub fn sign_file<P: AsRef<Path>>(&self, path: P) -> Result<PluginSignature, PluginError> {
        let content = std::fs::read(path.as_ref())?;
        self.sign(&content)
    }

    /// Sign content
    pub fn sign(&self, content: &[u8]) -> Result<PluginSignature, PluginError> {
        let signature = self.signing_key.sign(content);
        let signature_bytes = signature.to_bytes();

        Ok(PluginSignature {
            signature_bytes: signature_bytes.to_vec(),
            public_key: self.public_key(),
            metadata: Some(SignatureMetadata {
                created_at: chrono::Utc::now().to_rfc3339(),
                signer: None,
                notes: None,
            }),
        })
    }

    /// Export private key as hex (WARNING: Keep secret!)
    pub fn private_key_hex(&self) -> String {
        hex::encode(self.signing_key.to_bytes())
    }

    /// Import from private key hex
    pub fn from_private_key_hex(hex_str: &str) -> Result<Self, PluginError> {
        let bytes = hex::decode(hex_str)
            .map_err(|e| PluginError::SignatureError(format!("Invalid hex string: {}", e)))?;

        if bytes.len() != 32 {
            return Err(PluginError::SignatureError(
                format!("Private key must be 32 bytes, got {}", bytes.len())
            ));
        }

        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&bytes);

        let signing_key = SigningKey::from_bytes(&key_bytes);
        let verifying_key = signing_key.verifying_key();

        Ok(Self {
            signing_key,
            verifying_key,
        })
    }
}

/// Signature verification helper
#[cfg(feature = "signing")]
pub fn verify_plugin_signature<P: AsRef<Path>>(
    plugin_path: P,
    signature: &PluginSignature,
) -> Result<bool, PluginError> {
    let content = std::fs::read(plugin_path.as_ref())?;
    signature.verify(&content)
}

/// Verify against multiple public keys
#[cfg(feature = "signing")]
pub fn verify_with_keys<P: AsRef<Path>>(
    plugin_path: P,
    signature: &PluginSignature,
    allowed_keys: &[PublicKey],
) -> Result<bool, PluginError> {
    // Check if signature's public key is in allowed list
    if !allowed_keys.contains(&signature.public_key) {
        return Ok(false);
    }

    // Verify the signature
    verify_plugin_signature(plugin_path, signature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let keypair = SigningKeyPair::generate();
        let public_key_hex = keypair.public_key_hex();
        assert_eq!(public_key_hex.len(), 64); // 32 bytes as hex
    }

    #[test]
    fn test_sign_and_verify() {
        let keypair = SigningKeyPair::generate();
        let content = b"test plugin content";

        let signature = keypair.sign(content).unwrap();
        assert!(signature.verify(content).unwrap());

        // Wrong content should fail
        let wrong_content = b"wrong content";
        assert!(!signature.verify(wrong_content).unwrap());
    }

    #[test]
    fn test_public_key_hex() {
        let keypair = SigningKeyPair::generate();
        let hex = keypair.public_key_hex();
        let public_key = PublicKey::from_hex(&hex).unwrap();
        assert_eq!(public_key.key_bytes, keypair.public_key().key_bytes);
    }

    #[test]
    fn test_key_import_export() {
        let keypair1 = SigningKeyPair::generate();
        let private_hex = keypair1.private_key_hex();
        let public_hex = keypair1.public_key_hex();

        let keypair2 = SigningKeyPair::from_private_key_hex(&private_hex).unwrap();
        assert_eq!(keypair2.public_key_hex(), public_hex);
    }
}

