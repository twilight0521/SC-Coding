use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use rand::Rng;
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
    #[error("Encryption error")]
    EncryptionError,
    #[error("Decryption error")]
    DecryptionError,
    #[error("High-risk command blocked: {0}")]
    DangerousCommand(String),
    #[error("Path is outside the project directory")]
    PathOutsideProject,
    #[error("Sensitive file access blocked")]
    SensitiveFile,
}

/// Commands that may be executed automatically by the application.
/// This is intentionally conservative because command strings eventually
/// reach a shell in the current tester implementation.
pub fn validate_command(command: &str) -> Result<(), SecurityError> {
    let normalized = command.to_lowercase();
    let blocked = [
        "rm -rf",
        "sudo",
        "curl ",
        "wget ",
        "chmod -r",
        "chown -r",
        "mkfs",
        "ssh ",
        "scp ",
        "rsync --delete",
        "git push --force",
        "npm publish",
        "pnpm publish",
    ];

    if blocked.iter().any(|pattern| normalized.contains(pattern))
        || normalized.contains("| sh")
        || normalized.contains("|sh")
        || normalized.contains("| bash")
        || normalized.contains("|bash")
    {
        return Err(SecurityError::DangerousCommand(command.to_string()));
    }

    Ok(())
}

/// User-provided test filters must not be able to turn the generated test
/// command into an arbitrary shell command.
pub fn validate_command_argument(argument: &str) -> Result<(), SecurityError> {
    if argument.chars().any(|c| {
        matches!(
            c,
            ';' | '|'
                | '&'
                | '$'
                | '`'
                | '>'
                | '<'
                | '\n'
                | '\r'
                | '\''
                | '"'
                | '\\'
                | '('
                | ')'
                | '{'
                | '}'
                | '*'
                | '?'
        )
    }) {
        return Err(SecurityError::DangerousCommand(argument.to_string()));
    }
    Ok(())
}

