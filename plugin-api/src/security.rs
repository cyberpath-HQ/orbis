use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng, rand_core::RngCore},
    XChaCha20Poly1305, XNonce,
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use sha3::{Sha3_512, Digest};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use tracing::{info, warn, error};
use zeroize::Zeroize;
use crate::{PluginError, PublicKey, PluginSignature};

/// Trust level for plugins - only Trusted plugins are loaded
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TrustLevel {
    /// Untrusted plugin - NEVER loaded
    Untrusted = 0,
    /// Trusted plugin - verified and allowed to load
    Trusted = 1,
}

/// Plugin version information
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(bincode::Encode, bincode::Decode)]
pub struct PluginVersion {
    /// Major version
    pub major: u32,
    /// Minor version
    pub minor: u32,
    /// Patch version
    pub patch: u32,
}

impl PluginVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }

    /// Parse version from string (e.g., "1.0.0")
    pub fn from_string(s: &str) -> Result<Self, PluginError> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Err(PluginError::SecurityError(format!("Invalid version format: {}", s)));
        }

        let major = parts[0].parse().map_err(|_| PluginError::SecurityError("Invalid major version".to_string()))?;
        let minor = parts[1].parse().map_err(|_| PluginError::SecurityError("Invalid minor version".to_string()))?;
        let patch = parts[2].parse().map_err(|_| PluginError::SecurityError("Invalid patch version".to_string()))?;

        Ok(Self { major, minor, patch })
    }
}

impl std::fmt::Display for PluginVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Plugin trust entry with hash, version, and signature
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(bincode::Encode, bincode::Decode)]
pub struct TrustedPluginEntry {
    /// SHA3-512 hash of the plugin
    pub hash: String,
    /// Plugin version
    pub version: PluginVersion,
    /// Ed25519 signature (REQUIRED)
    pub signature: PluginSignature,
    /// Optional note/description
    pub note: Option<String>,
}

/// Security policy for plugin loading
#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    /// Only trusted plugins are allowed
    pub only_trusted: bool,
    /// Path to the encrypted user trust list
    pub trust_list_path: Option<PathBuf>,
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            only_trusted: true,
            trust_list_path: Some(PathBuf::from("data/plugin_trust_list.enc")),
        }
    }
}

/// Encrypted trust list stored on disk (binary format)
/// Format: [24-byte nonce][encrypted data]
/// No metadata is exposed - completely opaque binary blob
#[derive(Serialize, Deserialize, Clone)]
struct EncryptedTrustList {
    /// Raw binary data containing nonce + ciphertext
    data: Vec<u8>,
}

/// User's trust list (plaintext, in memory)
#[derive(Serialize, Deserialize, Clone, Default)]
#[derive(bincode::Encode, bincode::Decode)]
struct UserTrustList {
    /// Map of plugin hash to trust entry
    trusted_plugins: HashMap<String, TrustedPluginEntry>,
    /// User-added public keys for signature verification
    user_public_keys: HashSet<PublicKey>,
    /// Format version for future compatibility
    format_version: u32,
}

/// Plugin security manager with hash-based and signature verification
pub struct PluginSecurity {
    policy: SecurityPolicy,
    /// Hardcoded trusted plugin entries (compiled into the binary)
    hardcoded_trusted_plugins: RwLock<HashMap<String, TrustedPluginEntry>>,
    /// User's trusted plugin entries (loaded from encrypted file)
    user_trusted_plugins: RwLock<HashMap<String, TrustedPluginEntry>>,
    /// Hardcoded public keys (compiled into binary)
    hardcoded_public_keys: RwLock<HashSet<PublicKey>>,
    /// User's public keys (loaded from encrypted file)
    user_public_keys: RwLock<HashSet<PublicKey>>,
    /// Cached user credentials hash for encryption/decryption
    credentials_key: RwLock<Option<Vec<u8>>>,
}

