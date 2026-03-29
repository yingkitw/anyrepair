//! Key-value format repair module (INI, .env, .properties)

use crate::error::Result;
use crate::traits::{Repair, RepairStrategy, Validator};
use std::collections::HashSet;

struct FixMissingEqualsStrategy;

impl RepairStrategy for FixMissingEqualsStrategy {
    fn name(&self) -> &str {
        "FixMissingEquals"
    }

    fn apply(&self, content: &str) -> Result<String> {
        let mut result = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if is_skip_line(trimmed) {
                result.push(line.to_string());
                continue;
            }
            if !trimmed.contains('=') {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 2 {
                    result.push(format!("{}={}", parts[0], parts[1..].join(" ")));
                } else if parts.len() == 1 {
                    result.push(format!("{}=", parts[0]));
                } else {
                    result.push(line.to_string());
                }
            } else {
                result.push(line.to_string());
            }
        }
        Ok(result.join("\n"))
    }

    fn priority(&self) -> u8 {
        100
    }
}

struct FixWhitespaceAroundEqualsStrategy;

impl RepairStrategy for FixWhitespaceAroundEqualsStrategy {
    fn name(&self) -> &str {
        "FixWhitespaceAroundEquals"
    }

    fn apply(&self, content: &str) -> Result<String> {
        let mut result = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if is_skip_line(trimmed) {
                result.push(line.to_string());
                continue;
            }
            if let Some(eq_pos) = trimmed.find('=') {
                let key = trimmed[..eq_pos].trim();
                let value = trimmed[eq_pos + 1..].trim();
                result.push(format!("{}={}", key, value));
            } else {
                result.push(line.to_string());
            }
        }
        Ok(result.join("\n"))
    }

    fn priority(&self) -> u8 {
        90
    }
}

struct FixEmptyKeysStrategy {
    prefix: &'static str,
}

impl RepairStrategy for FixEmptyKeysStrategy {
    fn name(&self) -> &str {
        "FixEmptyKeys"
    }

    fn apply(&self, content: &str) -> Result<String> {
        let mut result = Vec::new();
        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if is_skip_line(trimmed) {
                result.push(line.to_string());
                continue;
            }
            if let Some(stripped) = trimmed.strip_prefix('=') {
                let value = stripped.trim();
                result.push(format!("{}_{}={}", self.prefix, i, value));
            } else {
                result.push(line.to_string());
            }
        }
        Ok(result.join("\n"))
    }

    fn priority(&self) -> u8 {
        80
    }
}

struct FixMalformedCommentsStrategy;

impl RepairStrategy for FixMalformedCommentsStrategy {
    fn name(&self) -> &str {
        "FixMalformedComments"
    }

    fn apply(&self, content: &str) -> Result<String> {
        let mut result = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.contains('#') && !trimmed.starts_with('#') && !trimmed.contains('=') {
                if let Some(hash_pos) = trimmed.find('#') {
                    let before = trimmed[..hash_pos].trim();
                    let after = &trimmed[hash_pos..];
                    if before.is_empty() {
                        result.push(after.to_string());
                    } else {
                        result.push(format!("#{} {}", before, after[1..].trim()));
                    }
                } else {
                    result.push(line.to_string());
                }
            } else {
                result.push(line.to_string());
            }
        }
        Ok(result.join("\n"))
    }

    fn priority(&self) -> u8 {
        70
    }
}

struct FixQuotedValuesStrategy;

impl RepairStrategy for FixQuotedValuesStrategy {
    fn name(&self) -> &str {
        "FixQuotedValues"
    }

    fn apply(&self, content: &str) -> Result<String> {
        let mut result = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if is_skip_line(trimmed) {
                result.push(line.to_string());
                continue;
            }
            if let Some(eq_pos) = trimmed.find('=') {
                let value = trimmed[eq_pos + 1..].trim();
                if value.starts_with('"') && !value.ends_with('"') {
                    result.push(format!("{}=\"{}\"", &trimmed[..=eq_pos], &value[1..]));
                } else if value.starts_with('\'') && !value.ends_with('\'') {
                    result.push(format!("{}='{}'", &trimmed[..=eq_pos], &value[1..]));
                } else if value.ends_with('"') && !value.starts_with('"') {
                    result.push(format!(
                        "{}\"{}\"",
                        &trimmed[..=eq_pos],
                        &value[..value.len() - 1]
                    ));
                } else if value.ends_with('\'') && !value.starts_with('\'') {
                    result.push(format!(
                        "{}'{}'",
                        &trimmed[..=eq_pos],
                        &value[..value.len() - 1]
                    ));
                } else {
                    result.push(line.to_string());
                }
            } else {
                result.push(line.to_string());
            }
        }
        Ok(result.join("\n"))
    }

    fn priority(&self) -> u8 {
        60
    }
}

