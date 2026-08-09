//! Comprehensive binary repair tests — corruption patterns, round-trip,
//! edge cases, and format-specific damage scenarios.
//!
//! These tests complement `tests/binary_tests.rs` with deeper coverage
//! of each format's repair strategies.

use anyrepair::{
    binary_detection::detect_binary_format, create_binary_repairer, create_binary_validator,
    repair_binary, repair_binary_with_format,
};

// ─── Test helpers ──────────────────────────────────────────────────────────

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

fn make_png() -> Vec<u8> {
    let mut png = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    // IHDR: 4x4 RGB
    let ihdr = [0, 0, 0, 4, 0, 0, 0, 4, 8, 2, 0, 0, 0];
    png.extend_from_slice(&13u32.to_be_bytes());
    png.extend_from_slice(b"IHDR");
    png.extend_from_slice(&ihdr);
    let crc = crc32(&{
        let mut c = b"IHDR".to_vec();
        c.extend_from_slice(&ihdr);
        c
    });
    png.extend_from_slice(&crc.to_be_bytes());
    // IDAT (dummy 10-byte data)
    let idat_data = [0x78, 0x9C, 0x62, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x00];
    png.extend_from_slice(&(idat_data.len() as u32).to_be_bytes());
    png.extend_from_slice(b"IDAT");
    png.extend_from_slice(&idat_data);
    let idat_crc = crc32(&{
        let mut c = b"IDAT".to_vec();
        c.extend_from_slice(&idat_data);
        c
    });
    png.extend_from_slice(&idat_crc.to_be_bytes());
    // IEND
    png.extend_from_slice(&0u32.to_be_bytes());
    png.extend_from_slice(b"IEND");
    let iend_crc = crc32(b"IEND");
    png.extend_from_slice(&iend_crc.to_be_bytes());
    png
}

fn make_jpeg() -> Vec<u8> {
    let mut j = vec![0xFF, 0xD8]; // SOI
    j.extend_from_slice(&[0xFF, 0xE0]); // APP0
    j.extend_from_slice(&16u16.to_be_bytes());
    j.extend_from_slice(b"JFIF\0\x01\x01\x00\x00\x00\x00\x00\x00");
    // DQT marker (minimal)
    j.extend_from_slice(&[0xFF, 0xDB]);
    j.extend_from_slice(&67u16.to_be_bytes());
    j.extend_from_slice(&[0; 65]);
    // SOF0 marker
    j.extend_from_slice(&[0xFF, 0xC0]);
    j.extend_from_slice(&17u16.to_be_bytes());
    j.extend_from_slice(&[8, 0, 4, 0, 4, 1, 1, 0x11, 0x00]); // 8-bit, 4x4, 1 component
    // DHT marker (minimal)
    j.extend_from_slice(&[0xFF, 0xC4]);
    j.extend_from_slice(&19u16.to_be_bytes());
    j.extend_from_slice(&[0; 17]);
    // SOS marker
    j.extend_from_slice(&[0xFF, 0xDA]);
    j.extend_from_slice(&8u16.to_be_bytes());
    j.extend_from_slice(&[1, 1, 0, 0, 63, 0]);
    // Entropy data
    j.extend_from_slice(&[0x00, 0x11, 0x22, 0x33, 0x44]);
    j.extend_from_slice(&[0xFF, 0xD9]); // EOI
    j
}

fn make_wav() -> Vec<u8> {
    let mut w = Vec::new();
    w.extend_from_slice(b"RIFF");
    w.extend_from_slice(&(36 + 200u32).to_le_bytes());
    w.extend_from_slice(b"WAVE");
    w.extend_from_slice(b"fmt ");
    w.extend_from_slice(&16u32.to_le_bytes());
    w.extend_from_slice(&1u16.to_le_bytes()); // PCM
    w.extend_from_slice(&2u16.to_le_bytes()); // stereo
    w.extend_from_slice(&44100u32.to_le_bytes());
    w.extend_from_slice(&176400u32.to_le_bytes()); // byte rate
    w.extend_from_slice(&4u16.to_le_bytes()); // block align
    w.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
    w.extend_from_slice(b"data");
    w.extend_from_slice(&200u32.to_le_bytes());
    w.extend_from_slice(&[0xAB; 200]);
    w
}

