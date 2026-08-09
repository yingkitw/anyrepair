//! Core traits for binary (media) file repair.
//!
//! These traits mirror the text-based [`Repair`](crate::traits::Repair) family
//! but operate on `&[u8]` / `Vec<u8>` instead of `&str` / `String`.

use crate::error::Result;

/// Trait for repairing binary content (images, audio, video).
pub trait BinaryRepair {
    /// Repair the given binary content and return the repaired version.
    fn repair(&mut self, data: &[u8]) -> Result<Vec<u8>>;

    /// Check if the content needs repair.
    fn needs_repair(&self, data: &[u8]) -> bool;

    /// Get the confidence score for repair (0.0 to 1.0).
    fn confidence(&self, data: &[u8]) -> f64;
}

/// Trait for binary content validation.
pub trait BinaryValidator {
    /// Validate the binary content and return true if valid.
    fn is_valid(&self, data: &[u8]) -> bool;
}
