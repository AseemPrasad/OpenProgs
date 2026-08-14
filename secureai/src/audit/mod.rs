pub mod keys;
pub mod ledger;
pub mod persist;
pub mod hooks;

pub use keys::{Ed25519KeyManager, KeyConfig};
pub use ledger::{AuditLedger, AuditEntry};
pub use persist::AuditPersist;
pub use hooks::{AuditHooks, GlobalAuditHooks, AuditLedgerRef};

use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct AuditConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default = "default_ledger_path")]
    pub ledger_path: PathBuf,

    #[serde(default = "default_key_path")]
    pub key_path: PathBuf,

    #[serde(default = "default_persistence")]
    pub persistence_enabled: bool,

    #[serde(default = "default_auto_generate")]
    pub auto_generate_keys: bool,
}

fn default_ledger_path() -> PathBuf {
    PathBuf::from("/var/log/secureai/audit.jsonl")
}

fn default_key_path() -> PathBuf {
    PathBuf::from("/etc/secureai/audit.key")
}

fn default_persistence() -> bool {
    true
}

fn default_auto_generate() -> bool {
    true
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            ledger_path: default_ledger_path(),
            key_path: default_key_path(),
            persistence_enabled: default_persistence(),
            auto_generate_keys: default_auto_generate(),
        }
    }
}
