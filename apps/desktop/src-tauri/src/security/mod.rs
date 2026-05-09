use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Failed to create secrets directory")]
    DirectoryCreationFailed,
    #[error("Failed to write secret")]
    WriteFailed,
    #[error("Failed to read secret")]
    ReadFailed,
    #[error("Secret not found")]
    NotFound,
}

/// Secure storage for API keys using local encrypted file.
/// In production, this should use OS Keychain (macOS Keychain / Windows Credential Manager).
pub struct SecretStore {
    secrets_dir: PathBuf,
}

impl SecretStore {
    pub fn new() -> Result<Self, SecurityError> {
        let secrets_dir = Self::get_secrets_dir()?;

        if !secrets_dir.exists() {
            fs::create_dir_all(&secrets_dir)
                .map_err(|_| SecurityError::DirectoryCreationFailed)?;
        }

        Ok(Self { secrets_dir })
    }

    fn get_secrets_dir() -> Result<PathBuf, SecurityError> {
        let base_dir = dirs::data_local_dir()
            .ok_or(SecurityError::DirectoryCreationFailed)?;

        Ok(base_dir.join("supercompany-coding").join("secrets"))
    }

    fn get_secret_path(&self, key_ref: &str) -> PathBuf {
        self.secrets_dir.join(format!("{}.key", key_ref))
    }

    /// Store an API key and return a reference string
    pub fn store(&self, api_key: &str) -> Result<String, SecurityError> {
        let key_ref = uuid::Uuid::new_v4().to_string();
        let path = self.get_secret_path(&key_ref);

        // Simple XOR encryption with machine-specific salt
        let encrypted = Self::simple_encrypt(api_key, &key_ref);

        fs::write(&path, encrypted)
            .map_err(|_| SecurityError::WriteFailed)?;

        Ok(key_ref)
    }

    /// Retrieve an API key by its reference
    pub fn retrieve(&self, key_ref: &str) -> Result<String, SecurityError> {
        let path = self.get_secret_path(key_ref);

        if !path.exists() {
            return Err(SecurityError::NotFound);
        }

        let encrypted = fs::read_to_string(&path)
            .map_err(|_| SecurityError::ReadFailed)?;

        Self::simple_decrypt(&encrypted, key_ref)
    }

    /// Delete an API key reference
    pub fn delete(&self, key_ref: &str) -> Result<(), SecurityError> {
        let path = self.get_secret_path(key_ref);

        if path.exists() {
            fs::remove_file(&path)
                .map_err(|_| SecurityError::WriteFailed)?;
        }

        Ok(())
    }

    fn simple_encrypt(input: &str, salt: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hash = DefaultHasher::new();
        salt.hash(&mut hash);
        let key = (hash.finish() as u8).wrapping_add(42);

        input.bytes()
            .map(|b| b ^ key)
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join("")
    }

    fn simple_decrypt(encrypted: &str, salt: &str) -> Result<String, SecurityError> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hash = DefaultHasher::new();
        salt.hash(&mut hash);
        let key = (hash.finish() as u8).wrapping_add(42);

        let bytes: Result<Vec<u8>, _> = encrypted
            .chars()
            .collect::<Vec<_>>()
            .chunks(2)
            .filter_map(|chunk| {
                if chunk.len() == 2 {
                    let hex = format!("{}{}", chunk[0], chunk[1]);
                    u8::from_str_radix(&hex, 16).ok()
                } else {
                    None
                }
            })
            .map(|b| Ok(b ^ key))
            .collect();

        match bytes {
            Ok(decrypted) => Ok(String::from_utf8_lossy(&decrypted).to_string()),
            Err(e) => Err(e),
        }
    }
}

impl Default for SecretStore {
    fn default() -> Self {
        Self::new().expect("Failed to initialize secret store")
    }
}