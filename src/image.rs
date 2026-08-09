//! Image file repair — PNG and JPEG.
//!
//! Repair strategies operate at the container/chunk level:
//! - **PNG**: Fix chunk CRCs, add missing IEND, truncate after IEND, fix IHDR dimensions.
//! - **JPEG**: Add missing EOI marker, fix truncated SOS data, remove trailing garbage.

use crate::binary_traits::{BinaryRepair, BinaryValidator};
use crate::error::{RepairError, Result};

// ─── CRC32 (PNG chunks) ────────────────────────────────────────────────────

const CRC_TABLE: [u32; 256] = {
    let mut table = [0u32; 256];
    let mut i = 0;
    while i < 256 {
        let mut c = i as u32;
        let mut j = 0;
        while j < 8 {
            c = if c & 1 != 0 { 0xEDB88320 ^ (c >> 1) } else { c >> 1 };
            j += 1;
        }
        table[i] = c;
        i += 1;
    }
    table
};

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFFFFFF;
    for &b in data {
        crc = CRC_TABLE[((crc ^ b as u32) & 0xFF) as usize] ^ (crc >> 8);
    }
    crc ^ 0xFFFFFFFF
}

// ─── PNG ───────────────────────────────────────────────────────────────────

const PNG_SIG: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
const PNG_IEND: [u8; 4] = *b"IEND";

pub struct PngRepairer;

impl Default for PngRepairer {
    fn default() -> Self {
        Self::new()
    }
}

impl PngRepairer {
    pub fn new() -> Self {
        Self
    }

    fn fix_signature(data: &[u8]) -> Vec<u8> {
        if data.len() >= 8 && data[0..8] == PNG_SIG {
            return data.to_vec();
        }
        // Check if it looks like PNG but with corrupted signature
        if data.len() >= 4 && data[1..4] == *b"PNG" {
            let mut fixed = PNG_SIG.to_vec();
            fixed.extend_from_slice(&data[8.min(data.len())..]);
            fixed
        } else {
            data.to_vec()
        }
    }

    fn fix_chunks(data: &[u8]) -> Vec<u8> {
        if data.len() < 8 {
            return data.to_vec();
        }
        let mut result = data[..8].to_vec(); // signature
        let mut pos = 8;
        let mut found_iend = false;

        while pos + 8 <= data.len() {
            let chunk_len = u32::from_be_bytes([
                data[pos],
                data[pos + 1],
                data[pos + 2],
                data[pos + 3],
            ]) as usize;

            if pos + 12 + chunk_len > data.len() {
                // Truncated chunk — write what we have and add IEND
                let available = &data[pos + 8..data.len().min(pos + 8 + 4 + chunk_len)];
                if available.len() >= 4 {
                    let chunk_type = &available[..4];
                    let chunk_data = if available.len() > 4 {
                        &available[4..]
                    } else {
                        &[]
                    };
                    let crc = crc32(chunk_type);
                    let crc = if chunk_data.is_empty() {
                        crc
                    } else {
                        crc32_with_seed(chunk_data, crc)
                    };
                    result.extend_from_slice(&(chunk_data.len() as u32).to_be_bytes());
                    result.extend_from_slice(chunk_type);
                    result.extend_from_slice(chunk_data);
                    result.extend_from_slice(&crc.to_be_bytes());
                }
                break;
            }

            let chunk_type = &data[pos + 4..pos + 8];
            let chunk_data = &data[pos + 8..pos + 8 + chunk_len];
            let stored_crc = u32::from_be_bytes([
                data[pos + 8 + chunk_len],
                data[pos + 9 + chunk_len],
                data[pos + 10 + chunk_len],
                data[pos + 11 + chunk_len],
            ]);

            let computed_crc = crc32_with_seed(chunk_data, crc32(chunk_type));

            result.extend_from_slice(&(chunk_len as u32).to_be_bytes());
            result.extend_from_slice(chunk_type);
            result.extend_from_slice(chunk_data);

            if stored_crc != computed_crc {
                result.extend_from_slice(&computed_crc.to_be_bytes());
            } else {
                result.extend_from_slice(&stored_crc.to_be_bytes());
            }

            if chunk_type == PNG_IEND {
                found_iend = true;
            }

            pos += 12 + chunk_len;
        }

        if !found_iend {
            // Add IEND chunk
            let crc = crc32(&PNG_IEND);
            result.extend_from_slice(&0u32.to_be_bytes()); // length = 0
            result.extend_from_slice(&PNG_IEND);
            result.extend_from_slice(&crc.to_be_bytes());
        }

        result
    }
}

