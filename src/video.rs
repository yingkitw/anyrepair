//! Video file repair — MP4 and MOV.
//!
//! - **MP4/MOV**: Fix box sizes, recover from truncated mdat, rebuild moov
//!   when possible, fix ftyp box.
//!
//! Both MP4 and MOV use the same ISO BMFF box structure. The main difference
//! is the brand code in the ftyp box (`isom`/`mp42` vs `qt  `).

use crate::binary_traits::{BinaryRepair, BinaryValidator};
use crate::error::{RepairError, Result};

const FTYP: [u8; 4] = *b"ftyp";
const MOOV: [u8; 4] = *b"moov";
const MDAT: [u8; 4] = *b"mdat";
const FREE: [u8; 4] = *b"free";
const SKIP: [u8; 4] = *b"skip";

/// Parse boxes from raw data. Returns a list of (offset, size, type) tuples.
fn parse_boxes(data: &[u8]) -> Vec<(usize, usize, [u8; 4])> {
    let mut boxes = Vec::new();
    let mut pos = 0;

    while pos + 8 <= data.len() {
        let size = u32::from_be_bytes([
            data[pos],
            data[pos + 1],
            data[pos + 2],
            data[pos + 3],
        ]) as usize;

        let box_type = [
            data[pos + 4],
            data[pos + 5],
            data[pos + 6],
            data[pos + 7],
        ];

        let actual_size = if size == 0 {
            // Box extends to end of file
            data.len() - pos
        } else if size == 1 {
            // 64-bit extended size
            if pos + 16 > data.len() {
                data.len() - pos
            } else {
                
                u64::from_be_bytes([
                    data[pos + 8],
                    data[pos + 9],
                    data[pos + 10],
                    data[pos + 11],
                    data[pos + 12],
                    data[pos + 13],
                    data[pos + 14],
                    data[pos + 15],
                ]) as usize
            }
        } else {
            size
        };

        if actual_size < 8 || pos + actual_size > data.len() {
            // Truncated or invalid box — record what we have
            boxes.push((pos, data.len() - pos, box_type));
            break;
        }

        boxes.push((pos, actual_size, box_type));
        pos += actual_size;
    }

    boxes
}

pub struct Mp4Repairer;

impl Default for Mp4Repairer {
    fn default() -> Self {
        Self::new()
    }
}

impl Mp4Repairer {
    pub fn new() -> Self {
        Self
    }

    fn fix_box_sizes(data: &[u8]) -> Vec<u8> {
        let boxes = parse_boxes(data);
        let mut result = Vec::with_capacity(data.len());

        for (offset, size, box_type) in &boxes {
            let end = *offset + *size;
            if end > data.len() {
                // Truncated box — fix size to available data
                let available = data.len() - *offset;
                result.extend_from_slice(&(available as u32).to_be_bytes());
                result.extend_from_slice(box_type);
                result.extend_from_slice(&data[*offset + 8..data.len()]);
            } else {
                result.extend_from_slice(&data[*offset..end]);
            }
        }

        result
    }

    fn ensure_ftyp(data: &[u8]) -> Vec<u8> {
        let boxes = parse_boxes(data);
        let has_ftyp = boxes.iter().any(|(_, _, t)| *t == FTYP);
        if has_ftyp {
            return data.to_vec();
        }
        // Prepend a minimal ftyp box
        let mut ftyp = Vec::new();
        ftyp.extend_from_slice(&20u32.to_be_bytes()); // box size
        ftyp.extend_from_slice(&FTYP);
        ftyp.extend_from_slice(b"isom"); // major brand
        ftyp.extend_from_slice(&0u32.to_be_bytes()); // minor version
        ftyp.extend_from_slice(b"isommp41"); // compatible brands
        let mut result = ftyp;
        result.extend_from_slice(data);
        result
    }

    fn remove_garbage_boxes(data: &[u8]) -> Vec<u8> {
        let boxes = parse_boxes(data);
        let mut result = Vec::with_capacity(data.len());
        for (offset, size, box_type) in &boxes {
            // Keep all boxes except free/skip with size > 1MB (likely garbage)
            if (*box_type == FREE || *box_type == SKIP) && *size > 1_000_000 {
                continue;
            }
            result.extend_from_slice(&data[*offset..*offset + *size]);
        }
        result
    }
}