impl PluginSecurity {
    /// Create a new security manager with hardcoded trusted plugin entries and public keys
    pub fn new(
        policy: SecurityPolicy,
        hardcoded_trusted_plugins: Vec<TrustedPluginEntry>,
        hardcoded_public_keys: Vec<PublicKey>,
    ) -> Self {
        info!("Initializing plugin security with {} hardcoded trusted plugins and {} public keys",
              hardcoded_trusted_plugins.len(), hardcoded_public_keys.len());

        let plugin_map: HashMap<String, TrustedPluginEntry> = hardcoded_trusted_plugins
            .into_iter()
            .map(|entry| (entry.hash.clone(), entry))
            .collect();

        let public_key_set: HashSet<PublicKey> = hardcoded_public_keys.into_iter().collect();

        Self {
            policy,
            hardcoded_trusted_plugins: RwLock::new(plugin_map),
            user_trusted_plugins: RwLock::new(HashMap::new()),
            hardcoded_public_keys: RwLock::new(public_key_set),
            user_public_keys: RwLock::new(HashSet::new()),
            credentials_key: RwLock::new(None),
        }
    }

    
    /// Initialize the security system - automatically uses system keyring or Docker env vars
    pub fn initialize(&self) -> Result<(), PluginError> {
        info!("Initializing plugin security system");

        // Get or create the encryption key
        let mut key = Self::get_or_create_encryption_key()?;

        // Store the key for future operations
        {
            let mut credentials_key = self.credentials_key.write()
                .map_err(|e| PluginError::SecurityError(format!("Failed to acquire key lock: {}", e)))?;
            *credentials_key = Some(key.clone());
        }

        // Load encrypted trust list if it exists
        if let Some(ref trust_list_path) = self.policy.trust_list_path {
            if trust_list_path.exists() {
                match self.load_user_trust_list(&key, trust_list_path) {
                    Ok(count) => {
                        info!("Loaded {} user-trusted plugin hashes", count);
                    }
                    Err(e) => {
                        warn!("Failed to load user trust list: {}. Starting with empty list.", e);
                    }
                }
            } else {
                info!("No existing user trust list found at {:?}", trust_list_path);
            }
        }

        // Zeroize the key from memory
        key.zeroize();

        Ok(())
    }

    /// Get or create the encryption key from system keyring or environment variables
    fn get_or_create_encryption_key() -> Result<Vec<u8>, PluginError> {
        // Detect if running in Docker
        let is_docker = Self::is_running_in_docker();

        if is_docker {
            info!("Docker environment detected, using environment variable for encryption key");
            return Self::get_key_from_env();
        }

        // Try to use system keyring first
        match Self::get_key_from_keyring() {
            Ok(key) => {
                info!("Using encryption key from system keyring");
                Ok(key)
            }
            Err(e) => {
                warn!("Failed to access system keyring: {}. Attempting to create new key.", e);

                // Create a new key and store it in the keyring
                match Self::create_and_store_key() {
                    Ok(key) => {
                        info!("Created and stored new encryption key in system keyring");
                        Ok(key)
                    }
                    Err(e) => {
                        warn!("Failed to store key in keyring: {}. Falling back to environment variable.", e);
                        Self::get_key_from_env()
                    }
                }
            }
        }
    }

    /// Check if running in a Docker container
    fn is_running_in_docker() -> bool {
        // Check for /.dockerenv file
        if std::path::Path::new("/.dockerenv").exists() {
            return true;
        }

        // Check cgroup for docker
        if let Ok(contents) = std::fs::read_to_string("/proc/self/cgroup") {
            if contents.contains("docker") || contents.contains("containerd") {
                return true;
            }
        }

        // Check for DOCKER_CONTAINER env var (custom indicator)
        std::env::var("DOCKER_CONTAINER").is_ok()
    }