fn make_mp3() -> Vec<u8> {
    let mut m = Vec::new();
    m.extend_from_slice(b"ID3\x03\x00\x00\x00\x00\x00\x00");
    // Multiple frames
    for _ in 0..3 {
        m.extend_from_slice(&[0xFF, 0xFB, 0x90, 0x64]);
        m.extend_from_slice(&[0; 100]);
    }
    m
}

fn make_mp4() -> Vec<u8> {
    let mut m = Vec::new();
    // ftyp: 8 + 16 = 24
    m.extend_from_slice(&24u32.to_be_bytes());
    m.extend_from_slice(b"ftyp");
    m.extend_from_slice(b"isom");
    m.extend_from_slice(&0u32.to_be_bytes());
    m.extend_from_slice(b"isommp41");
    // moov (with minimal mvhd)
    let moov_payload = [0; 20];
    m.extend_from_slice(&(8 + moov_payload.len() as u32).to_be_bytes());
    m.extend_from_slice(b"moov");
    m.extend_from_slice(&moov_payload);
    // mdat
    let mdat_data = [0xAA; 150];
    m.extend_from_slice(&(8 + mdat_data.len() as u32).to_be_bytes());
    m.extend_from_slice(b"mdat");
    m.extend_from_slice(&mdat_data);
    m
}

// ─── PNG corruption tests ──────────────────────────────────────────────────

#[test]
fn png_corrupt_ihdr_crc() {
    let mut png = make_png();
    // IHDR CRC starts at: 8 (sig) + 4 (len) + 4 (type) + 13 (data) = 29
    png[29] ^= 0xFF;
    let mut r = create_binary_repairer("png").unwrap();
    assert!(r.needs_repair(&png));
    let repaired = r.repair(&png).unwrap();
    let v = create_binary_validator("png").unwrap();
    assert!(v.is_valid(&repaired));
}

#[test]
fn png_corrupt_idat_crc() {
    let mut png = make_png();
    // IDAT chunk starts at: 8 + 25 (IHDR) = 33
    // IDAT CRC at: 33 + 4 + 4 + 10 = 51
    png[51] ^= 0xFF;
    let repaired = repair_binary_with_format(&png, "png").unwrap();
    let v = create_binary_validator("png").unwrap();
    assert!(v.is_valid(&repaired));
}

#[test]
fn png_corrupt_signature_first_byte() {
    let mut png = make_png();
    png[0] = 0x00;
    let repaired = repair_binary_with_format(&png, "png").unwrap();
    assert_eq!(&repaired[..8], &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
}

#[test]
fn png_missing_iend_and_trailing_garbage() {
    let mut png = make_png();
    // Remove IEND and add garbage
    png.truncate(png.len() - 12);
    png.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);
    let repaired = repair_binary_with_format(&png, "png").unwrap();
    let v = create_binary_validator("png").unwrap();
    assert!(v.is_valid(&repaired));
}

#[test]
fn png_truncated_in_idat() {
    let mut png = make_png();
    // Truncate in the middle of IDAT data
    png.truncate(40);
    let repaired = repair_binary_with_format(&png, "png").unwrap();
    // Should have IEND appended
    assert!(repaired.windows(4).any(|w| w == b"IEND"));
}

#[test]
fn png_valid_idempotent() {
    let png = make_png();
    let r1 = repair_binary_with_format(&png, "png").unwrap();
    let r2 = repair_binary_with_format(&r1, "png").unwrap();
    assert_eq!(r1, r2);
    assert_eq!(r1, png);
}

#[test]
fn png_empty_input_errors() {
    let result = repair_binary_with_format(&[], "png");
    assert!(result.is_err());
}

#[test]
fn png_confidence_high_for_valid() {
    let png = make_png();
    let r = create_binary_repairer("png").unwrap();
    assert!(r.confidence(&png) > 0.9);
}

#[test]
fn png_confidence_for_corrupted_sig() {
    let mut png = make_png();
    png[0] = 0x00;
    png[1] = b'P';
    png[2] = b'N';
    png[3] = b'G';
    let r = create_binary_repairer("png").unwrap();
    assert!(r.confidence(&png) > 0.5);
}

// ─── JPEG corruption tests ─────────────────────────────────────────────────

#[test]
fn jpeg_missing_soi() {
    let jpeg = make_jpeg();
    let no_soi = &jpeg[2..];
    let repaired = repair_binary_with_format(no_soi, "jpeg").unwrap();
    assert_eq!(&repaired[..2], &[0xFF, 0xD8]);
}