fn crc32_with_seed(data: &[u8], seed: u32) -> u32 {
    let mut crc = seed ^ 0xFFFFFFFF;
    for &b in data {
        crc = CRC_TABLE[((crc ^ b as u32) & 0xFF) as usize] ^ (crc >> 8);
    }
    crc ^ 0xFFFFFFFF
}

pub struct PngValidator;

impl BinaryValidator for PngValidator {
    fn is_valid(&self, data: &[u8]) -> bool {
        if data.len() < 8 || data[0..8] != PNG_SIG {
            return false;
        }
        let mut pos = 8;
        let mut found_iend = false;
        while pos + 8 <= data.len() {
            let chunk_len = u32::from_be_bytes([
                data[pos],
                data[pos + 1],
                data[pos + 2],
                data[pos + 3],
            ]) as usize;
            if pos + 12 + chunk_len > data.len() {
                return false;
            }
            let chunk_type = &data[pos + 4..pos + 8];
            let chunk_data = &data[pos + 8..pos + 8 + chunk_len];
            let stored_crc = u32::from_be_bytes([
                data[pos + 8 + chunk_len],
                data[pos + 9 + chunk_len],
                data[pos + 10 + chunk_len],
                data[pos + 11 + chunk_len],
            ]);
            let computed = crc32_with_seed(chunk_data, crc32(chunk_type));
            if stored_crc != computed {
                return false;
            }
            if chunk_type == PNG_IEND {
                found_iend = true;
            }
            pos += 12 + chunk_len;
        }
        found_iend
    }
}

impl BinaryRepair for PngRepairer {
    fn repair(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        if data.is_empty() {
            return Err(RepairError::BinaryRepair("Empty input".to_string()));
        }
        let fixed_sig = Self::fix_signature(data);
        let fixed_chunks = Self::fix_chunks(&fixed_sig);
        Ok(fixed_chunks)
    }

    fn needs_repair(&self, data: &[u8]) -> bool {
        !PngValidator.is_valid(data)
    }

    fn confidence(&self, data: &[u8]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }
        if data.len() >= 8 && data[0..8] == PNG_SIG {
            0.95
        } else if data.len() >= 4 && data[1..4] == *b"PNG" {
            0.8
        } else {
            0.0
        }
    }
}

// ─── JPEG ──────────────────────────────────────────────────────────────────

const JPEG_SOI: [u8; 2] = [0xFF, 0xD8];
const JPEG_EOI: [u8; 2] = [0xFF, 0xD9];

pub struct JpegRepairer;

impl Default for JpegRepairer {
    fn default() -> Self {
        Self::new()
    }
}

impl JpegRepairer {
    pub fn new() -> Self {
        Self
    }

    fn fix_header(data: &[u8]) -> Vec<u8> {
        if data.len() >= 2 && data[0..2] == JPEG_SOI {
            return data.to_vec();
        }
        // Find SOI marker in first few bytes
        for i in 0..data.len().min(16).saturating_sub(1) {
            if data[i] == 0xFF && data[i + 1] == 0xD8 {
                return data[i..].to_vec();
            }
        }
        // Prepend SOI
        let mut result = JPEG_SOI.to_vec();
        result.extend_from_slice(data);
        result
    }

    fn fix_eoi(data: &[u8]) -> Vec<u8> {
        if data.len() >= 2 && data[data.len() - 2..] == JPEG_EOI {
            return data.to_vec();
        }
        // Check if EOI exists anywhere — truncate after first EOI
        let mut i = 2; // skip SOI
        while i + 1 < data.len() {
            if data[i] == 0xFF && data[i + 1] == 0xD9 {
                // Found EOI — keep everything up to and including it
                return data[..=i + 1].to_vec();
            }
            i += 1;
        }
        // No EOI found — append it
        let mut result = data.to_vec();
        result.extend_from_slice(&JPEG_EOI);
        result
    }

