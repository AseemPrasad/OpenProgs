use crate::audit::keys::Ed25519KeyManager;
use crate::audit::persist::AuditPersist;
use ed25519_dalek::Signature;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::path::PathBuf;
use std::sync::Arc;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: u64,
    pub event_type: String,
    pub actor: String,
    pub details: serde_json::Value,
    pub previous_hash: Vec<u8>,
    pub entry_hash: Vec<u8>,
    pub signature: Vec<u8>,
}

pub struct AuditLedger {
    entries: Vec<AuditEntry>,
    key_manager: Arc<Ed25519KeyManager>,
    persistence: Option<AuditPersist>,
}

impl AuditLedger {
    pub fn new(
        key_manager: Arc<Ed25519KeyManager>,
        ledger_path: Option<PathBuf>,
    ) -> Result<Self> {
        let persistence = if let Some(path) = ledger_path {
            Some(AuditPersist::load_or_create(path)?)
        } else {
            None
        };

        Ok(Self {
            entries: Vec::new(),
            key_manager,
            persistence,
        })
    }

    pub fn append_entry(
        &mut self,
        event_type: impl Into<String>,
        actor: impl Into<String>,
        details: serde_json::Value,
    ) -> Result<AuditEntry> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos() as u64;

        let event_type_str = event_type.into();
        let actor_str = actor.into();

        let previous_hash = self
            .entries
            .last()
            .map(|e| e.entry_hash.clone())
            .unwrap_or_else(Vec::new);

        let entry_data = format!(
            "{:x}:{}:{}:{}",
            timestamp,
            event_type_str,
            actor_str,
            serde_json::to_string(&details)?
        );

        let mut hasher = Sha256::new();
        hasher.update(&previous_hash);
        hasher.update(entry_data.as_bytes());
        let entry_hash = hasher.finalize().to_vec();

        let signature = self.key_manager.sign(&entry_hash);
        let signature_bytes = signature.to_bytes().to_vec();

        let entry = AuditEntry {
            timestamp,
            event_type: event_type_str,
            actor: actor_str,
            details,
            previous_hash,
            entry_hash,
            signature: signature_bytes,
        };

        self.entries.push(entry.clone());

        // Persist if enabled
        if let Some(persist) = &mut self.persistence {
            persist.append_entry(&entry)?;
        }

        Ok(entry)
    }

    pub fn verify_chain(&self) -> Result<()> {
        if self.entries.is_empty() {
            return Ok(());
        }

        // Verify first entry
        if !self.entries[0].previous_hash.is_empty() {
            return Err(anyhow::anyhow!("First entry has non-empty previous hash"));
        }

        for i in 0..self.entries.len() {
            let entry = &self.entries[i];

            // Recompute the entry hash
            let entry_data = format!(
                "{:x}:{}:{}:{}",
                entry.timestamp,
                entry.event_type,
                entry.actor,
                serde_json::to_string(&entry.details)?
            );

            let mut hasher = Sha256::new();
            hasher.update(&entry.previous_hash);
            hasher.update(entry_data.as_bytes());
            let recomputed_hash = hasher.finalize().to_vec();

            if recomputed_hash != entry.entry_hash {
                return Err(anyhow::anyhow!(
                    "Entry {} hash mismatch: computed {:?} but stored {:?}",
                    i,
                    hex::encode(&recomputed_hash),
                    hex::encode(&entry.entry_hash)
                ));
            }

            // Verify signature
            let signature_bytes: [u8; 64] = entry.signature[..]
                .try_into()
                .map_err(|_| anyhow::anyhow!("Invalid signature length at entry {}", i))?;
            let signature = Signature::from_bytes(&signature_bytes);

            self.key_manager
                .verify(&entry.entry_hash, &signature)
                .map_err(|e| {
                    anyhow::anyhow!("Signature verification failed at entry {}: {}", i, e)
                })?;

            // Check hash chain continuity
            if i > 0 {
                let prev_entry = &self.entries[i - 1];
                if entry.previous_hash != prev_entry.entry_hash {
                    return Err(anyhow::anyhow!(
                        "Hash chain broken at entry {}: previous hash doesn't match",
                        i
                    ));
                }
            }
        }

        Ok(())
    }

    pub fn get_entries_range(&self, start_ts: u64, end_ts: u64) -> Vec<AuditEntry> {
        self.entries
            .iter()
            .filter(|e| e.timestamp >= start_ts && e.timestamp <= end_ts)
            .cloned()
            .collect()
    }

    pub fn get_entries(&self) -> &[AuditEntry] {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn export_merkle_root(&self) -> (Vec<u8>, usize) {
        let merkle_root = self
            .entries
            .last()
            .map(|e| e.entry_hash.clone())
            .unwrap_or_else(Vec::new);
        (merkle_root, self.entries.len())
    }

    pub fn flush(&mut self) -> Result<()> {
        if let Some(persist) = &mut self.persistence {
            persist.flush()?;
        }
        Ok(())
    }
}
