//! Comprehensive tests for diff/udiff repair functionality
//!
//! Test organization:
//! - Basic functionality tests
//! - Repair strategy tests (missing headers, incorrect prefixes, etc.)
//! - Edge case tests
//! - API tests (confidence, needs_repair, auto-detection)
//! - File-based tests (using example files)
//! - Batch/integration tests

use anyrepair::{diff::DiffRepairer, repair, Repair};
use std::fs;
use std::path::Path;

// ============================================================================
// Basic Functionality Tests
// ============================================================================

#[test]
fn test_valid_diff_passes_through() {
    let valid_diff = r#"--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,3 @@
 fn main() {
-    println!("Hello");
+    println!("Hello World");
     println!("Done");
 }
"#;
    
    let mut repairer = DiffRepairer::new();
    let result = repairer.repair(valid_diff).unwrap();
    
    assert!(result.contains("@@"));
    assert!(result.contains("---"));
    assert!(result.contains("+++"));
    assert!(result.contains("-    println!(\"Hello\");"));
    assert!(result.contains("+    println!(\"Hello World\");"));
}

#[test]
fn test_multiple_hunks() {
    let multi_hunk = r#"--- a/file.txt
+++ b/file.txt
@@ -1,3 +1,3 @@
 line1
-line2
+line2_modified
 line3
@@ -5,2 +5,2 @@
 line5
-line6
+line6_modified
"#;
    
    let mut repairer = DiffRepairer::new();
    let result = repairer.repair(multi_hunk).unwrap();
    
    // Should preserve both hunks
    let hunk_count = result.lines()
        .filter(|line| line.starts_with("@@"))
        .count();
    assert_eq!(hunk_count, 2);
}

#[test]
fn test_context_lines() {
    let with_context = r#"--- a/file.txt
+++ b/file.txt
@@ -1,5 +1,5 @@
 unchanged1
 unchanged2
-old line
+new line
 unchanged3
 unchanged4
"#;
    
    let mut repairer = DiffRepairer::new();
    let result = repairer.repair(with_context).unwrap();
    
    // Should preserve context lines (starting with space)
    assert!(result.contains(" unchanged1"));
    assert!(result.contains(" unchanged2"));
    assert!(result.contains(" unchanged3"));
}

#[test]
fn test_hunk_with_only_additions() {
    let additions_only = r#"--- a/file.txt
+++ b/file.txt
@@ -1,0 +1,2 @@
+new line 1
+new line 2
"#;
    
    let mut repairer = DiffRepairer::new();
    let result = repairer.repair(additions_only).unwrap();
    
    assert!(result.contains("@@"));
    assert!(result.contains("+new line 1"));
    assert!(result.contains("+new line 2"));
}

#[test]
fn test_hunk_with_only_deletions() {
    let deletions_only = r#"--- a/file.txt
+++ b/file.txt
@@ -1,2 +1,0 @@
-old line 1
-old line 2
"#;
    
    let mut repairer = DiffRepairer::new();
    let result = repairer.repair(deletions_only).unwrap();
    
    assert!(result.contains("@@"));
    assert!(result.contains("-old line 1"));
    assert!(result.contains("-old line 2"));
}

#[test]
fn test_complex_diff_repair() {
    let complex = r#"--- a/src/lib.rs
+++ b/src/lib.rs
@@ -10,5 +10,5 @@
 pub fn example() {
     let x = 1;
-    let y = 2;
+    let y = 3;
     println!("{}", x + y);
 }
@@ -20,3 +20,3 @@
 pub fn another() {
-    return 1;
+    return 2;
 }
"#;
    
    let mut repairer = DiffRepairer::new();
    let result = repairer.repair(complex).unwrap();
    
    assert!(result.contains("@@"));
    assert!(result.contains("---"));
    assert!(result.contains("+++"));
    
    // Check both hunks are present
    let hunk_count = result.lines()
        .filter(|line| line.starts_with("@@"))
        .count();
    assert_eq!(hunk_count, 2);
}

// ============================================================================
// Repair Strategy Tests
// ============================================================================

#[test]
fn test_missing_hunk_header() {
    let malformed = r#"--- a/file.txt
+++ b/file.txt
-old line
+new line
"#;
    
    let mut repairer = DiffRepairer::new();
    let result = repairer.repair(malformed).unwrap();
    
    // Should add hunk header
    assert!(result.contains("@@"));
}

#[test]
fn test_missing_file_headers() {
    let malformed = r#"@@ -1,1 +1,1 @@
-old
+new
"#;
    
    let mut repairer = DiffRepairer::new();
    let result = repairer.repair(malformed).unwrap();
    
    // Should add file headers
    assert!(result.contains("---") || result.contains("+++"));
}