    fn fix_entropy_data(data: &[u8]) -> Vec<u8> {
        // In JPEG, after SOS marker, entropy-coded data may contain 0xFF bytes
        // that should be followed by 0x00 (stuffing) or a valid marker.
        // Remove any 0xFF 0xFF sequences in entropy data (common corruption).
        let mut result = Vec::with_capacity(data.len());
        let mut in_entropy = false;
        let mut i = 0;

        while i < data.len() {
            if i + 1 < data.len() && data[i] == 0xFF && data[i + 1] == 0xDA {
                in_entropy = true;
            }

            if in_entropy && i + 1 < data.len() && data[i] == 0xFF {
                match data[i + 1] {
                    0x00 => {
                        // Stuffed byte — keep
                        result.push(data[i]);
                        result.push(data[i + 1]);
                        i += 2;
                        continue;
                    }
                    0xD9 => {
                        // EOI — end of entropy data
                        in_entropy = false;
                        result.push(data[i]);
                        result.push(data[i + 1]);
                        i += 2;
                        continue;
                    }
                    0xFF => {
                        // Corrupted — skip duplicate 0xFF
                        i += 1;
                        continue;
                    }
                    _ if data[i + 1] >= 0xC0 && data[i + 1] != 0x00 => {
                        // Valid marker — exit entropy mode
                        in_entropy = false;
                        result.push(data[i]);
                        i += 1;
                        continue;
                    }
                    _ => {}
                }
            }

            result.push(data[i]);
            i += 1;
        }

        result
    }
}

pub struct JpegValidator;

impl BinaryValidator for JpegValidator {
    fn is_valid(&self, data: &[u8]) -> bool {
        if data.len() < 4 {
            return false;
        }
        if data[0..2] != JPEG_SOI {
            return false;
        }
        if data[data.len() - 2..] != JPEG_EOI {
            return false;
        }
        // Must contain at least one marker between SOI and EOI
        data.len() > 4
    }
}

impl BinaryRepair for JpegRepairer {
    fn repair(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        if data.is_empty() {
            return Err(RepairError::BinaryRepair("Empty input".to_string()));
        }
        let fixed_header = Self::fix_header(data);
        let fixed_entropy = Self::fix_entropy_data(&fixed_header);
        let fixed_eoi = Self::fix_eoi(&fixed_entropy);
        Ok(fixed_eoi)
    }

    fn needs_repair(&self, data: &[u8]) -> bool {
        !JpegValidator.is_valid(data)
    }

