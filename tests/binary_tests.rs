//! Integration tests for binary (media) file repair.
//!
//! Tests the full pipeline: detection → repair → validation for all 6 media formats.

use anyrepair::{
    binary_detection::detect_binary_format, create_binary_repairer, create_binary_validator,
    repair_binary, repair_binary_with_format,
};

// ─── Helpers ───────────────────────────────────────────────────────────────

fn make_png() -> Vec<u8> {
    let mut png = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    // IHDR
    let ihdr = [0, 0, 0, 2, 0, 0, 0, 2, 8, 2, 0, 0, 0];
    png.extend_from_slice(&13u32.to_be_bytes());
    png.extend_from_slice(b"IHDR");
    png.extend_from_slice(&ihdr);
    // CRC computed inline
    let crc = png_crc(b"IHDR", &ihdr);
    png.extend_from_slice(&crc.to_be_bytes());
    // IEND
    png.extend_from_slice(&0u32.to_be_bytes());
    png.extend_from_slice(b"IEND");
    let iend_crc = png_crc(b"IEND", &[]);
    png.extend_from_slice(&iend_crc.to_be_bytes());
    png
}

fn png_crc(chunk_type: &[u8], data: &[u8]) -> u32 {
    // CRC32 with chunk type + data
    let mut combined = Vec::new();
    combined.extend_from_slice(chunk_type);
    combined.extend_from_slice(data);
    let mut crc = 0xFFFFFFFFu32;
    for &b in &combined {
        crc = crc32_step(crc, b);
    }
    crc ^ 0xFFFFFFFF
}

fn crc32_step(crc: u32, b: u8) -> u32 {
    const TABLE: [u32; 256] = {
        let mut t = [0u32; 256];
        let mut i = 0;
        while i < 256 {
            let mut c = i as u32;
            let mut j = 0;
            while j < 8 {
                c = if c & 1 != 0 { 0xEDB88320 ^ (c >> 1) } else { c >> 1 };
                j += 1;
            }
            t[i] = c;
            i += 1;
        }
        t
    };
    TABLE[((crc ^ b as u32) & 0xFF) as usize] ^ (crc >> 8)
}

fn make_jpeg() -> Vec<u8> {
    let mut j = vec![0xFF, 0xD8]; // SOI
    j.extend_from_slice(&[0xFF, 0xE0]); // APP0
    j.extend_from_slice(&16u16.to_be_bytes());
    j.extend_from_slice(b"JFIF\0\x01\x01\x00\x00\x00\x00\x00\x00");
    j.extend_from_slice(&[0xFF, 0xDA]); // SOS
    j.extend_from_slice(&2u16.to_be_bytes());
    j.extend_from_slice(&[0x00, 0x11, 0x22]);
    j.extend_from_slice(&[0xFF, 0xD9]); // EOI
    j
}

fn make_wav() -> Vec<u8> {
    let mut w = Vec::new();
    w.extend_from_slice(b"RIFF");
    w.extend_from_slice(&(36 + 100u32).to_le_bytes());
    w.extend_from_slice(b"WAVE");
    w.extend_from_slice(b"fmt ");
    w.extend_from_slice(&16u32.to_le_bytes());
    w.extend_from_slice(&1u16.to_le_bytes()); // PCM
    w.extend_from_slice(&1u16.to_le_bytes()); // mono
    w.extend_from_slice(&8000u32.to_le_bytes()); // sample rate
    w.extend_from_slice(&8000u32.to_le_bytes()); // byte rate
    w.extend_from_slice(&1u16.to_le_bytes()); // block align
    w.extend_from_slice(&8u16.to_le_bytes()); // bits per sample
    w.extend_from_slice(b"data");
    w.extend_from_slice(&100u32.to_le_bytes());
    w.extend_from_slice(&[0x80; 100]);
    w
}

fn make_mp3() -> Vec<u8> {
    let mut m = Vec::new();
    m.extend_from_slice(b"ID3\x03\x00\x00\x00\x00\x00\x00");
    m.extend_from_slice(&[0xFF, 0xFB, 0x90, 0x64]);
    m.extend_from_slice(&[0; 100]);
    m
}

