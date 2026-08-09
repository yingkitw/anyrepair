//! Binary format detection using magic bytes.
//!
//! Detects image (PNG, JPEG), audio (WAV, MP3), and video (MP4, MOV) formats
//! from file signatures.

/// Result of binary format detection with confidence.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BinaryDetectionResult {
    pub format: &'static str,
    pub confidence: f64,
}

/// All binary formats supported by anyrepair.
pub const BINARY_FORMATS: &[&str] = &["png", "jpeg", "wav", "mp3", "mp4", "mov"];

/// Detect binary format from magic bytes.
pub fn detect_binary_format(data: &[u8]) -> Option<BinaryDetectionResult> {
    if data.len() < 4 {
        return None;
    }

    // PNG: 89 50 4E 47 0D 0A 1A 0A
    if data.len() >= 8 && data[0..8] == [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
        return Some(BinaryDetectionResult {
            format: "png",
            confidence: 0.99,
        });
    }
    // PNG with corrupted first byte but PNG signature
    if data.len() >= 4 && &data[1..4] == b"PNG" {
        return Some(BinaryDetectionResult {
            format: "png",
            confidence: 0.8,
        });
    }

    // JPEG: FF D8
    if data.len() >= 2 && data[0] == 0xFF && data[1] == 0xD8 {
        return Some(BinaryDetectionResult {
            format: "jpeg",
            confidence: 0.95,
        });
    }

    // WAV: RIFF....WAVE
    if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WAVE" {
        return Some(BinaryDetectionResult {
            format: "wav",
            confidence: 0.95,
        });
    }
    if data.len() >= 4 && &data[0..4] == b"RIFF" {
        return Some(BinaryDetectionResult {
            format: "wav",
            confidence: 0.7,
        });
    }

    // MP3: ID3 or frame sync
    if data.len() >= 3 && &data[0..3] == b"ID3" {
        return Some(BinaryDetectionResult {
            format: "mp3",
            confidence: 0.9,
        });
    }
    if data.len() >= 2 && data[0] == 0xFF && (data[1] & 0xE0) == 0xE0 {
        return Some(BinaryDetectionResult {
            format: "mp3",
            confidence: 0.85,
        });
    }

    // MP4/MOV: ftyp box
    if data.len() >= 12 && &data[4..8] == b"ftyp" {
        let brand = &data[8..12];
        // QuickTime brands
        if brand == b"qt  " || brand == b"qt20" {
            return Some(BinaryDetectionResult {
                format: "mov",
                confidence: 0.95,
            });
        }
        // MP4 brands
        if brand == b"isom" || brand == b"mp41" || brand == b"mp42" || brand == b"avc1" || brand == b"dash" {
            return Some(BinaryDetectionResult {
                format: "mp4",
                confidence: 0.95,
            });
        }
        // Unknown ftyp brand — assume MP4
        return Some(BinaryDetectionResult {
            format: "mp4",
            confidence: 0.75,
        });
    }

    // Check for moov or mdat boxes at offset 4 (no ftyp)
    if data.len() >= 8 {
        let box_type = &data[4..8];
        if box_type == b"moov" || box_type == b"mdat" {
            // Check if it looks more like MOV or MP4 (can't tell without ftyp)
            return Some(BinaryDetectionResult {
                format: "mp4",
                confidence: 0.6,
            });
        }
    }

    None
}

/// Check if a format name is a binary format.
pub fn is_binary_format(format: &str) -> bool {
    BINARY_FORMATS.contains(&format)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_png() {
        let png = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 0];
        let result = detect_binary_format(&png);
        assert_eq!(result.map(|r| r.format), Some("png"));
    }

    #[test]
    fn test_detect_jpeg() {
        let jpeg = [0xFF, 0xD8, 0xFF, 0xE0];
        let result = detect_binary_format(&jpeg);
        assert_eq!(result.map(|r| r.format), Some("jpeg"));
    }

    #[test]
    fn test_detect_wav() {
        let wav = b"RIFF\x00\x00\x00\x00WAVEfmt ";
        let result = detect_binary_format(wav);
        assert_eq!(result.map(|r| r.format), Some("wav"));
    }

    #[test]
    fn test_detect_mp3_id3() {
        let mp3 = b"ID3\x03\x00\x00\x00\x00\x00\x00";
        let result = detect_binary_format(mp3);
        assert_eq!(result.map(|r| r.format), Some("mp3"));
    }

    #[test]
    fn test_detect_mp3_sync() {
        let mp3 = [0xFF, 0xFB, 0x90, 0x64];
        let result = detect_binary_format(&mp3);
        assert_eq!(result.map(|r| r.format), Some("mp3"));
    }

    #[test]
    fn test_detect_mp4() {
        let mut mp4 = vec![0u8; 20];
        mp4[0..4].copy_from_slice(&20u32.to_be_bytes());
        mp4[4..8].copy_from_slice(b"ftyp");
        mp4[8..12].copy_from_slice(b"isom");
        let result = detect_binary_format(&mp4);
        assert_eq!(result.map(|r| r.format), Some("mp4"));
    }

    #[test]
    fn test_detect_mov() {
        let mut mov = vec![0u8; 20];
        mov[0..4].copy_from_slice(&20u32.to_be_bytes());
        mov[4..8].copy_from_slice(b"ftyp");
        mov[8..12].copy_from_slice(b"qt  ");
        let result = detect_binary_format(&mov);
        assert_eq!(result.map(|r| r.format), Some("mov"));
    }

    #[test]
    fn test_detect_unknown() {
        let data = [0x00, 0x01, 0x02, 0x03];
        assert!(detect_binary_format(&data).is_none());
    }

    #[test]
    fn test_detect_empty() {
        assert!(detect_binary_format(&[]).is_none());
    }

    #[test]
    fn test_is_binary_format() {
        assert!(is_binary_format("png"));
        assert!(is_binary_format("mp4"));
        assert!(!is_binary_format("json"));
        assert!(!is_binary_format("yaml"));
    }
}
