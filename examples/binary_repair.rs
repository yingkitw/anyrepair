//! Example: Binary media file repair with AnyRepair
//!
//! Run with: cargo run --example binary_repair
//!
//! Demonstrates repairing corrupted PNG, JPEG, WAV, MP3, MP4, and MOV files
//! using the BinaryRepair trait and auto-detection.

use anyrepair::{
    binary_detection::detect_binary_format, create_binary_repairer, create_binary_validator,
    repair_binary, repair_binary_with_format,
};

fn main() {
    println!("=== AnyRepair Binary Media Repair Examples ===\n");

    // ─── PNG ───────────────────────────────────────────────────────────────
    println!("--- PNG Repair ---");
    let mut corrupted_png = make_valid_png();

    // Corrupt: remove IEND chunk (last 12 bytes)
    println!("Corruption: removed IEND chunk");
    corrupted_png.truncate(corrupted_png.len() - 12);

    let detected = detect_binary_format(&corrupted_png);
    println!(
        "Detected: {:?}",
        detected.map(|d| (d.format, d.confidence))
    );

    let repaired = repair_binary(&corrupted_png).unwrap();
    let validator = create_binary_validator("png").unwrap();
    println!(
        "Repaired valid: {} ({} -> {} bytes)\n",
        validator.is_valid(&repaired),
        corrupted_png.len(),
        repaired.len()
    );

    // ─── JPEG ──────────────────────────────────────────────────────────────
    println!("--- JPEG Repair ---");
    let mut corrupted_jpeg = make_valid_jpeg();

    // Corrupt: remove EOI marker (last 2 bytes)
    println!("Corruption: removed EOI marker");
    corrupted_jpeg.truncate(corrupted_jpeg.len() - 2);

    let repaired = repair_binary_with_format(&corrupted_jpeg, "jpeg").unwrap();
    println!(
        "Repaired ends with EOI: {} ({} -> {} bytes)\n",
        repaired.ends_with(&[0xFF, 0xD9]),
        corrupted_jpeg.len(),
        repaired.len()
    );

    // ─── WAV ───────────────────────────────────────────────────────────────
    println!("--- WAV Repair ---");
    let mut corrupted_wav = make_valid_wav();

    // Corrupt: zero out the RIFF size
    println!("Corruption: zeroed RIFF size field");
    corrupted_wav[4..8].copy_from_slice(&[0, 0, 0, 0]);

    let repaired = repair_binary_with_format(&corrupted_wav, "wav").unwrap();
    let validator = create_binary_validator("wav").unwrap();
    println!(
        "Repaired valid: {} ({} -> {} bytes)\n",
        validator.is_valid(&repaired),
        corrupted_wav.len(),
        repaired.len()
    );

    // ─── MP3 ───────────────────────────────────────────────────────────────
    println!("--- MP3 Repair ---");
    let mut corrupted_mp3 = make_valid_mp3();

    // Corrupt: insert garbage before frame sync
    println!("Corruption: inserted garbage before frame sync");
    let mut garbage = vec![0xDE, 0xAD, 0xBE, 0xEF];
    garbage.extend_from_slice(&corrupted_mp3[10..]); // after ID3 header
    corrupted_mp3.truncate(10);
    corrupted_mp3.extend_from_slice(&garbage);

    let repaired = repair_binary_with_format(&corrupted_mp3, "mp3").unwrap();
    println!(
        "Repaired starts with frame sync: {} ({} -> {} bytes)\n",
        repaired.len() > 10 && repaired[10] == 0xFF,
        corrupted_mp3.len(),
        repaired.len()
    );

    // ─── MP4 ───────────────────────────────────────────────────────────────
    println!("--- MP4 Repair ---");
    let mut corrupted_mp4 = make_valid_mp4();

    // Corrupt: truncate in the middle of mdat
    println!("Corruption: truncated mdat box");
    corrupted_mp4.truncate(corrupted_mp4.len() - 50);

    let repaired = repair_binary_with_format(&corrupted_mp4, "mp4").unwrap();
    println!(
        "Repaired size: {} bytes (was {} bytes)\n",
        repaired.len(),
        corrupted_mp4.len()
    );

    // ─── MOV ───────────────────────────────────────────────────────────────
    println!("--- MOV Repair ---");
    let corrupted_mov = make_valid_mov_without_ftyp();

    println!("Corruption: missing ftyp box");
    let repaired = repair_binary_with_format(&corrupted_mov, "mov").unwrap();
    println!(
        "Repaired has ftyp: {} ({} -> {} bytes)\n",
        repaired.len() >= 12 && &repaired[4..8] == b"ftyp",
        corrupted_mov.len(),
        repaired.len()
    );

    // ─── Using the trait directly ──────────────────────────────────────────
    println!("--- Using BinaryRepair trait directly ---");
    let png = make_valid_png();
    let repairer = create_binary_repairer("png").unwrap();
    let needs = repairer.needs_repair(&png);
    let conf = repairer.confidence(&png);
    println!(
        "Valid PNG: needs_repair={}, confidence={:.0}%",
        !needs,
        conf * 100.0
    );

    // ─── Auto-detect all formats ───────────────────────────────────────────
    println!("\n--- Auto-detection sweep ---");
    let samples: &[(&str, &[u8])] = &[
        ("PNG", &make_valid_png()),
        ("JPEG", &make_valid_jpeg()),
        ("WAV", &make_valid_wav()),
        ("MP3", &make_valid_mp3()),
        ("MP4", &make_valid_mp4()),
    ];

    for (name, data) in samples {
        let detected = detect_binary_format(data);
        match detected {
            Some(d) => println!(
                "  {}: detected as {} (confidence: {:.0}%)",
                name,
                d.format,
                d.confidence * 100.0
            ),
            None => println!("  {}: NOT detected", name),
        }
    }

    println!("\n=== All examples completed successfully ===");
}

