//! Audio file repair — WAV and MP3.
//!
//! - **WAV**: Fix RIFF header, chunk sizes, truncated data chunks.
//! - **MP3**: Fix ID3v2 header size, frame synchronization, remove invalid frames.

use crate::binary_traits::{BinaryRepair, BinaryValidator};
use crate::error::{RepairError, Result};

// ─── WAV ───────────────────────────────────────────────────────────────────

const WAV_RIFF: [u8; 4] = *b"RIFF";
const WAV_WAVE: [u8; 4] = *b"WAVE";
const WAV_FMT: [u8; 4] = *b"fmt ";

pub struct WavRepairer;

impl Default for WavRepairer {
    fn default() -> Self {
        Self::new()
    }
}

impl WavRepairer {
    pub fn new() -> Self {
        Self
    }

    fn fix_header(data: &[u8]) -> Vec<u8> {
        if data.len() < 12 {
            return data.to_vec();
        }
        let mut result = data.to_vec();
        // Fix RIFF tag
        if result[0..4] != WAV_RIFF {
            // Check if it's just case issue
            if result[0..4].eq_ignore_ascii_case(&WAV_RIFF) {
                result[0..4].copy_from_slice(&WAV_RIFF);
            } else if result.len() >= 8 && result[8..12] == WAV_WAVE {
                // Has WAVE but wrong RIFF tag
                result[0..4].copy_from_slice(&WAV_RIFF);
            }
        }
        // Fix WAVE tag
        if result.len() >= 12 && result[8..12] != WAV_WAVE
            && result[8..12].eq_ignore_ascii_case(&WAV_WAVE) {
                result[8..12].copy_from_slice(&WAV_WAVE);
            }
        // Fix file size in RIFF header (offset 4, should be file_size - 8)
        if result.len() >= 8 {
            let size = (result.len() - 8) as u32;
            result[4..8].copy_from_slice(&size.to_le_bytes());
        }
        result
    }

    fn fix_chunks(data: &[u8]) -> Vec<u8> {
        if data.len() < 12 {
            return data.to_vec();
        }
        let mut result = data[..12].to_vec(); // RIFF header + WAVE
        let mut pos = 12;
        let mut has_fmt = false;

        while pos + 8 <= data.len() {
            let chunk_id = &data[pos..pos + 4];
            let chunk_size = u32::from_le_bytes([
                data[pos + 4],
                data[pos + 5],
                data[pos + 6],
                data[pos + 7],
            ]) as usize;

            // Check if chunk size is valid
            let actual_data_end = pos + 8 + chunk_size;
            let available_end = if actual_data_end > data.len() {
                data.len()
            } else {
                actual_data_end
            };
            let actual_size = available_end - (pos + 8);

            if chunk_id == WAV_FMT {
                has_fmt = true;
            }

            result.extend_from_slice(chunk_id);
            result.extend_from_slice(&(actual_size as u32).to_le_bytes());
            if pos + 8 < data.len() {
                result.extend_from_slice(&data[pos + 8..available_end]);
            }

            // Pad to even boundary
            if actual_size % 2 != 0 && actual_data_end <= data.len() {
                result.push(0);
            }

            pos = actual_data_end;
            if pos % 2 != 0 {
                pos += 1; // Skip padding byte
            }
        }

        // If missing fmt chunk, add a minimal one
        if !has_fmt && result.len() > 12 {
            let fmt_data = [
                1, 0, // PCM format
                1, 0, // Mono
                0x40, 0x1F, 0, 0, // 8000 Hz sample rate
                0x80, 0x3E, 0, 0, // byte rate
                1, 0, // block align
                8, 0, // bits per sample
            ];
            let mut new_result = result[..12].to_vec();
            new_result.extend_from_slice(&WAV_FMT);
            new_result.extend_from_slice(&16u32.to_le_bytes());
            new_result.extend_from_slice(&fmt_data);
            new_result.extend_from_slice(&result[12..]);
            result = new_result;
        }

        // Fix RIFF size
        let size = (result.len() - 8) as u32;
        result[4..8].copy_from_slice(&size.to_le_bytes());

        result
    }
}