#[test]
fn jpeg_missing_eoi() {
    let mut jpeg = make_jpeg();
    jpeg.truncate(jpeg.len() - 2);
    let repaired = repair_binary_with_format(&jpeg, "jpeg").unwrap();
    assert!(repaired.ends_with(&[0xFF, 0xD9]));
}

#[test]
fn jpeg_trailing_garbage_after_eoi() {
    let mut jpeg = make_jpeg();
    jpeg.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x01]);
    let repaired = repair_binary_with_format(&jpeg, "jpeg").unwrap();
    // Should end at first EOI, no trailing garbage
    assert!(repaired.ends_with(&[0xFF, 0xD9]));
    assert!(!repaired.ends_with(&[0x00, 0x01]));
}

#[test]
fn jpeg_double_soi() {
    let jpeg = make_jpeg();
    let mut double = vec![0xFF, 0xD8, 0xFF, 0xD8];
    double.extend_from_slice(&jpeg[2..]);
    let repaired = repair_binary_with_format(&double, "jpeg").unwrap();
    // Should start with SOI
    assert_eq!(&repaired[..2], &[0xFF, 0xD8]);
}

#[test]
fn jpeg_valid_idempotent() {
    let jpeg = make_jpeg();
    let r1 = repair_binary_with_format(&jpeg, "jpeg").unwrap();
    let r2 = repair_binary_with_format(&r1, "jpeg").unwrap();
    assert_eq!(r1, r2);
}

#[test]
fn jpeg_confidence_zero_for_non_jpeg() {
    let r = create_binary_repairer("jpeg").unwrap();
    assert_eq!(r.confidence(b"not a jpeg"), 0.0);
}

#[test]
fn jpeg_needs_repair_for_missing_eoi() {
    let mut jpeg = make_jpeg();
    jpeg.truncate(jpeg.len() - 2);
    let r = create_binary_repairer("jpeg").unwrap();
    assert!(r.needs_repair(&jpeg));
}

// ─── WAV corruption tests ──────────────────────────────────────────────────

#[test]
fn wav_wrong_riff_tag_case() {
    let mut wav = make_wav();
    wav[0..4].copy_from_slice(b"riff");
    let repaired = repair_binary_with_format(&wav, "wav").unwrap();
    assert_eq!(&repaired[..4], b"RIFF");
}

#[test]
fn wav_wrong_wave_tag() {
    let mut wav = make_wav();
    wav[8..12].copy_from_slice(b"wave");
    let repaired = repair_binary_with_format(&wav, "wav").unwrap();
    assert_eq!(&repaired[8..12], b"WAVE");
}

#[test]
fn wav_truncated_data_chunk() {
    let mut wav = make_wav();
    wav.truncate(wav.len() - 50);
    let repaired = repair_binary_with_format(&wav, "wav").unwrap();
    let v = create_binary_validator("wav").unwrap();
    assert!(v.is_valid(&repaired));
}

#[test]
fn wav_oversized_data_chunk() {
    let mut wav = make_wav();
    // Claim data chunk is 1000 bytes but only provide 200
    wav[44..48].copy_from_slice(&1000u32.to_le_bytes());
    let repaired = repair_binary_with_format(&wav, "wav").unwrap();
    let v = create_binary_validator("wav").unwrap();
    assert!(v.is_valid(&repaired));
    // The repaired file should be smaller than what the oversized chunk claimed
    assert!(repaired.len() < wav.len() + 800);
}

#[test]
fn wav_valid_idempotent() {
    let wav = make_wav();
    let r1 = repair_binary_with_format(&wav, "wav").unwrap();
    let r2 = repair_binary_with_format(&r1, "wav").unwrap();
    assert_eq!(r1, r2);
}

#[test]
fn wav_confidence_for_just_riff() {
    let data = b"RIFF\x00\x00\x00\x00XXXX";
    let r = create_binary_repairer("wav").unwrap();
    assert!(r.confidence(data) > 0.5);
}

#[test]
fn wav_validator_rejects_non_wav() {
    let v = create_binary_validator("wav").unwrap();
    assert!(!v.is_valid(b"NOTRIFFXXXX"));
}

// ─── MP3 corruption tests ──────────────────────────────────────────────────

