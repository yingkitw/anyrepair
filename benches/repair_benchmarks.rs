//! Benchmarks for anyrepair

use anyrepair::{repair, json, yaml, markdown, traits::Repair};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_json_repair(c: &mut Criterion) {
    let repairer = json::JsonRepairer::new();
    let input = r#"{"name": "John", "age": 30, "city": "New York", "hobbies": ["reading", "coding"], "address": {"street": "123 Main St", "zip": "12345"}}"#;
    
    c.bench_function("json_repair_simple", |b| {
        b.iter(|| repairer.repair(black_box(input)))
    });
    
    let malformed_input = r#"{name: "John", age: 30, city: "New York", hobbies: ["reading", "coding",], address: {street: "123 Main St", zip: "12345"}}"#;
    
    c.bench_function("json_repair_malformed", |b| {
        b.iter(|| repairer.repair(black_box(malformed_input)))
    });
}

fn benchmark_yaml_repair(c: &mut Criterion) {
    let repairer = yaml::YamlRepairer::new();
    let input = "name: John\nage: 30\ncity: New York\nhobbies:\n  - reading\n  - coding\naddress:\n  street: 123 Main St\n  zip: 12345";
    
    c.bench_function("yaml_repair_simple", |b| {
        b.iter(|| repairer.repair(black_box(input)))
    });
    
    let malformed_input = "name John\nage 30\ncity New York\nhobbies\n  -reading\n  -coding\naddress\n  street 123 Main St\n  zip 12345";
    
    c.bench_function("yaml_repair_malformed", |b| {
        b.iter(|| repairer.repair(black_box(malformed_input)))
    });
}

fn benchmark_markdown_repair(c: &mut Criterion) {
    let repairer = markdown::MarkdownRepairer::new();
    let input = "# Header\n\nSome **bold** and *italic* text.\n\n## Subsection\n\n- item1\n- item2\n\n```code\nblock\n```";
    
    c.bench_function("markdown_repair_simple", |b| {
        b.iter(|| repairer.repair(black_box(input)))
    });
    
    let malformed_input = "#Header\nSome **bold and *italic* text.\n##Subsection\n-item1\n-item2\n```code\nblock";
    
    c.bench_function("markdown_repair_malformed", |b| {
        b.iter(|| repairer.repair(black_box(malformed_input)))
    });
}

fn benchmark_auto_detection(c: &mut Criterion) {
    let json_input = r#"{"name": "John", "age": 30,}"#;
    let yaml_input = "name: John\nage: 30";
    let markdown_input = "#Header\nSome **bold** text";
    
    c.bench_function("auto_detect_json", |b| {
        b.iter(|| repair(black_box(json_input)))
    });
    
    c.bench_function("auto_detect_yaml", |b| {
        b.iter(|| repair(black_box(yaml_input)))
    });
    
    c.bench_function("auto_detect_markdown", |b| {
        b.iter(|| repair(black_box(markdown_input)))
    });
}

fn benchmark_confidence_scoring(c: &mut Criterion) {
    let json_repairer = json::JsonRepairer::new();
    let yaml_repairer = yaml::YamlRepairer::new();
    let markdown_repairer = markdown::MarkdownRepairer::new();
    
    let json_input = r#"{"name": "John", "age": 30}"#;
    let yaml_input = "name: John\nage: 30";
    let markdown_input = "# Header\nSome **bold** text";
    
    c.bench_function("json_confidence", |b| {
        b.iter(|| json_repairer.confidence(black_box(json_input)))
    });
    
    c.bench_function("yaml_confidence", |b| {
        b.iter(|| yaml_repairer.confidence(black_box(yaml_input)))
    });
    
    c.bench_function("markdown_confidence", |b| {
        b.iter(|| markdown_repairer.confidence(black_box(markdown_input)))
    });
}

fn benchmark_large_documents(c: &mut Criterion) {
    let repairer = json::JsonRepairer::new();
    
    // Create a large JSON document
    let mut large_json = String::from(r#"{"users": ["#);
    for i in 0..1000 {
        if i > 0 {
            large_json.push(',');
        }
        large_json.push_str(&format!(r#"{{"id": {}, "name": "User {}", "email": "user{}@example.com", "active": {}}}"#, 
            i, i, i, i % 2 == 0));
    }
    large_json.push_str("]}");
    
    c.bench_function("json_large_document", |b| {
        b.iter(|| repairer.repair(black_box(&large_json)))
    });
}

fn benchmark_strategy_application(c: &mut Criterion) {
    let repairer = json::JsonRepairer::new();
    
    // Test with input that needs multiple strategies
    let complex_input = r#"{name: "John", age: 30, city: "New York", hobbies: ["reading", "coding",], address: {street: "123 Main St", zip: "12345",},}"#;
    
    c.bench_function("json_multiple_strategies", |b| {
        b.iter(|| repairer.repair(black_box(complex_input)))
    });
}

criterion_group!(
    benches,
    benchmark_json_repair,
    benchmark_yaml_repair,
    benchmark_markdown_repair,
    benchmark_auto_detection,
    benchmark_confidence_scoring,
    benchmark_large_documents,
    benchmark_strategy_application
);
criterion_main!(benches);
