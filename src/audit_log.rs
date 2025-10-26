//! Audit logging module for enterprise tracking and compliance

use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    /// Timestamp of the event
    pub timestamp: String,
    /// Event type
    pub event_type: String,
    /// User or system performing the action
    pub actor: String,
    /// Resource being acted upon
    pub resource: String,
    /// Action performed
    pub action: String,
    /// Result of the action
    pub result: String,
    /// Additional details
    pub details: Option<String>,
    /// IP address or source (optional)
    pub source: Option<String>,
}

/// Audit log level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditLevel {
    /// Info level
    Info,
    /// Warning level
    Warning,
    /// Error level
    Error,
    /// Critical level
    Critical,
}

/// Audit logger for tracking operations
pub struct AuditLogger {
    entries: Arc<Mutex<Vec<AuditLogEntry>>>,
    log_file: Option<String>,
    min_level: AuditLevel,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new() -> Self {
        Self {
            entries: Arc::new(Mutex::new(Vec::new())),
            log_file: None,
            min_level: AuditLevel::Info,
        }
    }

    /// Create audit logger with file output
    pub fn with_file(log_file: &str) -> Self {
        Self {
            entries: Arc::new(Mutex::new(Vec::new())),
            log_file: Some(log_file.to_string()),
            min_level: AuditLevel::Info,
        }
    }

    /// Set minimum audit level
    pub fn set_min_level(&mut self, level: AuditLevel) {
        self.min_level = level;
    }

    /// Log a repair operation
    pub fn log_repair(
        &self,
        file_path: &str,
        format: &str,
        success: bool,
        actor: &str,
        details: Option<&str>,
    ) {
        let entry = AuditLogEntry {
            timestamp: Utc::now().to_rfc3339(),
            event_type: "REPAIR".to_string(),
            actor: actor.to_string(),
            resource: file_path.to_string(),
            action: format!("Repair {}", format),
            result: if success {
                "SUCCESS".to_string()
            } else {
                "FAILURE".to_string()
            },
            details: details.map(|s| s.to_string()),
            source: None,
        };

        self.log_entry(&entry);
    }

    /// Log a validation operation
    pub fn log_validation(
        &self,
        file_path: &str,
        passed: bool,
        actor: &str,
        details: Option<&str>,
    ) {
        let entry = AuditLogEntry {
            timestamp: Utc::now().to_rfc3339(),
            event_type: "VALIDATION".to_string(),
            actor: actor.to_string(),
            resource: file_path.to_string(),
            action: "Validate".to_string(),
            result: if passed {
                "PASSED".to_string()
            } else {
                "FAILED".to_string()
            },
            details: details.map(|s| s.to_string()),
            source: None,
        };

        self.log_entry(&entry);
    }

    /// Log a batch operation
    pub fn log_batch_operation(
        &self,
        batch_id: &str,
        total_files: usize,
        successful: usize,
        failed: usize,
        actor: &str,
    ) {
        let entry = AuditLogEntry {
            timestamp: Utc::now().to_rfc3339(),
            event_type: "BATCH_OPERATION".to_string(),
            actor: actor.to_string(),
            resource: batch_id.to_string(),
            action: "Batch Process".to_string(),
            result: format!(
                "Processed {} files: {} successful, {} failed",
                total_files, successful, failed
            ),
            details: None,
            source: None,
        };

        self.log_entry(&entry);
    }

    /// Log a configuration change
    pub fn log_config_change(
        &self,
        config_name: &str,
        old_value: &str,
        new_value: &str,
        actor: &str,
    ) {
        let entry = AuditLogEntry {
            timestamp: Utc::now().to_rfc3339(),
            event_type: "CONFIG_CHANGE".to_string(),
            actor: actor.to_string(),
            resource: config_name.to_string(),
            action: "Configuration Changed".to_string(),
            result: "SUCCESS".to_string(),
            details: Some(format!("Changed from '{}' to '{}'", old_value, new_value)),
            source: None,
        };

        self.log_entry(&entry);
    }

    /// Log an entry
    pub fn log_entry(&self, entry: &AuditLogEntry) {
        let mut entries = self.entries.lock().unwrap();
        entries.push(entry.clone());

        // Write to file if configured
        if let Some(ref log_file) = self.log_file {
            let _ = self.write_to_file(entry, log_file);
        }
    }

    /// Write entry to file
    fn write_to_file(&self, entry: &AuditLogEntry, log_file: &str) -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)?;

        let json_line = serde_json::to_string(entry).unwrap_or_default();
        writeln!(file, "{}", json_line)?;
        Ok(())
    }

    /// Get all audit entries
    pub fn get_entries(&self) -> Vec<AuditLogEntry> {
        self.entries.lock().unwrap().clone()
    }

    /// Get entries by event type
    pub fn get_entries_by_type(&self, event_type: &str) -> Vec<AuditLogEntry> {
        self.entries
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.event_type == event_type)
            .cloned()
            .collect()
    }

    /// Get entries by actor
    pub fn get_entries_by_actor(&self, actor: &str) -> Vec<AuditLogEntry> {
        self.entries
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.actor == actor)
            .cloned()
            .collect()
    }

    /// Clear all entries
    pub fn clear_entries(&self) {
        self.entries.lock().unwrap().clear();
    }

    /// Get entry count
    pub fn entry_count(&self) -> usize {
        self.entries.lock().unwrap().len()
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_logger_creation() {
        let logger = AuditLogger::new();
        assert_eq!(logger.entry_count(), 0);
    }

    #[test]
    fn test_log_repair() {
        let logger = AuditLogger::new();
        logger.log_repair("test.json", "json", true, "system", Some("Test repair"));

        assert_eq!(logger.entry_count(), 1);
        let entries = logger.get_entries();
        assert_eq!(entries[0].event_type, "REPAIR");
        assert_eq!(entries[0].result, "SUCCESS");
    }

    #[test]
    fn test_log_validation() {
        let logger = AuditLogger::new();
        logger.log_validation("test.json", true, "system", None);

        assert_eq!(logger.entry_count(), 1);
        let entries = logger.get_entries();
        assert_eq!(entries[0].event_type, "VALIDATION");
        assert_eq!(entries[0].result, "PASSED");
    }

    #[test]
    fn test_log_batch_operation() {
        let logger = AuditLogger::new();
        logger.log_batch_operation("batch_001", 10, 8, 2, "system");

        assert_eq!(logger.entry_count(), 1);
        let entries = logger.get_entries();
        assert_eq!(entries[0].event_type, "BATCH_OPERATION");
    }

    #[test]
    fn test_get_entries_by_type() {
        let logger = AuditLogger::new();
        logger.log_repair("test1.json", "json", true, "system", None);
        logger.log_validation("test2.json", true, "system", None);
        logger.log_repair("test3.json", "json", false, "system", None);

        let repairs = logger.get_entries_by_type("REPAIR");
        assert_eq!(repairs.len(), 2);

        let validations = logger.get_entries_by_type("VALIDATION");
        assert_eq!(validations.len(), 1);
    }

    #[test]
    fn test_get_entries_by_actor() {
        let logger = AuditLogger::new();
        logger.log_repair("test1.json", "json", true, "user1", None);
        logger.log_repair("test2.json", "json", true, "user2", None);

        let user1_entries = logger.get_entries_by_actor("user1");
        assert_eq!(user1_entries.len(), 1);
    }

    #[test]
    fn test_clear_entries() {
        let logger = AuditLogger::new();
        logger.log_repair("test.json", "json", true, "system", None);
        assert_eq!(logger.entry_count(), 1);

        logger.clear_entries();
        assert_eq!(logger.entry_count(), 0);
    }
}
