use ed25519_dalek::{SigningKey, VerifyingKey, Signature};
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};

#[derive(Debug, Clone)]
pub struct Ed25519KeyManager {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl Ed25519KeyManager {
    pub fn generate() -> Result<Self> {
        let signing_key = SigningKey::generate(&mut rand::thread_rng());
        let verifying_key = signing_key.verifying_key();

        Ok(Self {
            signing_key,
            verifying_key,
        })
    }

    pub fn load_or_generate(key_path: &Path) -> Result<Self> {
        if key_path.exists() {
            Self::load_from_file(key_path)
        } else {
            let key_mgr = Self::generate()?;
            key_mgr.save_to_file(key_path)?;
            Ok(key_mgr)
        }
    }

    pub fn load_from_file(path: &Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .context(format!("Failed to read key file at {:?}", path))?;

        let lines: Vec<&str> = contents.lines().collect();
        if lines.len() < 2 {
            return Err(anyhow::anyhow!("Invalid key file format"));
        }

        let signing_key_bytes = hex::decode(lines[0])
            .context("Failed to decode signing key from hex")?;
        let signing_key = SigningKey::from_bytes(
            signing_key_bytes
                .as_slice()
                .try_into()
                .context("Invalid signing key length")?,
        );
        let verifying_key = signing_key.verifying_key();

        Ok(Self {
            signing_key,
            verifying_key,
        })
    }

    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .context(format!("Failed to create parent directory for {:?}", path))?;
        }

        let signing_key_hex = hex::encode(self.signing_key.to_bytes());
        let verifying_key_hex = hex::encode(self.verifying_key.to_bytes());

        let content = format!("{}\n{}\n", signing_key_hex, verifying_key_hex);
        std::fs::write(path, content).context(format!("Failed to write key file to {:?}", path))?;

        #[cfg(unix)]
        {
            use std::fs;
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o600);
            fs::set_permissions(path, perms)?;
        }

        Ok(())
    }

    pub fn sign(&self, data: &[u8]) -> Signature {
        self.signing_key.sign(data)
    }

    pub fn verify(&self, data: &[u8], signature: &Signature) -> Result<()> {
        self.verifying_key
            .verify(data, signature)
            .map_err(|e| anyhow::anyhow!("Signature verification failed: {}", e))
    }

    pub fn get_verifying_key_bytes(&self) -> [u8; 32] {
        *self.verifying_key.as_bytes()
    }
}

#[derive(Debug, Clone)]
pub struct KeyConfig {
    pub key_path: PathBuf,
    pub auto_generate: bool,
}

impl Default for KeyConfig {
    fn default() -> Self {
        Self {
            key_path: PathBuf::from("/var/lib/secureai/audit.key"),
            auto_generate: true,
        }
    }
}
