use crate::audit::ledger::AuditEntry;
use sha2::{Sha256, Digest};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write, BufRead, BufReader};
use std::path::PathBuf;
use anyhow::{Result, Context};

pub struct AuditPersist {
    ledger_path: PathBuf,
    buffer: Vec<AuditEntry>,
    buffer_size_threshold: usize,
    writer: BufWriter<File>,
}

impl AuditPersist {
    pub fn load_or_create(path: PathBuf) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .context(format!("Failed to create parent directory for {:?}", path))?;
        }

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .context(format!("Failed to open ledger file at {:?}", path))?;

        let writer = BufWriter::new(file);

        Ok(Self {
            ledger_path: path,
            buffer: Vec::new(),
            buffer_size_threshold: 128,
            writer,
        })
    }

    pub fn append_entry(&mut self, entry: &AuditEntry) -> Result<()> {
        self.buffer.push(entry.clone());

        if self.buffer.len() >= self.buffer_size_threshold {
            self.flush()?;
        }

        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        for entry in self.buffer.drain(..) {
            let json = serde_json::to_string(&entry)?;
            writeln!(self.writer, "{}", json)?;
        }

        self.writer.flush()?;
        Ok(())
    }

    pub fn load_ledger(path: &PathBuf) -> Result<Vec<AuditEntry>> {
        if !path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(path)
            .context(format!("Failed to open ledger file at {:?}", path))?;
        let reader = BufReader::new(file);

        let mut entries = Vec::new();
        for (line_num, line) in reader.lines().enumerate() {
            let line = line.context(format!("Failed to read line {} from ledger", line_num))?;
            if line.trim().is_empty() {
                continue;
            }

            let entry: AuditEntry = serde_json::from_str(&line)
                .context(format!("Failed to parse entry at line {}", line_num))?;
            entries.push(entry);
        }

        Ok(entries)
    }

    pub fn get_file_checksum(path: &PathBuf) -> Result<Vec<u8>> {
        if !path.exists() {
            return Ok(Vec::new());
        }

        let contents = std::fs::read(path)
            .context(format!("Failed to read ledger file at {:?}", path))?;

        let mut hasher = Sha256::new();
        hasher.update(contents);
        Ok(hasher.finalize().to_vec())
    }

    pub fn verify_integrity(path: &PathBuf, stored_checksum: &[u8]) -> Result<bool> {
        let current_checksum = Self::get_file_checksum(path)?;
        Ok(current_checksum == stored_checksum)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persist_create() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test_ledger.jsonl");

        let persist = AuditPersist::load_or_create(path.clone());
        assert!(persist.is_ok());
        assert!(path.exists());
    }

    #[test]
    fn test_file_checksum() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test_ledger.jsonl");

        std::fs::write(&path, "test data").unwrap();

        let checksum1 = AuditPersist::get_file_checksum(&path).unwrap();
        let checksum2 = AuditPersist::get_file_checksum(&path).unwrap();

        assert_eq!(checksum1, checksum2);
    }
}