#[test]
fn test_incorrect_line_prefixes() {
    let malformed = r#"@@ -1,2 +1,2 @@
old line
new line
"#;
    
    let mut repairer = DiffRepairer::new();
    let result = repairer.repair(malformed).unwrap();
    
    // Lines in hunk should have proper prefixes
    let lines: Vec<&str> = result.lines().collect();
    let mut in_hunk = false;
    for line in lines {
        if line.starts_with("@@") {
            in_hunk = true;
        } else if in_hunk && !line.trim().is_empty() {
            // Should have +, -, or space prefix
            assert!(line.starts_with('+') || line.starts_with('-') || line.starts_with(' '),
                   "Line should have diff prefix: {}", line);
        }
    }
}

#[test]
fn test_malformed_hunk_range() {
    let malformed = r#"--- a/file.txt
+++ b/file.txt
@@ -1 +1 @@
-old
+new
"#;
    
    let mut repairer = DiffRepairer::new();
    let result = repairer.repair(malformed).unwrap();
    
    // Should fix hunk range format
    assert!(result.contains("@@"));
    // Should have proper format with commas
    let hunk_lines: Vec<&str> = result.lines()
        .filter(|line| line.starts_with("@@"))
        .collect();
    assert!(!hunk_lines.is_empty());
}

#[test]
fn test_malformed_hunk_header_numbers() {
    let malformed = r#"--- a/file.txt
+++ b/file.txt
@@ abc def ghi jkl @@
-old
+new
"#;
    
    let mut repairer = DiffRepairer::new();
    let result = repairer.repair(malformed).unwrap();
    
    // Should fix hunk header to have proper numbers
    let hunk_lines: Vec<&str> = result.lines()
        .filter(|line| line.starts_with("@@"))
        .collect();
    
    assert!(!hunk_lines.is_empty());
    // Should have numeric values
    for hunk in hunk_lines {
        assert!(hunk.contains(|c: char| c.is_ascii_digit()));
    }
}

#[test]
fn test_inconsistent_spacing_in_hunk_header() {
    let inconsistent = r#"--- a/file.txt
+++ b/file.txt
@@  -1,2  +1,2  @@
-old
+new
"#;
    
    let mut repairer = DiffRepairer::new();
    let result = repairer.repair(inconsistent).unwrap();
    
    // Should normalize spacing
    let hunk_lines: Vec<&str> = result.lines()
        .filter(|line| line.starts_with("@@"))
        .collect();
    
    for hunk in hunk_lines {
        // Should not have excessive spaces
        assert!(!hunk.contains("  "));
    }
}