#[test]
fn mp3_strip_garbage_before_frames() {
    let mp3 = make_mp3();
    let mut corrupted = vec![0x00, 0x01, 0x02, 0x03, 0xDE, 0xAD];
    corrupted.extend_from_slice(&mp3[10..]); // skip ID3, add garbage before frames
    let mut with_id3 = mp3[..10].to_vec();
    with_id3.extend_from_slice(&corrupted);
    let repaired = repair_binary_with_format(&with_id3, "mp3").unwrap();
    // Should contain frame sync bytes
    assert!(repaired.windows(2).any(|w| w[0] == 0xFF && (w[1] & 0xE0) == 0xE0));
}

#[test]
fn mp3_truncated_id3_header() {
    let mut mp3 = make_mp3();
    // Set large ID3 size (claims 500 bytes of ID3 data)
    mp3[6] = 0x00;
    mp3[7] = 0x00;
    mp3[8] = 0x03;
    mp3[9] = 0xE8; // 500 in synchsafe
    // But file is only ~310 bytes
    let repaired = repair_binary_with_format(&mp3, "mp3").unwrap();
    assert!(!repaired.is_empty());
}

#[test]
fn mp3_no_id3_just_frames() {
    let mut mp3 = vec![0xFF, 0xFB, 0x90, 0x64];
    mp3.extend_from_slice(&[0; 100]);
    let repaired = repair_binary_with_format(&mp3, "mp3").unwrap();
    assert!(!repaired.is_empty());
    assert_eq!(repaired[0], 0xFF);
}

#[test]
fn mp3_valid_idempotent() {
    let mp3 = make_mp3();
    let r1 = repair_binary_with_format(&mp3, "mp3").unwrap();
    let r2 = repair_binary_with_format(&r1, "mp3").unwrap();
    assert_eq!(r1, r2);
}

#[test]
fn mp3_strip_trailing_nulls() {
    let mut mp3 = make_mp3();
    mp3.extend_from_slice(&[0; 50]);
    let repaired = repair_binary_with_format(&mp3, "mp3").unwrap();
    assert!(repaired.len() <= mp3.len());
}

#[test]
fn mp3_confidence_for_frame_sync() {
    let r = create_binary_repairer("mp3").unwrap();
    let data = [0xFF, 0xFB, 0x90, 0x64];
    assert!(r.confidence(&data) > 0.8);
}

#[test]
fn mp3_validator_accepts_id3() {
    let v = create_binary_validator("mp3").unwrap();
    let mp3 = make_mp3();
    assert!(v.is_valid(&mp3));
}

// ─── MP4/MOV corruption tests ──────────────────────────────────────────────

#[test]
fn mp4_corrupt_moov_box_size() {
    let mut mp4 = make_mp4();
    // moov box starts at offset 24 (after ftyp)
    // Set moov size to 0xFFFFFFFF (huge)
    mp4[24..28].copy_from_slice(&0xFFFFFFFFu32.to_be_bytes());
    let repaired = repair_binary_with_format(&mp4, "mp4").unwrap();
    assert!(!repaired.is_empty());
}

#[test]
fn mp4_truncated_in_mdat() {
    let mut mp4 = make_mp4();
    mp4.truncate(mp4.len() - 75);
    let repaired = repair_binary_with_format(&mp4, "mp4").unwrap();
    // Should still have valid box structure
    assert!(repaired.len() >= 24); // at least ftyp
}

#[test]
fn mp4_missing_ftyp_prepended() {
    let mp4 = make_mp4();
    let no_ftyp = &mp4[24..]; // skip ftyp
    let repaired = repair_binary_with_format(no_ftyp, "mp4").unwrap();
    // ftyp should be prepended
    assert_eq!(&repaired[4..8], b"ftyp");
}

#[test]
fn mp4_remove_large_free_box() {
    let mut mp4 = make_mp4();
    // Add a 1.5MB free box
    let free_size = 8u32 + 1_500_000u32;
    mp4.extend_from_slice(&free_size.to_be_bytes());
    mp4.extend_from_slice(b"free");
    mp4.extend_from_slice(&vec![0; 1_500_000]);
    let repaired = repair_binary_with_format(&mp4, "mp4").unwrap();
    assert!(repaired.len() < mp4.len());
    // Should not contain the free box
    assert!(!repaired.windows(4).any(|w| w == b"free" && repaired.len() > 1_500_000));
}

#[test]
fn mp4_valid_idempotent() {
    let mp4 = make_mp4();
    let r1 = repair_binary_with_format(&mp4, "mp4").unwrap();
    let r2 = repair_binary_with_format(&r1, "mp4").unwrap();
    assert_eq!(r1, r2);
}