pub struct WavValidator;

impl BinaryValidator for WavValidator {
    fn is_valid(&self, data: &[u8]) -> bool {
        if data.len() < 12 {
            return false;
        }
        if data[0..4] != WAV_RIFF {
            return false;
        }
        if data[8..12] != WAV_WAVE {
            return false;
        }
        // Check that RIFF size matches
        let riff_size = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;
        riff_size == data.len() - 8
    }
}

impl BinaryRepair for WavRepairer {
    fn repair(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        if data.is_empty() {
            return Err(RepairError::BinaryRepair("Empty input".to_string()));
        }
        let fixed_header = Self::fix_header(data);
        let fixed_chunks = Self::fix_chunks(&fixed_header);
        Ok(fixed_chunks)
    }

    fn needs_repair(&self, data: &[u8]) -> bool {
        !WavValidator.is_valid(data)
    }

    fn confidence(&self, data: &[u8]) -> f64 {
        if data.len() >= 12 && data[0..4] == WAV_RIFF && data[8..12] == WAV_WAVE {
            0.95
        } else if data.len() >= 4 && data[0..4] == WAV_RIFF {
            0.8
        } else {
            0.0
        }
    }
}

// ─── MP3 ───────────────────────────────────────────────────────────────────

const ID3V2_SIG: [u8; 3] = *b"ID3";

pub struct Mp3Repairer;

impl Default for Mp3Repairer {
    fn default() -> Self {
        Self::new()
    }
}

impl Mp3Repairer {
    pub fn new() -> Self {
        Self
    }

    fn fix_id3_header(data: &[u8]) -> Vec<u8> {
        if data.len() < 10 {
            return data.to_vec();
        }
        if data[0..3] != ID3V2_SIG {
            return data.to_vec();
        }
        let mut result = data.to_vec();
        // ID3v2 size is a synchsafe integer (4 bytes, each 7-bit) at offset 6-9
        // Recalculate if the size looks wrong
        let declared_size = ((data[6] as u32 & 0x7F) << 21)
            | ((data[7] as u32 & 0x7F) << 14)
            | ((data[8] as u32 & 0x7F) << 7)
            | (data[9] as u32 & 0x7F);

        let header_end = 10 + declared_size as usize;
        if header_end > data.len() {
            // Truncated ID3 — fix the size to match available data
            let actual_size = data.len() - 10;
            let synchsafe = (
                ((actual_size >> 21) & 0x7F) as u8,
                ((actual_size >> 14) & 0x7F) as u8,
                ((actual_size >> 7) & 0x7F) as u8,
                (actual_size & 0x7F) as u8,
            );
            result[6] = synchsafe.0;
            result[7] = synchsafe.1;
            result[8] = synchsafe.2;
            result[9] = synchsafe.3;
        }
        result
    }

    fn fix_frame_sync(data: &[u8]) -> Vec<u8> {
        let mut result = Vec::with_capacity(data.len());
        let mut i = 0;

        // Skip ID3v2 header if present
        if data.len() >= 10 && data[0..3] == ID3V2_SIG {
            let id3_size = ((data[6] as u32 & 0x7F) << 21)
                | ((data[7] as u32 & 0x7F) << 14)
                | ((data[8] as u32 & 0x7F) << 7)
                | (data[9] as u32 & 0x7F);
            let header_end = (10 + id3_size as usize).min(data.len());
            result.extend_from_slice(&data[..header_end]);
            i = header_end;
        }

        // Process frame data
        while i < data.len() {
            if i + 1 < data.len() && data[i] == 0xFF && (data[i + 1] & 0xE0) == 0xE0 {
                // Valid frame sync — copy until next sync or end
                let frame_start = i;
                i += 2;
                // Skip past frame header (4 bytes total for MPEG)
                while i < data.len() {
                    if i + 1 < data.len() && data[i] == 0xFF && (data[i + 1] & 0xE0) == 0xE0 {
                        break;
                    }
                    i += 1;
                }
                result.extend_from_slice(&data[frame_start..i]);
            } else {
                // Not a valid sync — skip byte
                i += 1;
            }
        }

        // If we stripped everything, keep original
        if result.is_empty() || (result.len() < 10 && !data.is_empty()) {
            return data.to_vec();
        }

        result
    }