    /// Get encryption key from system keyring
    fn get_key_from_keyring() -> Result<Vec<u8>, PluginError> {
        use keyring::Entry;

        let username = whoami::username();
        let service_name = "orbis-assets-plugin-security";

        let entry = Entry::new(service_name, &username)
            .map_err(|e| PluginError::SecurityError(format!("Failed to create keyring entry: {}", e)))?;

        let key_b64 = entry.get_password()
            .map_err(|e| PluginError::SecurityError(format!("Failed to retrieve key from keyring: {}", e)))?;

        BASE64.decode(key_b64.as_bytes())
            .map_err(|e| PluginError::SecurityError(format!("Failed to decode key from keyring: {}", e)))
    }

    /// Create a new encryption key and store it in the system keyring
    fn create_and_store_key() -> Result<Vec<u8>, PluginError> {
        use keyring::Entry;
        use sha3::Sha3_256;

        let username = whoami::username();
        let service_name = "orbis-assets-plugin-security";

        // Generate a random 32-byte key using system-specific data
        let mut hasher = Sha3_256::new();
        hasher.update(username.as_bytes());
        hasher.update(&std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .to_le_bytes());

        // Add some random bytes from the OS
        let mut random_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut random_bytes);
        hasher.update(&random_bytes);

        let key = hasher.finalize().to_vec();

        // Store in keyring
        let entry = Entry::new(service_name, &username)
            .map_err(|e| PluginError::SecurityError(format!("Failed to create keyring entry: {}", e)))?;

        let key_b64 = BASE64.encode(&key);
        entry.set_password(&key_b64)
            .map_err(|e| PluginError::SecurityError(format!("Failed to store key in keyring: {}", e)))?;