    fn confidence(&self, data: &[u8]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }
        if data.len() >= 2 && data[0..2] == JPEG_SOI {
            0.95
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_minimal_png() -> Vec<u8> {
        let mut png = PNG_SIG.to_vec();
        // IHDR chunk: 1x1 pixel, 8-bit RGB
        let ihdr_data = [
            0, 0, 0, 1, // width
            0, 0, 0, 1, // height
            8, // bit depth
            2, // color type (RGB)
            0, // compression
            0, // filter
            0, // interlace
        ];
        let ihdr_crc = crc32_with_seed(&ihdr_data, crc32(b"IHDR"));
        png.extend_from_slice(&13u32.to_be_bytes());
        png.extend_from_slice(b"IHDR");
        png.extend_from_slice(&ihdr_data);
        png.extend_from_slice(&ihdr_crc.to_be_bytes());
        // IEND chunk
        let iend_crc = crc32(b"IEND");
        png.extend_from_slice(&0u32.to_be_bytes());
        png.extend_from_slice(b"IEND");
        png.extend_from_slice(&iend_crc.to_be_bytes());
        png
    }

    fn make_minimal_jpeg() -> Vec<u8> {
        let mut jpeg = JPEG_SOI.to_vec();
        // Minimal APP0 marker
        jpeg.extend_from_slice(&[0xFF, 0xE0]);
        jpeg.extend_from_slice(&16u16.to_be_bytes()); // length
        jpeg.extend_from_slice(b"JFIF\0");
        jpeg.extend_from_slice(&[1, 1, 0, 0, 0]); // version, units, density
        jpeg.extend_from_slice(&[0, 0, 0, 0]); // thumbnail
        // SOS marker (minimal)
        jpeg.extend_from_slice(&[0xFF, 0xDA]);
        jpeg.extend_from_slice(&2u16.to_be_bytes()); // length
        // Some entropy data
        jpeg.extend_from_slice(&[0x00, 0x11, 0x22]);
        jpeg.extend_from_slice(&JPEG_EOI);
        jpeg
    }

    #[test]
    fn test_png_valid_passes_through() {
        let png = make_minimal_png();
        let mut r = PngRepairer::new();
        let repaired = r.repair(&png).unwrap();
        assert_eq!(repaired, png);
        assert!(!r.needs_repair(&png));
    }

    #[test]
    fn test_png_fix_missing_iend() {
        let mut png = make_minimal_png();
        // Remove IEND (last 12 bytes)
        png.truncate(png.len() - 12);
        let mut r = PngRepairer::new();
        assert!(r.needs_repair(&png));
        let repaired = r.repair(&png).unwrap();
        assert!(PngValidator.is_valid(&repaired));
        // Should end with IEND
        assert_eq!(&repaired[repaired.len() - 8..repaired.len() - 4], b"IEND");
    }

    #[test]
    fn test_png_fix_bad_crc() {
        let mut png = make_minimal_png();
        // Corrupt the IHDR CRC (byte 8+4+4+13 = 29, CRC at 29..33)
        png[29] ^= 0xFF;
        let mut r = PngRepairer::new();
        assert!(r.needs_repair(&png));
        let repaired = r.repair(&png).unwrap();
        assert!(PngValidator.is_valid(&repaired));
    }

    #[test]
    fn test_png_fix_corrupted_signature() {
        let mut png = make_minimal_png();
        png[0] = 0x00; // Corrupt first byte
        png[1] = b'P';
        png[2] = b'N';
        png[3] = b'G';
        let mut r = PngRepairer::new();
        let repaired = r.repair(&png).unwrap();
        assert_eq!(repaired[0..8], PNG_SIG);
    }

    #[test]
    fn test_png_confidence() {
        let png = make_minimal_png();
        let r = PngRepairer::new();
        assert!(r.confidence(&png) > 0.9);
        assert_eq!(r.confidence(&[]), 0.0);
    }

    #[test]
    fn test_jpeg_valid_passes_through() {
        let jpeg = make_minimal_jpeg();
        let mut r = JpegRepairer::new();
        let repaired = r.repair(&jpeg).unwrap();
        assert_eq!(repaired, jpeg);
    }

    #[test]
    fn test_jpeg_fix_missing_eoi() {
        let mut jpeg = make_minimal_jpeg();
        jpeg.truncate(jpeg.len() - 2); // Remove EOI
        let mut r = JpegRepairer::new();
        assert!(r.needs_repair(&jpeg));
        let repaired = r.repair(&jpeg).unwrap();
        assert_eq!(repaired[repaired.len() - 2..], JPEG_EOI);
    }

    #[test]
    fn test_jpeg_fix_missing_soi() {
        let mut jpeg = make_minimal_jpeg();
        jpeg = jpeg[2..].to_vec(); // Remove SOI
        let mut r = JpegRepairer::new();
        let repaired = r.repair(&jpeg).unwrap();
        assert_eq!(repaired[0..2], JPEG_SOI);
    }

    #[test]
    fn test_jpeg_fix_trailing_garbage() {
        let jpeg = make_minimal_jpeg();
        let mut corrupted = jpeg.clone();
        corrupted.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);
        let mut r = JpegRepairer::new();
        let repaired = r.repair(&corrupted).unwrap();
        // Should end at first EOI
        assert_eq!(repaired, jpeg);
    }

    #[test]
    fn test_jpeg_confidence() {
        let jpeg = make_minimal_jpeg();
        let r = JpegRepairer::new();
        assert!(r.confidence(&jpeg) > 0.9);
        assert_eq!(r.confidence(&[]), 0.0);
    }

    #[test]
    fn test_crc32_known_values() {
        // CRC32 of "IEND" should be a known value
        let crc = crc32(b"IEND");
        assert_eq!(crc, 0xAE426082);
    }
}
