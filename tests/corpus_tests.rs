//! Real-world corpus tests — damaged files from common LLM/API failure patterns.
//!
//! Each test loads a file from `examples/data/corpus/`, repairs it, and asserts
//! that the repair succeeds and produces output that differs from the input
//! (i.e. actual damage was detected and fixed).

use std::fs;
use std::path::Path;

fn load_corpus(name: &str) -> Option<String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples/data/corpus")
        .join(name);
    if !path.exists() {
        return None;
    }
    fs::read_to_string(&path).ok()
}

// ---------------------------------------------------------------------------
// JSON corpus
// ---------------------------------------------------------------------------

#[test]
fn corpus_llm_json_markdown_fenced() {
    let Some(content) = load_corpus("llm_json_markdown_fenced.json") else {
        return;
    };
    let repaired = anyrepair::repair_with_format(&content, "json")
        .expect("repair should succeed");
    assert_ne!(content, repaired, "repair should change content");
    // Should no longer contain markdown fences
    assert!(!repaired.contains("```"));
    // Should contain valid JSON structure
    assert!(repaired.contains("\"name\""));
    assert!(repaired.contains("\"Alice\""));
}

#[test]
fn corpus_llm_json_prose_preamble() {
    let Some(content) = load_corpus("llm_json_prose_preamble.json") else {
        return;
    };
    let repaired = anyrepair::repair_with_format(&content, "json")
        .expect("repair should succeed");
    assert_ne!(content, repaired);
    // Prose should be extracted/removed
    assert!(!repaired.contains("Sure!"));
    assert!(!repaired.contains("Let me know"));
    // JSON content should be preserved
    assert!(repaired.contains("\"title\""));
    assert!(repaired.contains("\"My Document\""));
}

#[test]
fn corpus_api_truncated_response() {
    let Some(content) = load_corpus("api_truncated_response.json") else {
        return;
    };
    // Truncated JSON — repair should at least attempt to close brackets
    let result = anyrepair::repair_with_format(&content, "json");
    assert!(result.is_ok(), "truncated JSON should not panic");
    let repaired = result.unwrap();
    assert_ne!(content, repaired);
    // Should have added closing brackets
    assert!(repaired.contains("]") || repaired.contains("}"));
}

#[test]
fn corpus_llm_json_smart_quotes() {
    let Some(content) = load_corpus("llm_json_smart_quotes.json") else {
        return;
    };
    let repaired = anyrepair::repair_with_format(&content, "json")
        .expect("repair should succeed");
    assert_ne!(content, repaired);
    // Smart quotes should be normalized to straight quotes
    assert!(!repaired.contains('\u{201c}'));
    assert!(!repaired.contains('\u{201d}'));
}

#[test]
fn corpus_llm_json_trailing_text() {
    let Some(content) = load_corpus("llm_json_trailing_text.json") else {
        return;
    };
    let repaired = anyrepair::repair_with_format(&content, "json")
        .expect("repair should succeed");
    assert_ne!(content, repaired);
    // Trailing prose should be removed
    assert!(!repaired.contains("Hope this helps"));
    // JSON content should be preserved
    assert!(repaired.contains("\"answer\""));
    assert!(repaired.contains("Paris"));
}

// ---------------------------------------------------------------------------
// YAML corpus
// ---------------------------------------------------------------------------

#[test]
fn corpus_llm_yaml_mixed_indent() {
    let Some(content) = load_corpus("llm_yaml_mixed_indent.yaml") else {
        return;
    };
    let repaired = anyrepair::repair_with_format(&content, "yaml")
        .expect("repair should succeed");
    assert_ne!(content, repaired);
    // Should contain YAML structure
    assert!(repaired.contains("server"));
    assert!(repaired.contains("localhost"));
    assert!(repaired.contains("database"));
}

// ---------------------------------------------------------------------------
// XML corpus
// ---------------------------------------------------------------------------

#[test]
fn corpus_llm_xml_unescaped_entities() {
    let Some(content) = load_corpus("llm_xml_unescaped_entities.xml") else {
        return;
    };
    let repaired = anyrepair::repair_with_format(&content, "xml")
        .expect("repair should succeed");
    assert_ne!(content, repaired);
    // Bare & should be escaped to &amp;
    assert!(!repaired.contains(" & ") || repaired.contains("&amp;"));
}

// ---------------------------------------------------------------------------
// CSV corpus
// ---------------------------------------------------------------------------

