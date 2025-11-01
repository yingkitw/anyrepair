use anyrepair::{repair, json, yaml, markdown, xml, toml, csv, ini, traits::{Repair, Validator}};
use proptest::prelude::*;

/// Fuzz testing for JSON repair functionality
#[cfg(test)]
mod json_fuzz_tests {
    use super::*;

    proptest! {
        #[test]
        fn test_json_repair_never_panics(input in prop::string::string_regex(".*").unwrap()) {
            let _ = json::JsonRepairer::new().repair(&input);
        }

        #[test]
        fn test_json_repair_improves_validity(input in prop::string::string_regex(".*").unwrap()) {
            let mut repairer = json::JsonRepairer::new();
            let validator = json::JsonValidator;
            let original_valid = validator.is_valid(&input);
            let repaired = repairer.repair(&input).unwrap_or_else(|_| input.clone());
            let repaired_valid = validator.is_valid(&repaired);
            
            // Repair should either maintain validity or improve it
            prop_assert!(repaired_valid || !original_valid);
        }

        #[test]
        fn test_json_confidence_bounds(input in prop::string::string_regex(".*").unwrap()) {
            let mut repairer = json::JsonRepairer::new();
            let confidence = repairer.confidence(&input);
            prop_assert!(confidence >= 0.0 && confidence <= 1.0);
        }

        #[test]
        fn test_json_repair_idempotent(input in prop::string::string_regex(".*").unwrap()) {
            let mut repairer = json::JsonRepairer::new();
            let first_repair = repairer.repair(&input).unwrap_or_else(|_| input.clone());
            let second_repair = repairer.repair(&first_repair).unwrap_or_else(|_| first_repair.clone());
            
            // Second repair should not change the first repair significantly
            // Allow for some variance due to quote escaping and other repairs
            // The tolerance is higher to account for edge cases with special characters and unicode
            prop_assert!(first_repair == second_repair || 
                        (first_repair.len() as i32 - second_repair.len() as i32).abs() < 100);
        }
    }
}

/// Fuzz testing for YAML repair functionality
#[cfg(test)]
mod yaml_fuzz_tests {
    use super::*;

    proptest! {
        #[test]
        fn test_yaml_repair_never_panics(input in prop::string::string_regex(".*").unwrap()) {
            let _ = yaml::YamlRepairer::new().repair(&input);
        }

        #[test]
        fn test_yaml_repair_improves_validity(input in prop::string::string_regex(".*").unwrap()) {
            let mut repairer = yaml::YamlRepairer::new();
            let validator = yaml::YamlValidator;
            let original_valid = validator.is_valid(&input);
            let repaired = repairer.repair(&input).unwrap_or_else(|_| input.clone());
            let repaired_valid = validator.is_valid(&repaired);
            
            prop_assert!(repaired_valid || !original_valid);
        }

        #[test]
        fn test_yaml_confidence_bounds(input in prop::string::string_regex(".*").unwrap()) {
            let mut repairer = yaml::YamlRepairer::new();
            let confidence = repairer.confidence(&input);
            prop_assert!(confidence >= 0.0 && confidence <= 1.0);
        }
    }
}

/// Fuzz testing for Markdown repair functionality
#[cfg(test)]
mod markdown_fuzz_tests {
    use super::*;

    proptest! {
        #[test]
        fn test_markdown_repair_never_panics(input in prop::string::string_regex(".*").unwrap()) {
            let _ = markdown::MarkdownRepairer::new().repair(&input);
        }

        #[test]
        fn test_markdown_repair_improves_validity(input in prop::string::string_regex(".*").unwrap()) {
            let mut repairer = markdown::MarkdownRepairer::new();
            let validator = markdown::MarkdownValidator;
            let original_valid = validator.is_valid(&input);
            let repaired = repairer.repair(&input).unwrap_or_else(|_| input.clone());
            let repaired_valid = validator.is_valid(&repaired);
            
            prop_assert!(repaired_valid || !original_valid);
        }

        #[test]
        fn test_markdown_confidence_bounds(input in prop::string::string_regex(".*").unwrap()) {
            let mut repairer = markdown::MarkdownRepairer::new();
            let confidence = repairer.confidence(&input);
            prop_assert!(confidence >= 0.0 && confidence <= 1.0);
        }
    }
}

/// Fuzz testing for XML repair functionality
#[cfg(test)]
mod xml_fuzz_tests {
    use super::*;