// --- INI-specific strategies ---

struct FixMalformedSectionsStrategy;

impl RepairStrategy for FixMalformedSectionsStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let mut result = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('[') && !trimmed.ends_with(']') {
                let indent = line
                    .chars()
                    .take_while(|c| c.is_whitespace())
                    .collect::<String>();
                let section_name = trimmed.trim_start_matches('[');
                result.push(format!("{}[{}]", indent, section_name));
            } else {
                result.push(line.to_string());
            }
        }
        Ok(result.join("\n"))
    }

    fn priority(&self) -> u8 {
        6
    }

    fn name(&self) -> &str {
        "FixMalformedSectionsStrategy"
    }
}

struct FixMalformedKeysStrategy;

impl RepairStrategy for FixMalformedKeysStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let mut result = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if is_skip_line(trimmed) || trimmed.contains('=') {
                result.push(line.to_string());
                continue;
            }
            let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
            if parts.len() == 2 {
                let indent = line
                    .chars()
                    .take_while(|c| c.is_whitespace())
                    .collect::<String>();
                result.push(format!("{}{} = {}", indent, parts[0], parts[1]));
            } else {
                result.push(line.to_string());
            }
        }
        Ok(result.join("\n"))
    }

    fn priority(&self) -> u8 {
        5
    }

    fn name(&self) -> &str {
        "FixMalformedKeysStrategy"
    }
}

struct RemoveDuplicateSectionsStrategy;

impl RepairStrategy for RemoveDuplicateSectionsStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let mut seen = HashSet::new();
        let mut result = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                let name = &trimmed[1..trimmed.len() - 1];
                if seen.contains(name) {
                    continue;
                }
                seen.insert(name.to_string());
            }
            result.push(line);
        }
        Ok(result.join("\n"))
    }

    fn priority(&self) -> u8 {
        1
    }

    fn name(&self) -> &str {
        "RemoveDuplicateSectionsStrategy"
    }
}

struct AddDefaultSectionStrategy;

impl RepairStrategy for AddDefaultSectionStrategy {
    fn apply(&self, content: &str) -> Result<String> {
        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() {
            return Ok(content.to_string());
        }
        let first = lines[0].trim();
        if first.contains('=') && !first.starts_with('[') {
            let mut result = vec!["[default]".to_string()];
            result.extend(lines.iter().map(|s| s.to_string()));
            Ok(result.join("\n"))
        } else {
            Ok(content.to_string())
        }
    }

    fn priority(&self) -> u8 {
        0
    }

    fn name(&self) -> &str {
        "AddDefaultSectionStrategy"
    }
}

// --- Helpers ---

fn is_skip_line(trimmed: &str) -> bool {
    trimmed.is_empty()
        || trimmed.starts_with('#')
        || trimmed.starts_with('!')
        || trimmed.starts_with('[')
}

// --- Public types ---

pub struct IniRepairer {
    inner: crate::repairer_base::GenericRepairer,
}

impl IniRepairer {
    pub fn new() -> Self {
        let strategies: Vec<Box<dyn RepairStrategy>> = vec![
            Box::new(FixMalformedSectionsStrategy),
            Box::new(FixMalformedKeysStrategy),
            Box::new(FixMissingEqualsStrategy),
            Box::new(FixWhitespaceAroundEqualsStrategy),
            Box::new(FixMalformedCommentsStrategy),
            Box::new(RemoveDuplicateSectionsStrategy),
            Box::new(AddDefaultSectionStrategy),
        ];
        let validator: Box<dyn Validator> = Box::new(IniValidator);
        Self {
            inner: crate::repairer_base::GenericRepairer::new(validator, strategies),
        }
    }
}

impl Default for IniRepairer {
    fn default() -> Self {
        Self::new()
    }
}

impl Repair for IniRepairer {
    fn repair(&mut self, content: &str) -> Result<String> {
        self.inner.repair(content)
    }

    fn needs_repair(&self, content: &str) -> bool {
        self.inner.needs_repair(content)
    }