#[test]
fn mp4_confidence_high_for_valid() {
    let mp4 = make_mp4();
    let r = create_binary_repairer("mp4").unwrap();
    assert!(r.confidence(&mp4) > 0.9);
}

#[test]
fn mov_missing_ftyp_gets_qt_brand() {
    let mut mov = Vec::new();
    mov.extend_from_slice(&12u32.to_be_bytes());
    mov.extend_from_slice(b"moov");
    mov.extend_from_slice(&[0; 4]);
    mov.extend_from_slice(&58u32.to_be_bytes());
    mov.extend_from_slice(b"mdat");
    mov.extend_from_slice(&[0; 50]);
    let repaired = repair_binary_with_format(&mov, "mov").unwrap();
    assert_eq!(&repaired[4..8], b"ftyp");
    assert_eq!(&repaired[8..12], b"qt  ");
}

#[test]
fn mov_valid_passes_through() {
    let mut mov = Vec::new();
    mov.extend_from_slice(&24u32.to_be_bytes());
    mov.extend_from_slice(b"ftyp");
    mov.extend_from_slice(b"qt  ");
    mov.extend_from_slice(&0u32.to_be_bytes());
    mov.extend_from_slice(b"qt  isom");
    mov.extend_from_slice(&12u32.to_be_bytes());
    mov.extend_from_slice(b"moov");
    mov.extend_from_slice(&[0; 4]);
    let repaired = repair_binary_with_format(&mov, "mov").unwrap();
    assert_eq!(repaired, mov);
}

// ─── Auto-detection tests ──────────────────────────────────────────────────

#[test]
fn auto_detect_corrupted_png() {
    let mut png = make_png();
    png[0] = 0x00;
    png[1] = b'P';
    png[2] = b'N';
    png[3] = b'G';
    let detected = detect_binary_format(&png);
    assert_eq!(detected.map(|d| d.format), Some("png"));
}

#[test]
fn auto_detect_corrupted_wav_just_riff() {
    let data = b"RIFF\x00\x00\x00\x00XXXX";
    let detected = detect_binary_format(data);
    assert_eq!(detected.map(|d| d.format), Some("wav"));
}

#[test]
fn auto_detect_unknown_returns_none() {
    let data = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
    assert!(detect_binary_format(&data).is_none());
}

#[test]
fn auto_detect_empty_returns_none() {
    assert!(detect_binary_format(&[]).is_none());
}

#[test]
fn auto_detect_short_data_returns_none() {
    assert!(detect_binary_format(&[0x89, 0x50]).is_none());
}

#[test]
fn auto_detect_mp4_with_unknown_brand() {
    let mut mp4 = vec![0u8; 24];
    mp4[0..4].copy_from_slice(&24u32.to_be_bytes());
    mp4[4..8].copy_from_slice(b"ftyp");
    mp4[8..12].copy_from_slice(b"xxxx"); // unknown brand
    let detected = detect_binary_format(&mp4);
    assert_eq!(detected.map(|d| d.format), Some("mp4"));
    assert!(detected.unwrap().confidence < 0.9);
}

#[test]
fn auto_detect_mov_with_qt_brand() {
    let mut mov = vec![0u8; 24];
    mov[0..4].copy_from_slice(&24u32.to_be_bytes());
    mov[4..8].copy_from_slice(b"ftyp");
    mov[8..12].copy_from_slice(b"qt  ");
    let detected = detect_binary_format(&mov);
    assert_eq!(detected.map(|d| d.format), Some("mov"));
}

#[test]
fn auto_detect_mp4_no_ftyp_but_has_moov() {
    let mut data = vec![0u8; 16];
    data[0..4].copy_from_slice(&12u32.to_be_bytes());
    data[4..8].copy_from_slice(b"moov");
    data[8..12].copy_from_slice(&[0; 4]);
    let detected = detect_binary_format(&data);
    assert_eq!(detected.map(|d| d.format), Some("mp4"));
}

// ─── Round-trip: repair → validate → repair (idempotency) ──────────────────

#[test]
fn round_trip_png_corrupt_then_repair_twice() {
    let mut png = make_png();
    png[29] ^= 0xFF; // corrupt IHDR CRC
    png.truncate(png.len() - 12); // remove IEND
    let r1 = repair_binary_with_format(&png, "png").unwrap();
    let v = create_binary_validator("png").unwrap();
    assert!(v.is_valid(&r1));
    let r2 = repair_binary_with_format(&r1, "png").unwrap();
    assert_eq!(r1, r2);
}