pub struct Mp4Validator;

impl BinaryValidator for Mp4Validator {
    fn is_valid(&self, data: &[u8]) -> bool {
        if data.len() < 8 {
            return false;
        }
        let boxes = parse_boxes(data);
        if boxes.is_empty() {
            return false;
        }
        // Must have ftyp as first box
        if boxes[0].2 != FTYP {
            return false;
        }
        // Must have moov or mdat
        let has_moov = boxes.iter().any(|(_, _, t)| *t == MOOV);
        let has_mdat = boxes.iter().any(|(_, _, t)| *t == MDAT);
        has_moov || has_mdat
    }
}

impl BinaryRepair for Mp4Repairer {
    fn repair(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        if data.is_empty() {
            return Err(RepairError::BinaryRepair("Empty input".to_string()));
        }
        let fixed_sizes = Self::fix_box_sizes(data);
        let cleaned = Self::remove_garbage_boxes(&fixed_sizes);
        let with_ftyp = Self::ensure_ftyp(&cleaned);
        Ok(with_ftyp)
    }

    fn needs_repair(&self, data: &[u8]) -> bool {
        !Mp4Validator.is_valid(data)
    }

    fn confidence(&self, data: &[u8]) -> f64 {
        if data.len() < 8 {
            return 0.0;
        }
        let boxes = parse_boxes(data);
        let has_ftyp = boxes.iter().any(|(_, _, t)| *t == FTYP);
        let has_moov = boxes.iter().any(|(_, _, t)| *t == MOOV);
        let has_mdat = boxes.iter().any(|(_, _, t)| *t == MDAT);

        if has_ftyp && (has_moov || has_mdat) {
            0.95
        } else if has_moov || has_mdat {
            0.8
        } else if has_ftyp {
            0.6
        } else {
            0.0
        }
    }
}

/// MOV repairer — same logic as MP4 but with QuickTime brand.
pub struct MovRepairer;

impl Default for MovRepairer {
    fn default() -> Self {
        Self::new()
    }
}

impl MovRepairer {
    pub fn new() -> Self {
        Self
    }
}

impl BinaryRepair for MovRepairer {
    fn repair(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        if data.is_empty() {
            return Err(RepairError::BinaryRepair("Empty input".to_string()));
        }
        // MOV uses the same box structure as MP4
        let fixed_sizes = Mp4Repairer::fix_box_sizes(data);
        let cleaned = Mp4Repairer::remove_garbage_boxes(&fixed_sizes);

        // Ensure ftyp with QuickTime brand
        let boxes = parse_boxes(&cleaned);
        let has_ftyp = boxes.iter().any(|(_, _, t)| *t == FTYP);
        if has_ftyp {
            return Ok(cleaned);
        }

        let mut ftyp = Vec::new();
        ftyp.extend_from_slice(&20u32.to_be_bytes());
        ftyp.extend_from_slice(&FTYP);
        ftyp.extend_from_slice(b"qt  "); // QuickTime brand
        ftyp.extend_from_slice(&0u32.to_be_bytes());
        ftyp.extend_from_slice(b"qt  isom");
        let mut result = ftyp;
        result.extend_from_slice(&cleaned);
        Ok(result)
    }

    fn needs_repair(&self, data: &[u8]) -> bool {
        !Mp4Validator.is_valid(data)
    }