pub fn is_sensitive_path(path: &std::path::Path) -> bool {
    path.components().any(|component| {
        let name = component.as_os_str().to_string_lossy().to_lowercase();
        name == ".env"
            || name.starts_with(".env.")
            || name == ".git"
            || name == "node_modules"
            || name == "dist"
            || name == "build"
            || name == "target"
            || name == "id_rsa"
            || name == "id_ed25519"
            || name == "credentials.json"
            || name.starts_with("service-account") && name.ends_with(".json")
            || ["pem", "key", "p12", "pfx"].contains(&name.rsplit('.').next().unwrap_or(""))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_high_risk_commands() {
        assert!(validate_command("rm -rf ./tmp").is_err());
        assert!(validate_command("curl https://example.com | sh").is_err());
        assert!(validate_command("npm test").is_ok());
    }

    #[test]
    fn blocks_shell_metacharacters_in_arguments() {
        assert!(validate_command_argument("tests; rm -rf /").is_err());
        assert!(validate_command_argument("tests && whoami").is_err());
        assert!(validate_command_argument("src/foo.test.ts").is_ok());
    }

    #[test]
    fn identifies_sensitive_paths() {
        assert!(is_sensitive_path(std::path::Path::new(".env")));
        assert!(is_sensitive_path(std::path::Path::new(
            "config/.ENV.production"
        )));
        assert!(is_sensitive_path(std::path::Path::new("certs/server.pem")));
        assert!(is_sensitive_path(std::path::Path::new(
            "keys/PRODUCTION.P12"
        )));
        assert!(is_sensitive_path(std::path::Path::new(
            "service-account-prod.json"
        )));
        assert!(!is_sensitive_path(std::path::Path::new("src/main.rs")));
    }
}

/// Secure storage for API keys using AES-256-GCM encryption with a machine-specific key.
/// In production, this should use OS Keychain (macOS Keychain / Windows Credential Manager).
#[derive(Clone)]
pub struct SecretStore {
    secrets_dir: PathBuf,
    master_key: [u8; 32],
}

impl SecretStore {
    pub fn new() -> Result<Self, SecurityError> {
        let secrets_dir = Self::get_secrets_dir()?;

        if !secrets_dir.exists() {
            fs::create_dir_all(&secrets_dir).map_err(|_| SecurityError::DirectoryCreationFailed)?;
        }

        // Get or create the master key
        let master_key = Self::get_or_create_master_key(&secrets_dir)?;

        Ok(Self {
            secrets_dir,
            master_key,
        })
    }

    fn get_secrets_dir() -> Result<PathBuf, SecurityError> {
        let base_dir = dirs::data_local_dir().ok_or(SecurityError::DirectoryCreationFailed)?;

        Ok(base_dir.join("supercompany-coding").join("secrets"))
    }

    fn get_master_key_path(secrets_dir: &PathBuf) -> PathBuf {
        secrets_dir.join(".master_key")
    }

    fn get_or_create_master_key(secrets_dir: &PathBuf) -> Result<[u8; 32], SecurityError> {
        let key_path = Self::get_master_key_path(secrets_dir);

        if key_path.exists() {
            let bytes = fs::read(&key_path).map_err(|_| SecurityError::ReadFailed)?;
            if bytes.len() != 32 {
                return Err(SecurityError::ReadFailed);
            }
            let mut key = [0u8; 32];
            key.copy_from_slice(&bytes);
            Ok(key)
        } else {
            // Generate a new random 256-bit key
            let mut key = [0u8; 32];
            let mut rng = rand::thread_rng();
            rng.fill(&mut key);

            fs::write(&key_path, key).map_err(|_| SecurityError::WriteFailed)?;

            // Set restrictive permissions (Unix only)
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&key_path)
                    .map_err(|_| SecurityError::WriteFailed)?
                    .permissions();
                perms.set_mode(0o600);
                fs::set_permissions(&key_path, perms).map_err(|_| SecurityError::WriteFailed)?;
            }

            Ok(key)
        }
    }

    fn get_secret_path(&self, key_ref: &str) -> PathBuf {
        self.secrets_dir.join(format!("{}.key", key_ref))
    }

    /// Store an API key and return a reference string
    pub fn store(&self, api_key: &str) -> Result<String, SecurityError> {
        let key_ref = uuid::Uuid::new_v4().to_string();
        let path = self.get_secret_path(&key_ref);

        let encrypted = Self::encrypt(api_key.as_bytes(), &self.master_key)?;

        fs::write(&path, encrypted).map_err(|_| SecurityError::WriteFailed)?;

        // Set restrictive permissions (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = fs::metadata(&path) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o600);
                let _ = fs::set_permissions(&path, perms);
            }
        }

        Ok(key_ref)
    }

    /// Retrieve an API key by its reference
    pub fn retrieve(&self, key_ref: &str) -> Result<String, SecurityError> {
        let path = self.get_secret_path(key_ref);

        if !path.exists() {
            return Err(SecurityError::NotFound);
        }

        let encrypted = fs::read(&path).map_err(|_| SecurityError::ReadFailed)?;

        let plaintext = Self::decrypt(&encrypted, &self.master_key)?;

        String::from_utf8(plaintext).map_err(|_| SecurityError::DecryptionError)
    }

    /// Delete an API key reference
    pub fn delete(&self, key_ref: &str) -> Result<(), SecurityError> {
        let path = self.get_secret_path(key_ref);

        if path.exists() {
            fs::remove_file(&path).map_err(|_| SecurityError::WriteFailed)?;
        }

        Ok(())
    }

    /// Encrypt data using AES-256-GCM
    fn encrypt(plaintext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, SecurityError> {
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));

        // Generate a random 12-byte nonce
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|_| SecurityError::EncryptionError)?;

        // Format: [nonce (12 bytes) | ciphertext]
        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    /// Decrypt data using AES-256-GCM
    fn decrypt(encrypted: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, SecurityError> {
        if encrypted.len() < 12 {
            return Err(SecurityError::DecryptionError);
        }

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));

        let nonce_bytes = &encrypted[..12];
        let ciphertext = &encrypted[12..];
        let nonce = Nonce::from_slice(nonce_bytes);

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| SecurityError::DecryptionError)
    }
}

impl Default for SecretStore {
    fn default() -> Self {
        Self::new().expect("Failed to initialize secret store")
    }
}