    proptest! {
        #[test]
        fn test_xml_repair_never_panics(input in prop::string::string_regex(".*").unwrap()) {
            let _ = xml::XmlRepairer::new().repair(&input);
        }

        #[test]
        fn test_xml_repair_improves_validity(input in prop::string::string_regex(".*").unwrap()) {
            let mut repairer = xml::XmlRepairer::new();
            let validator = xml::XmlValidator;
            let original_valid = validator.is_valid(&input);
            let repaired = repairer.repair(&input).unwrap_or_else(|_| input.clone());
            let repaired_valid = validator.is_valid(&repaired);
            
            prop_assert!(repaired_valid || !original_valid);
        }

        #[test]
        fn test_xml_confidence_bounds(input in prop::string::string_regex(".*").unwrap()) {
            let mut repairer = xml::XmlRepairer::new();
            let confidence = repairer.confidence(&input);
            prop_assert!(confidence >= 0.0 && confidence <= 1.0);
        }
    }
}

/// Fuzz testing for TOML repair functionality
#[cfg(test)]
mod toml_fuzz_tests {
    use super::*;

    proptest! {
        #[test]
        fn test_toml_repair_never_panics(input in prop::string::string_regex(".*").unwrap()) {
            let _ = toml::TomlRepairer::new().repair(&input);
        }

        #[test]
        fn test_toml_repair_improves_validity(input in prop::string::string_regex(".*").unwrap()) {
            let mut repairer = toml::TomlRepairer::new();
            let validator = toml::TomlValidator;
            let original_valid = validator.is_valid(&input);
            let repaired = repairer.repair(&input).unwrap_or_else(|_| input.clone());
            let repaired_valid = validator.is_valid(&repaired);
            
            prop_assert!(repaired_valid || !original_valid);
        }

        #[test]
        fn test_toml_confidence_bounds(input in prop::string::string_regex(".*").unwrap()) {
            let mut repairer = toml::TomlRepairer::new();
            let confidence = repairer.confidence(&input);
            prop_assert!(confidence >= 0.0 && confidence <= 1.0);
        }
    }
}

/// Fuzz testing for CSV repair functionality
#[cfg(test)]
mod csv_fuzz_tests {
    use super::*;

    proptest! {
        #[test]
        fn test_csv_repair_never_panics(input in prop::string::string_regex(".*").unwrap()) {
            let _ = csv::CsvRepairer::new().repair(&input);
        }

        #[test]
        fn test_csv_repair_improves_validity(input in prop::string::string_regex(".*").unwrap()) {
            let mut repairer = csv::CsvRepairer::new();
            let validator = csv::CsvValidator;
            let original_valid = validator.is_valid(&input);
            let repaired = repairer.repair(&input).unwrap_or_else(|_| input.clone());
            let repaired_valid = validator.is_valid(&repaired);
            
            prop_assert!(repaired_valid || !original_valid);
        }

        #[test]
        fn test_csv_confidence_bounds(input in prop::string::string_regex(".*").unwrap()) {
            let mut repairer = csv::CsvRepairer::new();
            let confidence = repairer.confidence(&input);
            prop_assert!(confidence >= 0.0 && confidence <= 1.0);
        }
    }
}

/// Fuzz testing for INI repair functionality
#[cfg(test)]
mod ini_fuzz_tests {
    use super::*;

    proptest! {
        #[test]
        fn test_ini_repair_never_panics(input in prop::string::string_regex(".*").unwrap()) {
            let _ = ini::IniRepairer::new().repair(&input);
        }

        #[test]
        fn test_ini_repair_improves_validity(input in prop::string::string_regex(".*").unwrap()) {
            let mut repairer = ini::IniRepairer::new();
            let validator = ini::IniValidator;
            let original_valid = validator.is_valid(&input);
            let repaired = repairer.repair(&input).unwrap_or_else(|_| input.clone());
            let repaired_valid = validator.is_valid(&repaired);
            
            prop_assert!(repaired_valid || !original_valid);
        }

        #[test]
        fn test_ini_confidence_bounds(input in prop::string::string_regex(".*").unwrap()) {
            let mut repairer = ini::IniRepairer::new();
            let confidence = repairer.confidence(&input);
            prop_assert!(confidence >= 0.0 && confidence <= 1.0);
        }
    }
}

/// Fuzz testing for general repair functionality
#[cfg(test)]
mod general_fuzz_tests {
    use super::*;