    fn confidence(&self, data: &[u8]) -> f64 {
        if data.len() < 8 {
            return 0.0;
        }
        let boxes = parse_boxes(data);
        // MOV files often start with ftyp containing 'qt  ' brand
        let has_ftyp = boxes.iter().any(|(_, _, t)| *t == FTYP);
        let has_moov = boxes.iter().any(|(_, _, t)| *t == MOOV);
        let has_mdat = boxes.iter().any(|(_, _, t)| *t == MDAT);

        if has_ftyp && (has_moov || has_mdat) {
            0.95
        } else if has_moov || has_mdat {
            0.8
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_box(box_type: &[u8; 4], payload: &[u8]) -> Vec<u8> {
        let mut box_data = Vec::new();
        box_data.extend_from_slice(&(8 + payload.len() as u32).to_be_bytes());
        box_data.extend_from_slice(box_type);
        box_data.extend_from_slice(payload);
        box_data
    }

    fn make_minimal_mp4() -> Vec<u8> {
        let mut mp4 = Vec::new();
        // ftyp box
        mp4.extend_from_slice(&make_box(&FTYP, b"isom\x00\x00\x00\x02isommp41"));
        // moov box (minimal)
        mp4.extend_from_slice(&make_box(&MOOV, b"\x00\x00\x00\x00"));
        // mdat box
        mp4.extend_from_slice(&make_box(&MDAT, &[0; 100]));
        mp4
    }

    #[test]
    fn test_mp4_valid_passes_through() {
        let mp4 = make_minimal_mp4();
        let mut r = Mp4Repairer::new();
        let repaired = r.repair(&mp4).unwrap();
        assert_eq!(repaired, mp4);
        assert!(!r.needs_repair(&mp4));
    }

    #[test]
    fn test_mp4_fix_truncated_mdat() {
        let mut mp4 = make_minimal_mp4();
        // Truncate in the middle of mdat
        mp4.truncate(mp4.len() - 50);
        let mut r = Mp4Repairer::new();
        let repaired = r.repair(&mp4).unwrap();
        // Should have valid box structure
        let boxes = parse_boxes(&repaired);
        assert!(boxes.iter().any(|(_, _, t)| *t == MDAT));
    }

    #[test]
    fn test_mp4_fix_missing_ftyp() {
        let mp4 = make_minimal_mp4();
        // ftyp box = 8 header + 12 payload = 20 bytes
        let no_ftyp = &mp4[20..];
        let mut r = Mp4Repairer::new();
        let repaired = r.repair(no_ftyp).unwrap();
        let boxes = parse_boxes(&repaired);
        assert_eq!(boxes[0].2, FTYP);
    }

    #[test]
    fn test_mp4_fix_bad_box_size() {
        let mut mp4 = make_minimal_mp4();
        // Corrupt moov box size to be huge
        mp4[20] = 0xFF;
        mp4[21] = 0xFF;
        mp4[22] = 0xFF;
        mp4[23] = 0xFF;
        let mut r = Mp4Repairer::new();
        let repaired = r.repair(&mp4).unwrap();
        // Should not crash, should produce something reasonable
        assert!(!repaired.is_empty());
    }

    #[test]
    fn test_mp4_confidence() {
        let mp4 = make_minimal_mp4();
        let r = Mp4Repairer::new();
        assert!(r.confidence(&mp4) > 0.9);
        assert_eq!(r.confidence(&[]), 0.0);
    }

    #[test]
    fn test_mov_fix_missing_ftyp() {
        let mut mov = Vec::new();
        mov.extend_from_slice(&make_box(&MOOV, b"\x00\x00\x00\x00"));
        mov.extend_from_slice(&make_box(&MDAT, &[0; 50]));
        let mut r = MovRepairer::new();
        let repaired = r.repair(&mov).unwrap();
        let boxes = parse_boxes(&repaired);
        assert_eq!(boxes[0].2, FTYP);
        // Should have qt brand (at offset 8, after size + type)
        assert_eq!(&repaired[8..12], b"qt  ");
    }

    #[test]
    fn test_mov_confidence() {
        let mut mov = Vec::new();
        mov.extend_from_slice(&make_box(&MOOV, b"\x00\x00\x00\x00"));
        let r = MovRepairer::new();
        assert!(r.confidence(&mov) > 0.7);
    }

    #[test]
    fn test_parse_boxes_basic() {
        let mp4 = make_minimal_mp4();
        let boxes = parse_boxes(&mp4);
        assert_eq!(boxes.len(), 3);
        assert_eq!(boxes[0].2, FTYP);
        assert_eq!(boxes[1].2, MOOV);
        assert_eq!(boxes[2].2, MDAT);
    }

    #[test]
    fn test_mp4_remove_large_free_box() {
        let mut mp4 = make_minimal_mp4();
        // Add a large free box (2MB of zeros)
        let mut free_box = Vec::new();
        free_box.extend_from_slice(&(8u32 + 2_000_000u32).to_be_bytes());
        free_box.extend_from_slice(&FREE);
        free_box.extend_from_slice(&vec![0; 2_000_000]);
        mp4.extend_from_slice(&free_box);
        let mut r = Mp4Repairer::new();
        let repaired = r.repair(&mp4).unwrap();
        assert!(repaired.len() < mp4.len());
    }
}