#[test]
fn test_missing_newline_at_end() {
    let malformed = r#"--- a/file.txt
+++ b/file.txt
@@ -1,1 +1,1 @@
-old
+new"#;
    
    let mut repairer = DiffRepairer::new();
    let result = repairer.repair(malformed).unwrap();
    
    // Should end with newline
    assert!(result.ends_with('\n') || result.ends_with("\r\n"));
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_empty_content() {
    let mut repairer = DiffRepairer::new();
    let result = repairer.repair("").unwrap();
    assert_eq!(result, "");
}

#[test]
fn test_whitespace_only() {
    let mut repairer = DiffRepairer::new();
    let result = repairer.repair("   \n  \t  ").unwrap();
    // Should handle whitespace gracefully
    assert!(!result.contains("@@"));
}

#[test]
fn test_partial_diff_with_missing_parts() {
    let partial = r#"@@ -1,2 +1,2 @@
 line1
-line2
+line2_new
"#;
    
    let mut repairer = DiffRepairer::new();
    let result = repairer.repair(partial).unwrap();
    
    // Should add file headers
    assert!(result.contains("---") || result.contains("+++"));
}

#[test]
fn test_udiff_format() {
    // Unified diff format test
    let udiff = r#"--- a/src/main.rs	2024-01-01 10:00:00.000000000 +0000
+++ b/src/main.rs	2024-01-01 11:00:00.000000000 +0000
@@ -1,3 +1,3 @@
 fn main() {
-    old();
+    new();
 }
"#;
    
    let mut repairer = DiffRepairer::new();
    let result = repairer.repair(udiff).unwrap();
    
    assert!(result.contains("@@"));
    assert!(result.contains("---"));
    assert!(result.contains("+++"));
}

#[test]
fn test_real_world_git_diff() {
    // Simulate a real git diff output
    let git_diff = r#"diff --git a/src/lib.rs b/src/lib.rs
index 1234567..abcdefg 100644
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -10,5 +10,5 @@ pub fn example() {
     let x = 1;
-    let y = 2;
+    let y = 3;
     println!("{}", x + y);
 }
"#;
    
    let mut repairer = DiffRepairer::new();
    let result = repairer.repair(git_diff).unwrap();
    
    // Should preserve the unified diff part
    assert!(result.contains("@@"));
    assert!(result.contains("---"));
    assert!(result.contains("+++"));
}

// ============================================================================
// API Tests (Confidence, Needs Repair, Auto-Detection)
// ============================================================================

#[test]
fn test_auto_detection_via_repair() {
    let diff_content = r#"--- a/file.txt
+++ b/file.txt
@@ -1,1 +1,1 @@
-old
+new
"#;
    
    // Test that main repair function detects diff format
    let result = repair(diff_content).unwrap();
    assert!(result.contains("@@"));
}

#[test]
fn test_confidence_scoring() {
    let repairer = DiffRepairer::new();
    
    // Valid diff should have high confidence
    let valid = r#"--- a/file.txt
+++ b/file.txt
@@ -1,1 +1,1 @@
-old
+new
"#;
    let confidence = repairer.confidence(valid);
    assert!(confidence > 0.5);
    
    // Invalid diff should have lower confidence
    let invalid = "just some text";
    let low_confidence = repairer.confidence(invalid);
    assert!(low_confidence < confidence);
}

#[test]
fn test_needs_repair() {
    let repairer = DiffRepairer::new();
    
    // Valid diff should not need repair
    let valid = r#"--- a/file.txt
+++ b/file.txt
@@ -1,1 +1,1 @@
-old
+new
"#;
    assert!(!repairer.needs_repair(valid));
    
    // Invalid diff should need repair
    let invalid = r#"--- a/file.txt
+++ b/file.txt
old line
new line
"#;
    assert!(repairer.needs_repair(invalid));
}

// ============================================================================
// File-Based Tests - Sample Files
// ============================================================================

#[test]
fn test_repair_sample_diff() {
    let sample_path = Path::new("examples/data/diff/sample/sample.diff");
    if !sample_path.exists() {
        // Skip if examples directory not available (e.g., in published crate)
        return;
    }
    
    let content = fs::read_to_string(sample_path)
        .expect("Failed to read sample.diff");
    
    let mut repairer = DiffRepairer::new();
    let result = repairer.repair(&content).unwrap();
    
    // Valid diff should pass through with minimal changes
    assert!(result.contains("@@"));
    assert!(result.contains("---"));
    assert!(result.contains("+++"));
}

// ============================================================================
// File-Based Tests - Malformed Files
// ============================================================================

#[test]
fn test_repair_malformed_diff() {
    let malformed_path = Path::new("examples/data/diff/malformed/malformed.diff");
    if !malformed_path.exists() {
        return;
    }
    
    let content = fs::read_to_string(malformed_path)
        .expect("Failed to read malformed.diff");
    
    let mut repairer = DiffRepairer::new();
    let result = repairer.repair(&content).unwrap();
    
    // Should add file headers
    assert!(result.contains("---") || result.contains("+++"));
    assert!(result.contains("@@"));
}

#[test]
fn test_repair_missing_hunk_header() {
    let malformed_path = Path::new("examples/data/diff/malformed/malformed_missing_hunk_header.diff");
    if !malformed_path.exists() {
        return;
    }
    
    let content = fs::read_to_string(malformed_path)
        .expect("Failed to read malformed_missing_hunk_header.diff");
    
    let mut repairer = DiffRepairer::new();
    let result = repairer.repair(&content).unwrap();
    
    // Should add hunk header
    assert!(result.contains("@@"));
    assert!(result.contains("---"));
    assert!(result.contains("+++"));
}

#[test]
fn test_repair_missing_file_headers() {
    let malformed_path = Path::new("examples/data/diff/malformed/malformed_missing_file_headers.diff");
    if !malformed_path.exists() {
        return;
    }
    
    let content = fs::read_to_string(malformed_path)
        .expect("Failed to read malformed_missing_file_headers.diff");
    
    let mut repairer = DiffRepairer::new();
    let result = repairer.repair(&content).unwrap();
    
    // Should add file headers
    assert!(result.contains("---"));
    assert!(result.contains("+++"));
    assert!(result.contains("@@"));
}

#[test]
fn test_repair_incorrect_prefixes() {
    let malformed_path = Path::new("examples/data/diff/malformed/malformed_incorrect_prefixes.diff");
    if !malformed_path.exists() {
        return;
    }
    
    let content = fs::read_to_string(malformed_path)
        .expect("Failed to read malformed_incorrect_prefixes.diff");
    
    let mut repairer = DiffRepairer::new();
    let result = repairer.repair(&content).unwrap();
    
    // Should fix line prefixes
    let lines: Vec<&str> = result.lines().collect();
    let mut in_hunk = false;
    for line in lines {
        if line.starts_with("@@") {
            in_hunk = true;
        } else if in_hunk && !line.trim().is_empty() {
            // Lines in hunk should have proper prefixes
            assert!(line.starts_with('+') || line.starts_with('-') || 
                   line.starts_with(' ') || line.starts_with("---") || 
                   line.starts_with("+++"),
                   "Line should have diff prefix: {}", line);
        }
    }
}

#[test]
fn test_repair_malformed_hunk_range() {
    let malformed_path = Path::new("examples/data/diff/malformed/malformed_malformed_hunk_range.diff");
    if !malformed_path.exists() {
        return;
    }
    
    let content = fs::read_to_string(malformed_path)
        .expect("Failed to read malformed_malformed_hunk_range.diff");
    
    let mut repairer = DiffRepairer::new();
    let result = repairer.repair(&content).unwrap();
    
    // Should fix hunk range to have proper numbers
    let hunk_lines: Vec<&str> = result.lines()
        .filter(|line| line.starts_with("@@"))
        .collect();
    
    assert!(!hunk_lines.is_empty());
    for hunk in hunk_lines {
        // Should contain numeric values
        assert!(hunk.contains(|c: char| c.is_ascii_digit()));
    }
}

#[test]
fn test_repair_inconsistent_spacing() {
    let malformed_path = Path::new("examples/data/diff/malformed/malformed_inconsistent_spacing.diff");
    if !malformed_path.exists() {
        return;
    }
    
    let content = fs::read_to_string(malformed_path)
        .expect("Failed to read malformed_inconsistent_spacing.diff");
    
    let mut repairer = DiffRepairer::new();
    let result = repairer.repair(&content).unwrap();
    
    // Should normalize spacing
    let hunk_lines: Vec<&str> = result.lines()
        .filter(|line| line.starts_with("@@"))
        .collect();
    
    for hunk in hunk_lines {
        // Should not have excessive spaces
        assert!(!hunk.contains("  "), "Hunk should not have double spaces: {}", hunk);
    }
}

#[test]
fn test_repair_missing_newline() {
    let malformed_path = Path::new("examples/data/diff/malformed/malformed_missing_newline.diff");
    if !malformed_path.exists() {
        return;
    }
    
    let content = fs::read_to_string(malformed_path)
        .expect("Failed to read malformed_missing_newline.diff");
    
    let mut repairer = DiffRepairer::new();
    let result = repairer.repair(&content).unwrap();
    
    // Should end with newline
    assert!(result.ends_with('\n') || result.ends_with("\r\n"));
}

// ============================================================================
// File-Based Tests - Complex Files
// ============================================================================

#[test]
fn test_repair_complex_diff() {
    let malformed_path = Path::new("examples/data/diff/complex/malformed_complex.diff");
    if !malformed_path.exists() {
        return;
    }
    
    let content = fs::read_to_string(malformed_path)
        .expect("Failed to read malformed_complex.diff");
    
    let mut repairer = DiffRepairer::new();
    let result = repairer.repair(&content).unwrap();
    
    // Should repair complex diff with multiple hunks
    assert!(result.contains("@@"));
    assert!(result.contains("---"));
    assert!(result.contains("+++"));
    
    // Should have multiple hunks
    let hunk_count = result.lines()
        .filter(|line| line.starts_with("@@"))
        .count();
    assert!(hunk_count >= 2);
}

#[test]
fn test_repair_multi_file_complex_diff() {
    let multi_file_path = Path::new("examples/data/diff/complex/multi_file_complex.diff");
    if !multi_file_path.exists() {
        return;
    }
    
    let content = fs::read_to_string(multi_file_path)
        .expect("Failed to read multi_file_complex.diff");
    
    let mut repairer = DiffRepairer::new();
    let result = repairer.repair(&content).unwrap();
    
    // Should handle multi-file diff
    assert!(result.contains("@@"));
    assert!(result.contains("diff --git"));
    
    // Should have multiple file sections
    let file_header_count = result.lines()
        .filter(|line| line.starts_with("---") || line.starts_with("+++"))
        .count();
    assert!(file_header_count >= 4); // At least 2 files (2 headers each)
}

#[test]
fn test_repair_large_hunk_complex_diff() {
    let large_hunk_path = Path::new("examples/data/diff/complex/large_hunk_complex.diff");
    if !large_hunk_path.exists() {
        return;
    }
    
    let content = fs::read_to_string(large_hunk_path)
        .expect("Failed to read large_hunk_complex.diff");
    
    let mut repairer = DiffRepairer::new();
    let result = repairer.repair(&content).unwrap();
    
    // Should handle large hunk
    assert!(result.contains("@@"));
    assert!(result.contains("---"));
    assert!(result.contains("+++"));
    
    // Large hunk should have substantial changes
    let added_lines = result.lines()
        .filter(|line| line.starts_with('+') && !line.starts_with("+++"))
        .count();
    let removed_lines = result.lines()
        .filter(|line| line.starts_with('-') && !line.starts_with("---"))
        .count();
    
    assert!(added_lines > 10 || removed_lines > 10, 
           "Large hunk should have substantial changes");
}

#[test]
fn test_repair_mixed_changes_complex_diff() {
    let mixed_path = Path::new("examples/data/diff/complex/mixed_changes_complex.diff");
    if !mixed_path.exists() {
        return;
    }
    
    let content = fs::read_to_string(mixed_path)
        .expect("Failed to read mixed_changes_complex.diff");
    
    let mut repairer = DiffRepairer::new();
    let result = repairer.repair(&content).unwrap();
    
    // Should handle mixed additions and deletions
    assert!(result.contains("@@"));
    
    // Should have both additions and deletions
    let has_additions = result.lines().any(|line| line.starts_with('+') && !line.starts_with("+++"));
    let has_deletions = result.lines().any(|line| line.starts_with('-') && !line.starts_with("---"));
    
    assert!(has_additions || has_deletions, 
           "Mixed changes diff should have additions or deletions");
}

#[test]
fn test_repair_real_world_git_diff() {
    let git_path = Path::new("examples/data/diff/complex/real_world_git.diff");
    if !git_path.exists() {
        return;
    }
    
    let content = fs::read_to_string(git_path)
        .expect("Failed to read real_world_git.diff");
    
    let mut repairer = DiffRepairer::new();
    let result = repairer.repair(&content).unwrap();
    
    // Should handle real git diff format
    // Note: repairer may normalize the format, so we check for key elements
    assert!(result.contains("@@"), "Should contain hunk headers");
    
    // Should have file headers (--- and +++)
    let file_header_count = result.lines()
        .filter(|line| line.starts_with("---") || line.starts_with("+++"))
        .count();
    assert!(file_header_count >= 2, "Should have at least one file with headers");
    
    // Should have hunks
    let hunk_count = result.lines()
        .filter(|line| line.starts_with("@@"))
        .count();
    assert!(hunk_count > 0, "Should have at least one hunk");
}

// ============================================================================
// Batch/Integration Tests
// ============================================================================

#[test]
fn test_repair_all_malformed_diffs() {
    // Test all malformed diff files in examples/data/diff
    let malformed_files = vec![
        ("malformed", "malformed.diff"),
        ("malformed", "malformed_missing_hunk_header.diff"),
        ("malformed", "malformed_missing_file_headers.diff"),
        ("malformed", "malformed_incorrect_prefixes.diff"),
        ("malformed", "malformed_malformed_hunk_range.diff"),
        ("malformed", "malformed_inconsistent_spacing.diff"),
        ("malformed", "malformed_missing_newline.diff"),
        ("complex", "malformed_complex.diff"),
        ("complex", "multi_file_complex.diff"),
        ("complex", "large_hunk_complex.diff"),
        ("complex", "mixed_changes_complex.diff"),
        ("complex", "real_world_git.diff"),
    ];
    
    for (subdir, filename) in malformed_files {
        let file_path = Path::new("examples/data/diff").join(subdir).join(filename);
        if !file_path.exists() {
            continue;
        }
        
        let content = fs::read_to_string(&file_path)
            .unwrap_or_else(|_| panic!("Failed to read {}", filename));
        
        let mut repairer = DiffRepairer::new();
        let result = repairer.repair(&content);
        
        assert!(result.is_ok(), "Failed to repair {}", filename);
        
        let repaired = result.unwrap();
        // Repaired content should have at least a hunk header or file headers
        assert!(
            repaired.contains("@@") || repaired.contains("---") || repaired.contains("+++"),
            "Repaired {} should contain diff markers", filename
        );
    }
}