#[test]
fn round_trip_jpeg_multiple_corruptions() {
    let mut jpeg = make_jpeg();
    jpeg.truncate(jpeg.len() - 2); // remove EOI
    let mut corrupted = vec![0x00, 0x00]; // missing SOI
    corrupted.extend_from_slice(&jpeg[2..]);
    corrupted.extend_from_slice(&[0xDE, 0xAD]); // trailing garbage
    let r1 = repair_binary_with_format(&corrupted, "jpeg").unwrap();
    assert_eq!(&r1[..2], &[0xFF, 0xD8]);
    assert!(r1.ends_with(&[0xFF, 0xD9]));
    let r2 = repair_binary_with_format(&r1, "jpeg").unwrap();
    assert_eq!(r1, r2);
}

#[test]
fn round_trip_wav_multiple_corruptions() {
    let mut wav = make_wav();
    wav[0..4].copy_from_slice(b"riff"); // wrong case
    wav[4..8].copy_from_slice(&[0; 4]); // wrong size
    wav.truncate(wav.len() - 30); // truncated
    let r1 = repair_binary_with_format(&wav, "wav").unwrap();
    let v = create_binary_validator("wav").unwrap();
    assert!(v.is_valid(&r1));
    let r2 = repair_binary_with_format(&r1, "wav").unwrap();
    assert_eq!(r1, r2);
}

#[test]
fn round_trip_mp4_multiple_corruptions() {
    let mut mp4 = make_mp4();
    mp4[24..28].copy_from_slice(&0xFFFFFFFFu32.to_be_bytes()); // bad moov size
    mp4.truncate(mp4.len() - 80); // truncated mdat
    let r1 = repair_binary_with_format(&mp4, "mp4").unwrap();
    let r2 = repair_binary_with_format(&r1, "mp4").unwrap();
    assert_eq!(r1, r2);
}

// ─── Cross-format tests ────────────────────────────────────────────────────

#[test]
fn repair_binary_auto_detect_all_formats() {
    let samples: Vec<(&str, Vec<u8>)> = vec![
        ("png", make_png()),
        ("jpeg", make_jpeg()),
        ("wav", make_wav()),
        ("mp3", make_mp3()),
        ("mp4", make_mp4()),
    ];

    for (expected_fmt, data) in &samples {
        let repaired = repair_binary(data).unwrap();
        let detected = detect_binary_format(&repaired);
        assert_eq!(
            detected.map(|d| d.format),
            Some(*expected_fmt),
            "Format {} should be detected after repair",
            expected_fmt
        );
    }
}

#[test]
fn all_formats_validator_round_trip() {
    let samples: Vec<(&str, Vec<u8>)> = vec![
        ("png", make_png()),
        ("jpeg", make_jpeg()),
        ("wav", make_wav()),
        ("mp4", make_mp4()),
    ];

    for (fmt, data) in &samples {
        let repaired = repair_binary_with_format(data, fmt).unwrap();
        let v = create_binary_validator(fmt).unwrap();
        assert!(
            v.is_valid(&repaired),
            "Format {} should pass validation after repair",
            fmt
        );
    }
}

// ─── Edge cases ────────────────────────────────────────────────────────────

#[test]
fn very_small_png() {
    // Just signature + IEND (no IHDR — technically invalid but should not crash)
    let mut png = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    png.extend_from_slice(&0u32.to_be_bytes());
    png.extend_from_slice(b"IEND");
    png.extend_from_slice(&crc32(b"IEND").to_be_bytes());
    let repaired = repair_binary_with_format(&png, "png").unwrap();
    assert!(!repaired.is_empty());
}

#[test]
fn very_small_jpeg() {
    // Just SOI + EOI
    let jpeg = vec![0xFF, 0xD8, 0xFF, 0xD9];
    let repaired = repair_binary_with_format(&jpeg, "jpeg").unwrap();
    assert_eq!(repaired, jpeg);
}

#[test]
fn very_small_wav() {
    // Minimal: RIFF header + WAVE (no chunks)
    let mut wav = Vec::new();
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&4u32.to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    let repaired = repair_binary_with_format(&wav, "wav").unwrap();
    let v = create_binary_validator("wav").unwrap();
    assert!(v.is_valid(&repaired));
}