    fn confidence(&self, content: &str) -> f64 {
        if content.trim().is_empty() {
            return 0.0;
        }
        let mut score: f64 = 0.0;
        if content.contains('[') && content.contains(']') {
            score += 0.3;
        }
        if content.contains('=') {
            score += 0.3;
        }
        if content.contains('#') {
            score += 0.1;
        }
        let lines: Vec<&str> = content.lines().collect();
        let has_sections = lines.iter().any(|l| l.trim().starts_with('['));
        let has_keys = lines
            .iter()
            .any(|l| l.contains('=') && !l.trim().starts_with('#'));
        if has_sections || has_keys {
            score += 0.2;
        }
        if content.contains('\n') {
            score += 0.1;
        }
        score.min(1.0)
    }
}

pub struct IniValidator;

impl Validator for IniValidator {
    fn is_valid(&self, content: &str) -> bool {
        if content.trim().is_empty() {
            return false;
        }
        let lines: Vec<&str> = content.lines().collect();
        for line in &lines {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
                continue;
            }
            if line.starts_with('[') && !line.ends_with(']') {
                return false;
            }
            if line.contains(' ') && !line.contains('=') && !line.starts_with('[') {
                return false;
            }
        }
        let has_sections = lines
            .iter()
            .any(|l| l.trim().starts_with('[') && l.contains(']'));
        let has_keys = lines
            .iter()
            .any(|l| l.contains('=') && !l.trim().starts_with('#') && !l.trim().starts_with('['));
        has_sections || has_keys
    }

    fn validate(&self, content: &str) -> Vec<String> {
        let mut errors = Vec::new();
        if content.trim().is_empty() {
            errors.push("Empty INI content".to_string());
            return errors;
        }
        for (i, line) in content.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if line.starts_with('[') && !line.contains(']') {
                errors.push(format!(
                    "Malformed section header at line {}: {}",
                    i + 1,
                    line
                ));
            } else if line.contains('=') {
                let parts: Vec<&str> = line.splitn(2, '=').collect();
                if parts.len() != 2 {
                    errors.push(format!(
                        "Malformed key-value pair at line {}: {}",
                        i + 1,
                        line
                    ));
                }
            }
        }
        errors
    }
}

pub struct EnvRepairer {
    inner: crate::repairer_base::GenericRepairer,
}

impl EnvRepairer {
    pub fn new() -> Self {
        let strategies: Vec<Box<dyn RepairStrategy>> = vec![
            Box::new(FixMissingEqualsStrategy),
            Box::new(FixWhitespaceAroundEqualsStrategy),
            Box::new(FixEmptyKeysStrategy { prefix: "ENV_VAR" }),
            Box::new(FixMalformedCommentsStrategy),
            Box::new(FixQuotedValuesStrategy),
        ];
        let validator: Box<dyn Validator> = Box::new(EnvValidator);
        Self {
            inner: crate::repairer_base::GenericRepairer::new(validator, strategies),
        }
    }
}

impl Default for EnvRepairer {
    fn default() -> Self {
        Self::new()
    }
}

impl Repair for EnvRepairer {
    fn repair(&mut self, content: &str) -> Result<String> {
        self.inner.repair(content)
    }

    fn needs_repair(&self, content: &str) -> bool {
        self.inner.needs_repair(content)
    }

    fn confidence(&self, content: &str) -> f64 {
        if content.trim().is_empty() {
            return 0.0;
        }
        let mut score: f64 = 0.0;
        if content.contains('=') {
            score += 0.4;
        }
        if content.contains('#') {
            score += 0.2;
        }
        let uppercase_count = content.matches(char::is_uppercase).count();
        let total_chars = content.len();
        if total_chars > 0 && uppercase_count as f64 / total_chars as f64 > 0.2 {
            score += 0.2;
        }
        if content.contains('_') {
            score += 0.1;
        }
        if content.contains('"') || content.contains('\'') {
            score += 0.1;
        }
        score.clamp(0.0, 1.0)
    }
}

pub struct EnvValidator;

impl Validator for EnvValidator {
    fn is_valid(&self, content: &str) -> bool {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if !trimmed.contains('=') {
                return false;
            }
            if let Some(eq_pos) = trimmed.find('=') {
                let key = trimmed[..eq_pos].trim();
                if key.is_empty() {
                    return false;
                }
            }
        }
        true
    }

    fn validate(&self, content: &str) -> Vec<String> {
        let mut errors = Vec::new();
        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if !trimmed.contains('=') {
                errors.push(format!("Line {}: Missing '=' delimiter", line_num + 1));
                continue;
            }
            if trimmed.starts_with('=') {
                errors.push(format!("Line {}: Empty key", line_num + 1));
            }
        }
        errors
    }
}