fn make_mp4() -> Vec<u8> {
    let mut m = Vec::new();
    // ftyp: 8 header + 16 payload = 24 bytes
    m.extend_from_slice(&24u32.to_be_bytes());
    m.extend_from_slice(b"ftyp");
    m.extend_from_slice(b"isom");
    m.extend_from_slice(&0u32.to_be_bytes());
    m.extend_from_slice(b"isommp41");
    // moov
    m.extend_from_slice(&12u32.to_be_bytes());
    m.extend_from_slice(b"moov");
    m.extend_from_slice(&[0; 4]);
    // mdat
    m.extend_from_slice(&108u32.to_be_bytes());
    m.extend_from_slice(b"mdat");
    m.extend_from_slice(&[0; 100]);
    m
}

// ─── Detection tests ───────────────────────────────────────────────────────

#[test]
fn test_detect_all_formats() {
    assert_eq!(
        detect_binary_format(&make_png()).map(|r| r.format),
        Some("png")
    );
    assert_eq!(
        detect_binary_format(&make_jpeg()).map(|r| r.format),
        Some("jpeg")
    );
    assert_eq!(
        detect_binary_format(&make_wav()).map(|r| r.format),
        Some("wav")
    );
    assert_eq!(
        detect_binary_format(&make_mp3()).map(|r| r.format),
        Some("mp3")
    );
    assert_eq!(
        detect_binary_format(&make_mp4()).map(|r| r.format),
        Some("mp4")
    );
}

// ─── Repair + validate round-trip ──────────────────────────────────────────

#[test]
fn test_png_repair_valid_output() {
    let png = make_png();
    let mut r = create_binary_repairer("png").unwrap();
    let repaired = r.repair(&png).unwrap();
    let v = create_binary_validator("png").unwrap();
    assert!(v.is_valid(&repaired));
}

#[test]
fn test_jpeg_repair_valid_output() {
    let jpeg = make_jpeg();
    let mut r = create_binary_repairer("jpeg").unwrap();
    let repaired = r.repair(&jpeg).unwrap();
    let v = create_binary_validator("jpeg").unwrap();
    assert!(v.is_valid(&repaired));
}

#[test]
fn test_wav_repair_valid_output() {
    let wav = make_wav();
    let mut r = create_binary_repairer("wav").unwrap();
    let repaired = r.repair(&wav).unwrap();
    let v = create_binary_validator("wav").unwrap();
    assert!(v.is_valid(&repaired));
}

#[test]
fn test_mp3_repair_valid_output() {
    let mp3 = make_mp3();
    let mut r = create_binary_repairer("mp3").unwrap();
    let repaired = r.repair(&mp3).unwrap();
    let v = create_binary_validator("mp3").unwrap();
    assert!(v.is_valid(&repaired));
}

#[test]
fn test_mp4_repair_valid_output() {
    let mp4 = make_mp4();
    let mut r = create_binary_repairer("mp4").unwrap();
    let repaired = r.repair(&mp4).unwrap();
    let v = create_binary_validator("mp4").unwrap();
    assert!(v.is_valid(&repaired));
}

// ─── Auto-detect repair ────────────────────────────────────────────────────