#[test]
fn very_small_mp4() {
    // Just ftyp box — no moov/mdat, so repair won't make it valid.
    // But it should not crash and should return data.
    let mut mp4 = Vec::new();
    mp4.extend_from_slice(&24u32.to_be_bytes());
    mp4.extend_from_slice(b"ftyp");
    mp4.extend_from_slice(b"isom");
    mp4.extend_from_slice(&0u32.to_be_bytes());
    mp4.extend_from_slice(b"isommp41");
    let repaired = repair_binary_with_format(&mp4, "mp4").unwrap();
    assert!(!repaired.is_empty());
    assert_eq!(&repaired[4..8], b"ftyp");
}

#[test]
fn format_alias_jpg_works() {
    let jpeg = make_jpeg();
    let repaired = repair_binary_with_format(&jpeg, "jpg").unwrap();
    assert!(!repaired.is_empty());
}

#[test]
fn unknown_binary_format_errors() {
    assert!(create_binary_repairer("xyz").is_err());
    assert!(create_binary_validator("xyz").is_err());
}

#[test]
fn empty_input_all_formats_error() {
    for fmt in &["png", "jpeg", "wav", "mp3", "mp4", "mov"] {
        let result = repair_binary_with_format(&[], fmt);
        assert!(result.is_err(), "Empty input should error for {}", fmt);
    }
}

#[test]
fn random_data_auto_detect_returns_original() {
    let data = vec![0x42; 100];
    let result = repair_binary(&data).unwrap();
    assert_eq!(result, data);
}

// ─── Never-panics sweep with heavy corruption ──────────────────────────────

#[test]
fn never_panics_heavy_corruption() {
    let samples = vec![
        ("png", make_png()),
        ("jpeg", make_jpeg()),
        ("wav", make_wav()),
        ("mp3", make_mp3()),
        ("mp4", make_mp4()),
    ];

    for (fmt, data) in &samples {
        // Truncate at various points
        for cut in [1, 5, 10, data.len() / 2, data.len() - 1] {
            let truncated = &data[..cut.min(data.len())];
            let _ = repair_binary_with_format(truncated, fmt);
        }

        // Flip random bytes
        let mut corrupted = data.clone();
        for i in (0..corrupted.len()).step_by(7) {
            corrupted[i] ^= 0xFF;
        }
        let _ = repair_binary_with_format(&corrupted, fmt);

        // Add trailing garbage
        let mut with_garbage = data.clone();
        with_garbage.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF].repeat(20));
        let _ = repair_binary_with_format(&with_garbage, fmt);

        // Prepend garbage
        let mut prefixed = vec![0x00, 0x01, 0x02, 0x03];
        prefixed.extend_from_slice(data);
        let _ = repair_binary_with_format(&prefixed, fmt);
    }
}

// ─── Confidence scoring tests ──────────────────────────────────────────────

#[test]
fn confidence_zero_for_empty() {
    for fmt in &["png", "jpeg", "wav", "mp3", "mp4", "mov"] {
        let r = create_binary_repairer(fmt).unwrap();
        assert_eq!(r.confidence(&[]), 0.0, "Confidence should be 0 for empty input ({})", fmt);
    }
}

#[test]
fn confidence_high_for_valid_formats() {
    let samples: Vec<(&str, Vec<u8>)> = vec![
        ("png", make_png()),
        ("jpeg", make_jpeg()),
        ("wav", make_wav()),
        ("mp3", make_mp3()),
        ("mp4", make_mp4()),
    ];

    for (fmt, data) in &samples {
        let r = create_binary_repairer(fmt).unwrap();
        assert!(
            r.confidence(data) > 0.8,
            "Confidence should be high for valid {} ({:.2})",
            fmt,
            r.confidence(data)
        );
    }
}

// ─── Validator tests ───────────────────────────────────────────────────────

#[test]
fn validators_reject_empty() {
    for fmt in &["png", "jpeg", "wav", "mp3", "mp4"] {
        let v = create_binary_validator(fmt).unwrap();
        assert!(!v.is_valid(&[]), "Validator should reject empty for {}", fmt);
    }
}

#[test]
fn validators_reject_random_data() {
    let data = vec![0x42; 100];
    for fmt in &["png", "jpeg", "wav", "mp4"] {
        let v = create_binary_validator(fmt).unwrap();
        assert!(!v.is_valid(&data), "Validator should reject random data for {}", fmt);
    }
}