    fn fix_trailing_garbage(data: &[u8]) -> Vec<u8> {
        // Remove trailing ID3v1 tag if it's incomplete (less than 128 bytes from end)
        if data.len() >= 128 && &data[data.len() - 128..data.len() - 125] == b"TAG" {
            return data.to_vec(); // Valid ID3v1 tag
        }
        // Remove trailing null bytes
        let mut end = data.len();
        while end > 0 && data[end - 1] == 0 {
            end -= 1;
        }
        if end < data.len() {
            data[..end].to_vec()
        } else {
            data.to_vec()
        }
    }
}

pub struct Mp3Validator;

impl BinaryValidator for Mp3Validator {
    fn is_valid(&self, data: &[u8]) -> bool {
        if data.len() < 4 {
            return false;
        }
        // Has ID3v2 header or at least one valid frame sync
        if data[0..3] == ID3V2_SIG {
            return true;
        }
        // Check for frame sync
        data[0] == 0xFF && (data[1] & 0xE0) == 0xE0
    }
}

impl BinaryRepair for Mp3Repairer {
    fn repair(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        if data.is_empty() {
            return Err(RepairError::BinaryRepair("Empty input".to_string()));
        }
        let fixed_id3 = Self::fix_id3_header(data);
        let fixed_frames = Self::fix_frame_sync(&fixed_id3);
        let fixed_trailing = Self::fix_trailing_garbage(&fixed_frames);
        Ok(fixed_trailing)
    }

    fn needs_repair(&self, data: &[u8]) -> bool {
        !Mp3Validator.is_valid(data)
    }

    fn confidence(&self, data: &[u8]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }
        if data.len() >= 3 && data[0..3] == ID3V2_SIG {
            0.9
        } else if data.len() >= 2 && data[0] == 0xFF && (data[1] & 0xE0) == 0xE0 {
            0.85
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_minimal_wav() -> Vec<u8> {
        let sample_rate = 8000u32;
        let bits_per_sample = 8u16;
        let channels = 1u16;
        let byte_rate = sample_rate * channels as u32 * (bits_per_sample as u32 / 8);
        let block_align = channels * (bits_per_sample / 8);
        let audio_data = [0x80; 100]; // 100 bytes of silence (unsigned 8-bit)

        let mut wav = Vec::new();
        // RIFF header
        wav.extend_from_slice(&WAV_RIFF);
        wav.extend_from_slice(&(36 + audio_data.len() as u32).to_le_bytes()); // file size - 8
        wav.extend_from_slice(&WAV_WAVE);
        // fmt chunk
        wav.extend_from_slice(&WAV_FMT);
        wav.extend_from_slice(&16u32.to_le_bytes()); // chunk size
        wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
        wav.extend_from_slice(&channels.to_le_bytes());
        wav.extend_from_slice(&sample_rate.to_le_bytes());
        wav.extend_from_slice(&byte_rate.to_le_bytes());
        wav.extend_from_slice(&block_align.to_le_bytes());
        wav.extend_from_slice(&bits_per_sample.to_le_bytes());
        // data chunk
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&(audio_data.len() as u32).to_le_bytes());
        wav.extend_from_slice(&audio_data);
        wav
    }

    fn make_minimal_mp3() -> Vec<u8> {
        let mut mp3 = Vec::new();
        // ID3v2 header
        mp3.extend_from_slice(&ID3V2_SIG);
        mp3.extend_from_slice(&[3, 0]); // version 3.0, flags = 0
        mp3.extend_from_slice(&[0, 0, 0, 0]); // size = 0 (no extended header)
        // Frame sync (MPEG1 Layer3, 128kbps, 44100Hz)
        mp3.extend_from_slice(&[0xFF, 0xFB, 0x90, 0x64]);
        mp3.extend_from_slice(&[0; 100]); // frame data
        mp3
    }