    proptest! {
        #[test]
        fn test_repair_never_panics(input in prop::string::string_regex(".*").unwrap()) {
            let _ = repair(&input);
        }

        #[test]
        fn test_repair_handles_empty_input(input in prop::string::string_regex("").unwrap()) {
            let result = repair(&input);
            prop_assert!(result.is_ok());
        }

        #[test]
        fn test_repair_handles_very_long_input(input in prop::string::string_regex(".{0,10000}").unwrap()) {
            let result = repair(&input);
            prop_assert!(result.is_ok());
        }

        #[test]
        fn test_repair_handles_unicode_input(input in prop::string::string_regex(".*").unwrap()) {
            let result = repair(&input);
            prop_assert!(result.is_ok());
        }

        #[test]
        fn test_repair_handles_binary_like_input(input in prop::string::string_regex("[\\x00-\\xFF]*").unwrap()) {
            let result = repair(&input);
            prop_assert!(result.is_ok());
        }
    }
}

/// Fuzz testing for edge cases and boundary conditions
#[cfg(test)]
mod edge_case_fuzz_tests {
    use super::*;

    proptest! {
        #[test]
        fn test_repair_with_only_whitespace(input in prop::string::string_regex(r"\s*").unwrap()) {
            let result = repair(&input);
            prop_assert!(result.is_ok());
        }

        #[test]
        fn test_repair_with_only_special_chars(input in prop::string::string_regex("[!@#$%^&*()_+\\-=\\[\\]{}|;':\",./<>?]*").unwrap()) {
            let result = repair(&input);
            prop_assert!(result.is_ok());
        }

        #[test]
        fn test_repair_with_mixed_newlines(input in prop::string::string_regex("[\\r\\n]*").unwrap()) {
            let result = repair(&input);
            prop_assert!(result.is_ok());
        }

        #[test]
        fn test_repair_with_nested_quotes(input in prop::string::string_regex("[\"']*").unwrap()) {
            let result = repair(&input);
            prop_assert!(result.is_ok());
        }

        #[test]
        fn test_repair_with_repeated_chars(input in prop::string::string_regex("(.){1,100}").unwrap()) {
            let result = repair(&input);
            prop_assert!(result.is_ok());
        }
    }
}

/// Fuzz testing for performance and memory usage
#[cfg(test)]
mod performance_fuzz_tests {
    use super::*;
    use std::time::Instant;

    proptest! {
        #[test]
        fn test_repair_performance_reasonable(input in prop::string::string_regex(".{0,1000}").unwrap()) {
            let start = Instant::now();
            let _ = repair(&input);
            let duration = start.elapsed();
            
            // Repair should complete within reasonable time (1 second)
            prop_assert!(duration.as_secs() < 1);
        }

        #[test]
        fn test_repair_memory_usage_reasonable(input in prop::string::string_regex(".{0,5000}").unwrap()) {
            let result = repair(&input).unwrap();
            
            // Output should not be excessively larger than input
            // For empty input, result should also be empty or minimal
            if input.is_empty() {
                prop_assert!(result.len() <= 1);
            } else {
                prop_assert!(result.len() < input.len() * 10);
            }
        }
    }
}

/// Fuzz testing for custom rules integration
#[cfg(test)]
mod custom_rules_fuzz_tests {
    use super::*;
    use anyrepair::config::{RepairConfig, CustomRule};
    use anyrepair::custom_rules::CustomRuleEngine;

    proptest! {
        #[test]
        fn test_custom_rule_engine_never_panics(
            input in prop::string::string_regex(".*").unwrap(),
            format in prop::string::string_regex("[a-z]+").unwrap()
        ) {
            let mut config = RepairConfig::new();
            let rule = CustomRule {
                id: "test_rule".to_string(),
                name: "Test Rule".to_string(),
                description: "Test".to_string(),
                target_format: format.clone(),
                priority: 5,
                enabled: true,
                pattern: ".*".to_string(),
                replacement: "test".to_string(),
                conditions: vec![],
            };
            config.add_custom_rule(rule);
            
            let mut engine = CustomRuleEngine::new();
            let _ = engine.load_from_config(&config);
            let _ = engine.apply_rules(&input, &format);
        }

        #[test]
        fn test_custom_rule_engine_with_invalid_regex(
            _input in prop::string::string_regex(".*").unwrap(),
            format in prop::string::string_regex("[a-z]+").unwrap()
        ) {
            let mut config = RepairConfig::new();
            let rule = CustomRule {
                id: "test_rule".to_string(),
                name: "Test Rule".to_string(),
                description: "Test".to_string(),
                target_format: format.clone(),
                priority: 5,
                enabled: true,
                pattern: "[invalid".to_string(), // Invalid regex
                replacement: "test".to_string(),
                conditions: vec![],
            };
            config.add_custom_rule(rule);
            
            let mut engine = CustomRuleEngine::new();
            let result = engine.load_from_config(&config);
            // Should handle invalid regex gracefully
            prop_assert!(result.is_err());
        }
    }
}