// ─── Helpers to generate valid media files ─────────────────────────────────

fn make_valid_png() -> Vec<u8> {
    let mut png = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    // IHDR: 2x2 RGB
    let ihdr = [0, 0, 0, 2, 0, 0, 0, 2, 8, 2, 0, 0, 0];
    png.extend_from_slice(&13u32.to_be_bytes());
    png.extend_from_slice(b"IHDR");
    png.extend_from_slice(&ihdr);
    let crc = crc32_concat(b"IHDR", &ihdr);
    png.extend_from_slice(&crc.to_be_bytes());
    // IEND
    png.extend_from_slice(&0u32.to_be_bytes());
    png.extend_from_slice(b"IEND");
    let iend_crc = crc32_concat(b"IEND", &[]);
    png.extend_from_slice(&iend_crc.to_be_bytes());
    png
}

fn crc32_concat(a: &[u8], b: &[u8]) -> u32 {
    let mut combined = Vec::new();
    combined.extend_from_slice(a);
    combined.extend_from_slice(b);
    let mut crc = 0xFFFFFFFFu32;
    for &byte in &combined {
        crc = CRC_TABLE[((crc ^ byte as u32) & 0xFF) as usize] ^ (crc >> 8);
    }
    crc ^ 0xFFFFFFFF
}

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

fn make_valid_jpeg() -> Vec<u8> {
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

fn make_valid_wav() -> Vec<u8> {
    let mut w = Vec::new();
    w.extend_from_slice(b"RIFF");
    w.extend_from_slice(&(36 + 100u32).to_le_bytes());
    w.extend_from_slice(b"WAVE");
    w.extend_from_slice(b"fmt ");
    w.extend_from_slice(&16u32.to_le_bytes());
    w.extend_from_slice(&1u16.to_le_bytes());
    w.extend_from_slice(&1u16.to_le_bytes());
    w.extend_from_slice(&8000u32.to_le_bytes());
    w.extend_from_slice(&8000u32.to_le_bytes());
    w.extend_from_slice(&1u16.to_le_bytes());
    w.extend_from_slice(&8u16.to_le_bytes());
    w.extend_from_slice(b"data");
    w.extend_from_slice(&100u32.to_le_bytes());
    w.extend_from_slice(&[0x80; 100]);
    w
}

fn make_valid_mp3() -> Vec<u8> {
    let mut m = Vec::new();
    m.extend_from_slice(b"ID3\x03\x00\x00\x00\x00\x00\x00");
    m.extend_from_slice(&[0xFF, 0xFB, 0x90, 0x64]);
    m.extend_from_slice(&[0; 100]);
    m
}

fn make_valid_mp4() -> Vec<u8> {
    let mut m = Vec::new();
    // ftyp: 8 + 16 = 24
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

fn make_valid_mov_without_ftyp() -> Vec<u8> {
    let mut m = Vec::new();
    // moov
    m.extend_from_slice(&12u32.to_be_bytes());
    m.extend_from_slice(b"moov");
    m.extend_from_slice(&[0; 4]);
    // mdat
    m.extend_from_slice(&58u32.to_be_bytes());
    m.extend_from_slice(b"mdat");
    m.extend_from_slice(&[0; 50]);
    m
}