    #[test]
    fn test_wav_valid_passes_through() {
        let wav = make_minimal_wav();
        let mut r = WavRepairer::new();
        let repaired = r.repair(&wav).unwrap();
        assert_eq!(repaired, wav);
        assert!(!r.needs_repair(&wav));
    }

    #[test]
    fn test_wav_fix_riff_size() {
        let mut wav = make_minimal_wav();
        wav[4] = 0; // Corrupt RIFF size
        wav[5] = 0;
        let mut r = WavRepairer::new();
        assert!(r.needs_repair(&wav));
        let repaired = r.repair(&wav).unwrap();
        assert!(WavValidator.is_valid(&repaired));
    }

    #[test]
    fn test_wav_fix_truncated_data() {
        let mut wav = make_minimal_wav();
        // Truncate in the middle of data chunk
        wav.truncate(wav.len() - 20);
        let mut r = WavRepairer::new();
        let repaired = r.repair(&wav).unwrap();
        assert!(WavValidator.is_valid(&repaired));
    }

    #[test]
    fn test_wav_fix_wrong_riff_tag() {
        let mut wav = make_minimal_wav();
        wav[0..4].copy_from_slice(b"RIFX"); // Wrong tag
        let mut r = WavRepairer::new();
        let repaired = r.repair(&wav).unwrap();
        assert_eq!(&repaired[0..4], &WAV_RIFF);
    }

    #[test]
    fn test_wav_confidence() {
        let wav = make_minimal_wav();
        let r = WavRepairer::new();
        assert!(r.confidence(&wav) > 0.9);
        assert_eq!(r.confidence(&[]), 0.0);
    }

    #[test]
    fn test_mp3_valid_passes_through() {
        let mp3 = make_minimal_mp3();
        let mut r = Mp3Repairer::new();
        let repaired = r.repair(&mp3).unwrap();
        // ID3 header should be preserved
        assert_eq!(&repaired[0..3], &ID3V2_SIG);
    }

    #[test]
    fn test_mp3_fix_truncated_id3() {
        let mut mp3 = make_minimal_mp3();
        // Set large ID3 size then truncate
        mp3[6] = 0x01; // Claims 128 bytes of ID3 data
        mp3[7] = 0;
        mp3[8] = 0;
        mp3[9] = 0;
        // But we only have ~110 bytes total
        let mut r = Mp3Repairer::new();
        let repaired = r.repair(&mp3).unwrap();
        // Should not crash and should produce something
        assert!(!repaired.is_empty());
    }

    #[test]
    fn test_mp3_fix_no_id3_with_sync() {
        let mut mp3 = vec![0xFF, 0xFB, 0x90, 0x64]; // Frame sync only
        mp3.extend_from_slice(&[0; 50]);
        let mut r = Mp3Repairer::new();
        let repaired = r.repair(&mp3).unwrap();
        assert!(!repaired.is_empty());
        assert_eq!(repaired[0], 0xFF);
    }

    #[test]
    fn test_mp3_confidence() {
        let mp3 = make_minimal_mp3();
        let r = Mp3Repairer::new();
        assert!(r.confidence(&mp3) > 0.8);
        assert_eq!(r.confidence(&[]), 0.0);
    }

    #[test]
    fn test_mp3_strip_garbage_before_sync() {
        let mut mp3 = vec![0x00, 0x00, 0xDE, 0xAD]; // Garbage
        mp3.extend_from_slice(&[0xFF, 0xFB, 0x90, 0x64]); // Frame sync
        mp3.extend_from_slice(&[0; 50]);
        let mut r = Mp3Repairer::new();
        let repaired = r.repair(&mp3).unwrap();
        // Should start with frame sync (garbage stripped)
        assert_eq!(repaired[0], 0xFF);
    }
}