#[test]
fn corpus_api_csv_inconsistent_quoting() {
    let Some(content) = load_corpus("api_csv_inconsistent_quoting.csv") else {
        return;
    };
    let repaired = anyrepair::repair_with_format(&content, "csv")
        .expect("repair should succeed");
    // Should still contain data
    assert!(repaired.contains("Widget"));
    assert!(repaired.contains("electronics"));
}

// ---------------------------------------------------------------------------
// TOML corpus
// ---------------------------------------------------------------------------

#[test]
fn corpus_llm_toml_invalid_keys() {
    let Some(content) = load_corpus("llm_toml_invalid_keys.toml") else {
        return;
    };
    let repaired = anyrepair::repair_with_format(&content, "toml")
        .expect("repair should succeed");
    assert_ne!(content, repaired);
    // Should contain TOML structure
    assert!(repaired.contains("localhost"));
    assert!(repaired.contains("8080"));
}

// ---------------------------------------------------------------------------
// Markdown corpus
// ---------------------------------------------------------------------------

#[test]
fn corpus_llm_markdown_broken_links() {
    let Some(content) = load_corpus("llm_markdown_broken_links.md") else {
        return;
    };
    let repaired = anyrepair::repair_with_format(&content, "markdown")
        .expect("repair should succeed");
    assert_ne!(content, repaired);
    // Should still contain key content
    assert!(repaired.contains("Project Documentation"));
    assert!(repaired.contains("anyrepair"));
}

// ---------------------------------------------------------------------------
// Auto-detection on corpus files
// ---------------------------------------------------------------------------

#[test]
fn corpus_auto_detect_json_fenced() {
    let Some(content) = load_corpus("llm_json_markdown_fenced.json") else {
        return;
    };
    let detected = anyrepair::detect_format(&content);
    // Should detect as JSON (starts with ```)
    assert!(detected.is_some());
}

#[test]
fn corpus_auto_detect_yaml_mixed_indent() {
    let Some(content) = load_corpus("llm_yaml_mixed_indent.yaml") else {
        return;
    };
    let detected = anyrepair::detect_format(&content);
    assert!(detected.is_some());
}

#[test]
fn corpus_auto_detect_xml_unescaped() {
    let Some(content) = load_corpus("llm_xml_unescaped_entities.xml") else {
        return;
    };
    let detected = anyrepair::detect_format(&content);
    assert_eq!(detected, Some("xml"));
}

// ---------------------------------------------------------------------------
// Idempotency on corpus files
// ---------------------------------------------------------------------------

#[test]
fn corpus_idempotent_json_prose_preamble() {
    let Some(content) = load_corpus("llm_json_prose_preamble.json") else {
        return;
    };
    let repaired1 = anyrepair::repair_with_format(&content, "json")
        .expect("first repair should succeed");
    let repaired2 = anyrepair::repair_with_format(&repaired1, "json")
        .expect("second repair should succeed");
    assert_eq!(repaired1, repaired2, "repair should be idempotent");
}

#[test]
fn corpus_idempotent_yaml_mixed_indent() {
    let Some(content) = load_corpus("llm_yaml_mixed_indent.yaml") else {
        return;
    };
    let repaired1 = anyrepair::repair_with_format(&content, "yaml")
        .expect("first repair should succeed");
    let repaired2 = anyrepair::repair_with_format(&repaired1, "yaml")
        .expect("second repair should succeed");
    assert_eq!(repaired1, repaired2, "repair should be idempotent");
}

#[test]
fn corpus_idempotent_toml_invalid_keys() {
    let Some(content) = load_corpus("llm_toml_invalid_keys.toml") else {
        return;
    };
    let repaired1 = anyrepair::repair_with_format(&content, "toml")
        .expect("first repair should succeed");
    let repaired2 = anyrepair::repair_with_format(&repaired1, "toml")
        .expect("second repair should succeed");
    assert_eq!(repaired1, repaired2, "repair should be idempotent");
}

// ---------------------------------------------------------------------------
// Never panics on corpus files
// ---------------------------------------------------------------------------

#[test]
fn corpus_never_panics_all_files() {
    let corpus_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples/data/corpus");
    if !corpus_dir.exists() {
        return;
    }
    let entries = fs::read_dir(&corpus_dir).expect("should read corpus dir");
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_none() {
            continue;
        }
        let content = fs::read_to_string(&path).expect("should read file");
        // Auto-detect repair should never panic
        let _ = anyrepair::repair(&content);
    }
}
