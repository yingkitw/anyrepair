//! Markdown validation functionality

use crate::traits::Validator;

/// Markdown validator
pub struct MarkdownValidator;

impl Validator for MarkdownValidator {
    fn is_valid(&self, content: &str) -> bool {
        // Basic markdown validation
        if content.is_empty() {
            return true;
        }
        
        // Check for balanced markers
        let bold_count = content.matches("**").count();
        let italic_count = content.matches('*').count();
        let code_fence_count = content.matches("```").count();
        
        // Bold should be balanced
        if bold_count % 2 != 0 {
            return false;
        }
        
        // Code fences should be balanced
        if code_fence_count % 2 != 0 {
            return false;
        }
        
        // Check for malformed headers (# without space)
        for line in content.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                // Count leading #
                let hash_count = trimmed.chars().take_while(|c| *c == '#').count();
                if hash_count <= 6 {
                    // Check if there's a space after the hashes
                    if let Some(ch) = trimmed.chars().nth(hash_count) {
                        if ch != ' ' && ch != '\n' {
                            return false; // Malformed header
                        }
                    }
                }
            }
        }
        
        // Basic structure check
        let has_valid_structure = !content.contains("[[") && !content.contains("]]");
        
        has_valid_structure
    }
    
    fn validate(&self, content: &str) -> Vec<String> {
        let mut errors = Vec::new();
        
        if content.is_empty() {
            return errors;
        }
        
        // Check for unbalanced bold markers
        let bold_count = content.matches("**").count();
        if bold_count % 2 != 0 {
            errors.push("Unbalanced bold markers (**)".to_string());
        }
        
        // Check for unbalanced code fences
        let code_fence_count = content.matches("```").count();
        if code_fence_count % 2 != 0 {
            errors.push("Unbalanced code block fences (```)".to_string());
        }
        
        // Check for malformed links
        if content.contains("[[") || content.contains("]]") {
            errors.push("Malformed link syntax".to_string());
        }
        
        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_markdown() {
        let validator = MarkdownValidator;
        assert!(validator.is_valid("# Header\n\nSome content"));
    }

    #[test]
    fn test_invalid_markdown_unbalanced_bold() {
        let validator = MarkdownValidator;
        assert!(!validator.is_valid("**bold text"));
    }

    #[test]
    fn test_invalid_markdown_unbalanced_code() {
        let validator = MarkdownValidator;
        assert!(!validator.is_valid("```\ncode"));
    }

    #[test]
    fn test_validate_errors() {
        let validator = MarkdownValidator;
        let errors = validator.validate("**bold text");
        assert!(!errors.is_empty());
    }
}