        Ok(key)
    }

    /// Get encryption key from environment variable (Docker fallback)
    fn get_key_from_env() -> Result<Vec<u8>, PluginError> {
        // Try to get from environment variable
        if let Ok(key_b64) = std::env::var("ORBIS_PLUGIN_ENCRYPTION_KEY") {
            info!("Using encryption key from ORBIS_PLUGIN_ENCRYPTION_KEY environment variable");
            return BASE64.decode(key_b64.as_bytes())
                .map_err(|e| PluginError::SecurityError(format!("Invalid base64 in ORBIS_PLUGIN_ENCRYPTION_KEY: {}", e)));
        }

        // If not set, generate one and warn
        warn!("No ORBIS_PLUGIN_ENCRYPTION_KEY environment variable set. Generating a temporary key.");
        warn!("WARNING: This key will not persist across container restarts!");
        warn!("Set ORBIS_PLUGIN_ENCRYPTION_KEY to a base64-encoded 32-byte key for persistence.");

        // Generate a temporary key
        use sha3::Sha3_256;
        let mut hasher = Sha3_256::new();
        hasher.update(whoami::username().as_bytes());
        hasher.update(b"orbis-assets-temp-key");
        hasher.update(&std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .to_le_bytes());

        Ok(hasher.finalize().to_vec())
    }

    /// Calculate the SHA3-512 hash of a plugin file
    pub fn calculate_hash<P: AsRef<Path>>(&self, path: P) -> Result<String, PluginError> {
        let contents = fs::read(path.as_ref())
            .map_err(|e| PluginError::SecurityError(format!("Failed to read plugin file: {}", e)))?;

        let mut hasher = Sha3_512::new();
        hasher.update(&contents);
        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Check if a plugin hash is trusted (either hardcoded or user-added)
    pub fn is_trusted_hash(&self, hash: &str) -> Result<bool, PluginError> {
        // Check hardcoded plugins first
        {
            let hardcoded = self.hardcoded_trusted_plugins.read()
                .map_err(|e| PluginError::SecurityError(format!("Failed to acquire hardcoded lock: {}", e)))?;
            if hardcoded.contains_key(hash) {
                return Ok(true);
            }
        }

        // Check user trusted plugins
        let user_plugins = self.user_trusted_plugins.read()
            .map_err(|e| PluginError::SecurityError(format!("Failed to acquire user lock: {}", e)))?;

        Ok(user_plugins.contains_key(hash))
    }

    /// Get plugin info by hash
    pub fn get_plugin_info(&self, hash: &str) -> Result<Option<TrustedPluginEntry>, PluginError> {
        // Check hardcoded first
        {
            let hardcoded = self.hardcoded_trusted_plugins.read()
                .map_err(|e| PluginError::SecurityError(format!("Failed to acquire hardcoded lock: {}", e)))?;
            if let Some(entry) = hardcoded.get(hash) {
                return Ok(Some(entry.clone()));
            }
        }

        // Check user plugins
        let user_plugins = self.user_trusted_plugins.read()
            .map_err(|e| PluginError::SecurityError(format!("Failed to acquire user lock: {}", e)))?;

        Ok(user_plugins.get(hash).cloned())
    }

    /// Add a plugin to the user's trusted list with version info and save to disk
    pub fn add_trusted_plugin(&self, entry: TrustedPluginEntry) -> Result<(), PluginError> {
        info!("Adding plugin to user trust list: {} v{}", entry.hash, entry.version);

        // Add to in-memory list
        {
            let mut user_plugins = self.user_trusted_plugins.write()
                .map_err(|e| PluginError::SecurityError(format!("Failed to acquire plugin lock: {}", e)))?;
            user_plugins.insert(entry.hash.clone(), entry);
        }

        // Save to disk
        self.save_user_trust_list()?;

        Ok(())
    }

    /// Add a hash to the user's trusted list with signature (required)
    pub fn add_trusted_hash(
        &self,
        hash: String,
        version: PluginVersion,
        signature: PluginSignature,
    ) -> Result<(), PluginError> {
        let entry = TrustedPluginEntry {
            hash,
            version,
            signature,
            note: None,
        };
        self.add_trusted_plugin(entry)
    }

    /// Remove a plugin from the user's trusted list and save to disk
    pub fn remove_trusted_hash(&self, hash: &str) -> Result<(), PluginError> {
        info!("Removing plugin from user trust list: {}", hash);

        {
            let mut user_plugins = self.user_trusted_plugins.write()
                .map_err(|e| PluginError::SecurityError(format!("Failed to acquire plugin lock: {}", e)))?;
            user_plugins.remove(hash);
        }

        self.save_user_trust_list()?;

        Ok(())
    }

    /// Validate a plugin before loading - ONLY trusted plugins are allowed
    pub fn validate_plugin(
        &self,
        _plugin_name: &str,  // Name is ignored, we only check hash
        library_path: &Path,
        _trust_level: TrustLevel,  // Ignored, we determine trust by hash
    ) -> Result<(), PluginError> {
        // Calculate the plugin hash
        let hash = self.calculate_hash(library_path)?;

        // Check if the hash is trusted
        if !self.is_trusted_hash(&hash)? {
            error!("Plugin not trusted: hash not in trust list");
            return Err(PluginError::UntrustedPlugin);
        }

        // Get the trusted entry (which contains the signature)
        let entry = self.get_plugin_info(&hash)?
            .ok_or_else(|| PluginError::SecurityError("Plugin in trust list but entry not found".to_string()))?;

        // Verify the signature
        info!("Verifying plugin signature with public key: {}", entry.signature.public_key().to_hex());

        // Check if the public key is allowed
        let is_key_allowed = self.is_public_key_allowed(entry.signature.public_key())?;
        if !is_key_allowed {
            error!("Plugin signature uses unknown public key: {}", entry.signature.public_key().to_hex());
            return Err(PluginError::SignatureError("Public key not in allowed list".to_string()));
        }

        // Read plugin file and verify signature
        let plugin_content = fs::read(library_path)?;
        let signature_valid = entry.signature.verify(&plugin_content)?;

        if !signature_valid {
            error!("Plugin signature verification failed for: {}", library_path.display());
            return Err(PluginError::SignatureError("Invalid signature".to_string()));
        }

        info!("Plugin validated successfully (hash + signature): {}", library_path.display());
        Ok(())
    }

    /// Check if a public key is in the allowed list (hardcoded or user-added)
    pub fn is_public_key_allowed(&self, key: &PublicKey) -> Result<bool, PluginError> {
        // Check hardcoded keys
        {
            let hardcoded = self.hardcoded_public_keys.read()
                .map_err(|e| PluginError::SecurityError(format!("Failed to acquire hardcoded keys lock: {}", e)))?;
            if hardcoded.contains(key) {
                return Ok(true);
            }
        }

        // Check user keys
        let user_keys = self.user_public_keys.read()
            .map_err(|e| PluginError::SecurityError(format!("Failed to acquire user keys lock: {}", e)))?;

        Ok(user_keys.contains(key))
    }

    /// Add a public key to the user's allowed list
    pub fn add_public_key(&self, key: PublicKey) -> Result<(), PluginError> {
        info!("Adding public key to user allowed list: {}", key.to_hex());

        {
            let mut user_keys = self.user_public_keys.write()
                .map_err(|e| PluginError::SecurityError(format!("Failed to acquire user keys lock: {}", e)))?;
            user_keys.insert(key);
        }

        // Save to disk
        self.save_user_trust_list()?;

        Ok(())
    }

    /// Remove a public key from the user's allowed list
    pub fn remove_public_key(&self, key: &PublicKey) -> Result<(), PluginError> {
        info!("Removing public key from user allowed list: {}", key.to_hex());

        {
            let mut user_keys = self.user_public_keys.write()
                .map_err(|e| PluginError::SecurityError(format!("Failed to acquire user keys lock: {}", e)))?;
            user_keys.remove(key);
        }

        self.save_user_trust_list()?;

        Ok(())
    }

    /// Load user trust list from encrypted binary file
    fn load_user_trust_list(&self, key: &[u8], path: &Path) -> Result<usize, PluginError> {
        // Read encrypted binary file
        let encrypted_data = fs::read(path)
            .map_err(|e| PluginError::IoError(e))?;

        // Binary format: [24-byte nonce][encrypted data]
        if encrypted_data.len() < 24 {
            return Err(PluginError::SecurityError("Invalid encrypted file format".to_string()));
        }

        // Extract nonce and ciphertext
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(24);
        let nonce = XNonce::from_slice(nonce_bytes);

        // Decrypt using XChaCha20-Poly1305
        let cipher = XChaCha20Poly1305::new_from_slice(key)
            .map_err(|e| PluginError::SecurityError(format!("Failed to create cipher: {}", e)))?;

        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| PluginError::SecurityError(format!("Failed to decrypt trust list (invalid key?): {}", e)))?;

        // Deserialize using bincode 2.0
        let (trust_list, _len): (UserTrustList, usize) = bincode::decode_from_slice(&plaintext, bincode::config::standard())
            .map_err(|e| PluginError::SecurityError(format!("Failed to deserialize trust list: {}", e)))?;

        let count = trust_list.trusted_plugins.len();

        // Update in-memory trust list
        {
            let mut user_plugins = self.user_trusted_plugins.write()
                .map_err(|e| PluginError::SecurityError(format!("Failed to acquire plugin lock: {}", e)))?;
            *user_plugins = trust_list.trusted_plugins;
        }

        // Update in-memory public keys
        {
            let mut user_keys = self.user_public_keys.write()
                .map_err(|e| PluginError::SecurityError(format!("Failed to acquire keys lock: {}", e)))?;
            *user_keys = trust_list.user_public_keys;
        }

        Ok(count)
    }

    /// Save user trust list to encrypted binary file
    fn save_user_trust_list(&self) -> Result<(), PluginError> {
        let trust_list_path = self.policy.trust_list_path.as_ref()
            .ok_or_else(|| PluginError::SecurityError("No trust list path configured".to_string()))?;

        // Get the encryption key
        let key = {
            let credentials_key = self.credentials_key.read()
                .map_err(|e| PluginError::SecurityError(format!("Failed to acquire key lock: {}", e)))?;
            credentials_key.clone()
                .ok_or_else(|| PluginError::SecurityError("No credentials initialized".to_string()))?
        };

        // Get user plugins and keys
        let trust_list = {
            let user_plugins = self.user_trusted_plugins.read()
                .map_err(|e| PluginError::SecurityError(format!("Failed to acquire plugin lock: {}", e)))?;
            let user_keys = self.user_public_keys.read()
                .map_err(|e| PluginError::SecurityError(format!("Failed to acquire keys lock: {}", e)))?;
            UserTrustList {
                trusted_plugins: user_plugins.clone(),
                user_public_keys: user_keys.clone(),
                format_version: 2, // Increment version for public keys support
            }
        };

        // Serialize using bincode 2.0 (binary format)
        let plaintext = bincode::encode_to_vec(&trust_list, bincode::config::standard())
            .map_err(|e| PluginError::SecurityError(format!("Failed to serialize trust list: {}", e)))?;

        // Generate 24-byte nonce for XChaCha20-Poly1305
        use chacha20poly1305::aead::AeadCore;
        let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);

        // Encrypt using XChaCha20-Poly1305
        let cipher = XChaCha20Poly1305::new_from_slice(&key)
            .map_err(|e| PluginError::SecurityError(format!("Failed to create cipher: {}", e)))?;

        let ciphertext = cipher.encrypt(&nonce, plaintext.as_ref())
            .map_err(|e| PluginError::SecurityError(format!("Failed to encrypt trust list: {}", e)))?;

        // Create binary blob: [24-byte nonce][encrypted data]
        let mut encrypted_data = Vec::with_capacity(24 + ciphertext.len());
        encrypted_data.extend_from_slice(&nonce);
        encrypted_data.extend_from_slice(&ciphertext);

        // Create parent directory if it doesn't exist
        if let Some(parent) = trust_list_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write binary file
        fs::write(trust_list_path, encrypted_data)?;

        info!("Saved encrypted trust list to {:?}", trust_list_path);
        Ok(())
    }

    /// Get the current security policy
    pub fn policy(&self) -> &SecurityPolicy {
        &self.policy
    }

    /// Get the number of hardcoded trusted plugins
    pub fn hardcoded_trust_count(&self) -> Result<usize, PluginError> {
        let hardcoded = self.hardcoded_trusted_plugins.read()
            .map_err(|e| PluginError::SecurityError(format!("Failed to acquire hardcoded lock: {}", e)))?;
        Ok(hardcoded.len())
    }

    /// Get the number of user trusted plugins
    pub fn user_trust_count(&self) -> Result<usize, PluginError> {
        let user_plugins = self.user_trusted_plugins.read()
            .map_err(|e| PluginError::SecurityError(format!("Failed to acquire plugin lock: {}", e)))?;
        Ok(user_plugins.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_level_ordering() {
        assert!(TrustLevel::Untrusted < TrustLevel::Trusted);
    }

    #[test]
    fn test_hash_calculation() {
        let security = PluginSecurity::new(SecurityPolicy::default(), vec![], vec![]);
        // This will fail without a real file, but tests the interface
        assert!(security.calculate_hash("/nonexistent").is_err());
    }

    #[test]
    fn test_hardcoded_trust() {
        let test_hash = "abc123".to_string();
        let dummy_key = PublicKey::from_bytes([0u8; 32]);
        let dummy_signature = PluginSignature::from_bytes([0u8; 64], dummy_key.clone());
        
        let entry = TrustedPluginEntry {
            hash: test_hash.clone(),
            version: PluginVersion::new(1, 0, 0),
            signature: dummy_signature,
            note: Some("Test plugin".to_string()),
        };
        let security = PluginSecurity::new(
            SecurityPolicy::default(),
            vec![entry],
            vec![dummy_key]
        );
        assert!(security.is_trusted_hash(&test_hash).unwrap());
        assert!(!security.is_trusted_hash("other_hash").unwrap());
    }

    #[test]
    fn test_plugin_version() {
        let version = PluginVersion::new(1, 2, 3);
        assert_eq!(version.to_string(), "1.2.3");

        let parsed = PluginVersion::from_string("1.2.3").unwrap();
        assert_eq!(parsed, version);
    }
}