pub struct PropertiesRepairer {
    inner: crate::repairer_base::GenericRepairer,
}

impl PropertiesRepairer {
    pub fn new() -> Self {
        let strategies: Vec<Box<dyn RepairStrategy>> = vec![
            Box::new(FixMissingEqualsStrategy),
            Box::new(FixWhitespaceAroundEqualsStrategy),
            Box::new(FixEmptyKeysStrategy { prefix: "key" }),
            Box::new(FixMalformedCommentsStrategy),
            Box::new(FixQuotedValuesStrategy),
        ];
        let validator: Box<dyn Validator> = Box::new(PropertiesValidator);
        Self {
            inner: crate::repairer_base::GenericRepairer::new(validator, strategies),
        }
    }
}

impl Default for PropertiesRepairer {
    fn default() -> Self {
        Self::new()
    }
}

impl Repair for PropertiesRepairer {
    fn repair(&mut self, content: &str) -> Result<String> {
        self.inner.repair(content)
    }

    fn needs_repair(&self, content: &str) -> bool {
        self.inner.needs_repair(content)
    }

    fn confidence(&self, content: &str) -> f64 {
        if content.trim().is_empty() {
            return 0.0;
        }
        let mut score: f64 = 0.0;
        if content.contains('=') {
            score += 0.5;
        }
        if content.contains('#') || content.contains('!') {
            score += 0.2;
        }
        if content.matches('.').count() > 0 {
            score += 0.2;
        }
        if content.contains("\\\n") {
            score += 0.1;
        }
        score.clamp(0.0, 1.0)
    }
}

pub struct PropertiesValidator;

impl Validator for PropertiesValidator {
    fn is_valid(&self, content: &str) -> bool {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('!') {
                continue;
            }
            if !trimmed.contains('=') {
                return false;
            }
            if let Some(eq_pos) = trimmed.find('=') {
                if trimmed[..eq_pos].trim().is_empty() {
                    return false;
                }
            }
        }
        true
    }

    fn validate(&self, content: &str) -> Vec<String> {
        let mut errors = Vec::new();
        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('!') {
                continue;
            }
            if !trimmed.contains('=') {
                errors.push(format!("Line {}: Missing '=' delimiter", line_num + 1));
                continue;
            }
            if trimmed.starts_with('=') {
                errors.push(format!("Line {}: Empty key", line_num + 1));
            }
        }
        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ini_repair() {
        let mut r = IniRepairer::new();
        let result = r.repair("[section]\nkey value\nother = ok").unwrap();
        assert!(result.contains("[section]"));
        assert!(result.contains("key = value") || result.contains("key=value"));
    }

    #[test]
    fn test_env_repair() {
        let mut r = EnvRepairer::new();
        let result = r
            .repair("DATABASE_URL postgresql://localhost/mydb\nAPI_KEY secret_key")
            .unwrap();
        assert!(result.contains("DATABASE_URL="));
        assert!(result.contains("API_KEY="));
    }

    #[test]
    fn test_properties_repair() {
        let mut r = PropertiesRepairer::new();
        let result = r.repair("key1 value1\nkey2 value2").unwrap();
        assert!(result.contains("key1=value1"));
        assert!(result.contains("key2=value2"));
    }

    #[test]
    fn test_ini_validator() {
        let v = IniValidator;
        assert!(v.is_valid("[section]\nkey=value"));
        assert!(!v.is_valid(""));
    }

    #[test]
    fn test_env_validator() {
        let v = EnvValidator;
        assert!(v.is_valid("KEY=value\nOTHER=val2"));
        assert!(!v.is_valid("KEY value"));
    }

    #[test]
    fn test_properties_validator() {
        let v = PropertiesValidator;
        assert!(v.is_valid("key=value\nother=val"));
        assert!(!v.is_valid("key value"));
    }

    #[test]
    fn test_ini_sections() {
        let mut r = IniRepairer::new();
        let result = r.repair("[section\nkey=value").unwrap();
        assert!(result.contains("[section]"));
    }

    #[test]
    fn test_env_confidence() {
        let r = EnvRepairer::new();
        assert!(
            r.confidence("DATABASE_URL=postgresql://localhost/mydb\nAPI_KEY=secret_key") >= 0.5
        );
        assert!(r.confidence("some random text") < 0.5);
    }
}