#[test]
fn test_auto_detect_repair_png() {
    let png = make_png();
    let repaired = repair_binary(&png).unwrap();
    assert!(!repaired.is_empty());
    assert_eq!(&repaired[..8], &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
}

#[test]
fn test_auto_detect_repair_jpeg() {
    let jpeg = make_jpeg();
    let repaired = repair_binary(&jpeg).unwrap();
    assert!(!repaired.is_empty());
    assert_eq!(&repaired[..2], &[0xFF, 0xD8]);
}

#[test]
fn test_auto_detect_repair_wav() {
    let wav = make_wav();
    let repaired = repair_binary(&wav).unwrap();
    assert!(&repaired[..4] == b"RIFF");
}

#[test]
fn test_auto_detect_repair_mp3() {
    let mp3 = make_mp3();
    let repaired = repair_binary(&mp3).unwrap();
    assert!(&repaired[..3] == b"ID3");
}

#[test]
fn test_auto_detect_repair_mp4() {
    let mp4 = make_mp4();
    let repaired = repair_binary(&mp4).unwrap();
    assert!(&repaired[4..8] == b"ftyp");
}

// ─── Repair corrupted files ────────────────────────────────────────────────

#[test]
fn test_repair_corrupted_png() {
    let mut png = make_png();
    // Corrupt IEND (remove last 12 bytes)
    png.truncate(png.len() - 12);
    let repaired = repair_binary_with_format(&png, "png").unwrap();
    let v = create_binary_validator("png").unwrap();
    assert!(v.is_valid(&repaired));
}

#[test]
fn test_repair_corrupted_jpeg_missing_eoi() {
    let mut jpeg = make_jpeg();
    jpeg.truncate(jpeg.len() - 2);
    let repaired = repair_binary_with_format(&jpeg, "jpeg").unwrap();
    assert_eq!(&repaired[repaired.len() - 2..], &[0xFF, 0xD9]);
}

#[test]
fn test_repair_corrupted_wav_bad_size() {
    let mut wav = make_wav();
    wav[4] = 0;
    wav[5] = 0;
    let repaired = repair_binary_with_format(&wav, "wav").unwrap();
    let v = create_binary_validator("wav").unwrap();
    assert!(v.is_valid(&repaired));
}

#[test]
fn test_repair_truncated_mp4() {
    let mut mp4 = make_mp4();
    mp4.truncate(mp4.len() - 50);
    let repaired = repair_binary_with_format(&mp4, "mp4").unwrap();
    assert!(!repaired.is_empty());
}

// ─── Idempotency ───────────────────────────────────────────────────────────

#[test]
fn test_idempotent_png() {
    let png = make_png();
    let r1 = repair_binary_with_format(&png, "png").unwrap();
    let r2 = repair_binary_with_format(&r1, "png").unwrap();
    assert_eq!(r1, r2);
}

#[test]
fn test_idempotent_jpeg() {
    let jpeg = make_jpeg();
    let r1 = repair_binary_with_format(&jpeg, "jpeg").unwrap();
    let r2 = repair_binary_with_format(&r1, "jpeg").unwrap();
    assert_eq!(r1, r2);
}

#[test]
fn test_idempotent_wav() {
    let wav = make_wav();
    let r1 = repair_binary_with_format(&wav, "wav").unwrap();
    let r2 = repair_binary_with_format(&r1, "wav").unwrap();
    assert_eq!(r1, r2);
}

// ─── Edge cases ────────────────────────────────────────────────────────────

#[test]
fn test_repair_binary_unknown_format() {
    let data = vec![0x00, 0x01, 0x02, 0x03];
    // Auto-detect returns original data unchanged
    let result = repair_binary(&data).unwrap();
    assert_eq!(result, data);
}

#[test]
fn test_repair_binary_empty_input() {
    let result = repair_binary_with_format(&[], "png");
    assert!(result.is_err());
}

#[test]
fn test_create_binary_repairer_unknown() {
    assert!(create_binary_repairer("xyz").is_err());
}

#[test]
fn test_format_alias_jpg() {
    let jpeg = make_jpeg();
    let r = create_binary_repairer("jpg");
    assert!(r.is_ok());
    let mut r = r.unwrap();
    assert!(r.repair(&jpeg).is_ok());
}

// ─── Never panics ──────────────────────────────────────────────────────────

#[test]
fn test_never_panics_all_formats() {
    let inputs = vec![
        ("png", make_png()),
        ("jpeg", make_jpeg()),
        ("wav", make_wav()),
        ("mp3", make_mp3()),
        ("mp4", make_mp4()),
    ];

    for (fmt, data) in &inputs {
        let _ = repair_binary_with_format(data, fmt);
    }

    // Also with corrupted versions
    for (_, mut data) in inputs {
        if data.len() > 10 {
            data.truncate(data.len() / 2);
        }
        let _ = repair_binary(&data);
    }
}
