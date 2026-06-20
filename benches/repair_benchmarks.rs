//! Benchmarks for anyrepair — all 10 formats + format detection

use anyrepair::{
    csv, detect_format, diff, json, key_value, markdown, toml, xml, yaml,
};
use anyrepair::traits::Repair;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_json(c: &mut Criterion) {
    let mut r = json::JsonRepairer::new();
    let ok = r#"{"name":"John","age":30,"city":"New York","hobbies":["reading","coding"],"address":{"street":"123 Main St","zip":"12345"}}"#;
    let bad = r#"{name: "John", age: 30, city: "New York", hobbies: ["reading", "coding",], address: {street: "123 Main St", zip: "12345"}}"#;
    c.bench_function("json_ok", |b| b.iter(|| r.repair(black_box(ok))));
    c.bench_function("json_bad", |b| b.iter(|| r.repair(black_box(bad))));
}

fn bench_yaml(c: &mut Criterion) {
    let mut r = yaml::YamlRepairer::new();
    let ok = "name: John\nage: 30\ncity: New York\nhobbies:\n  - reading\n  - coding";
    let bad = "name John\nage 30\ncity New York\nhobbies\n  -reading\n  -coding";
    c.bench_function("yaml_ok", |b| b.iter(|| r.repair(black_box(ok))));
    c.bench_function("yaml_bad", |b| b.iter(|| r.repair(black_box(bad))));
}

fn bench_markdown(c: &mut Criterion) {
    let mut r = markdown::MarkdownRepairer::new();
    let ok = "# Header\n\nSome **bold** and *italic* text.\n\n- item1\n- item2";
    let bad = "#Header\nSome **bold and *italic* text.\n-item1\n-item2";
    c.bench_function("md_ok", |b| b.iter(|| r.repair(black_box(ok))));
    c.bench_function("md_bad", |b| b.iter(|| r.repair(black_box(bad))));
}

fn bench_xml(c: &mut Criterion) {
    let mut r = xml::XmlRepairer::new();
    let ok = "<root><item>value</item></root>";
    let bad = "<root><item>value</item>";
    c.bench_function("xml_ok", |b| b.iter(|| r.repair(black_box(ok))));
    c.bench_function("xml_bad", |b| b.iter(|| r.repair(black_box(bad))));
}

fn bench_toml(c: &mut Criterion) {
    let mut r = toml::TomlRepairer::new();
    let ok = "[user]\nname = \"John\"\nage = 30";
    let bad = "[user\nname = \"John\"\nage = 30";
    c.bench_function("toml_ok", |b| b.iter(|| r.repair(black_box(ok))));
    c.bench_function("toml_bad", |b| b.iter(|| r.repair(black_box(bad))));
}

fn bench_csv(c: &mut Criterion) {
    let mut r = csv::CsvRepairer::new();
    let ok = "name,age\nJohn,30\nJane,25";
    let bad = "name,age\nJohn,30,\nJane,25,";
    c.bench_function("csv_ok", |b| b.iter(|| r.repair(black_box(ok))));
    c.bench_function("csv_bad", |b| b.iter(|| r.repair(black_box(bad))));
}

fn bench_ini(c: &mut Criterion) {
    let mut r = key_value::IniRepairer::new();
    let ok = "[section]\nkey=value\nother=val";
    let bad = "[section\nkey value\nother val";
    c.bench_function("ini_ok", |b| b.iter(|| r.repair(black_box(ok))));
    c.bench_function("ini_bad", |b| b.iter(|| r.repair(black_box(bad))));
}

fn bench_diff(c: &mut Criterion) {
    let mut r = diff::DiffRepairer::new();
    let ok = "--- a/file\n+++ b/file\n@@ -1,3 +1,3 @@\n line1\n-line2\n+line2new\n line3";
    let bad = "--- a/file\n+++ b/file\n@@ -1,3 +1,3 @@\nline1\n-line2\n+line2new\nline3";
    c.bench_function("diff_ok", |b| b.iter(|| r.repair(black_box(ok))));
    c.bench_function("diff_bad", |b| b.iter(|| r.repair(black_box(bad))));
}

fn bench_properties(c: &mut Criterion) {
    let mut r = key_value::PropertiesRepairer::new();
    let ok = "app.name=myapp\napp.version=1.0";
    let bad = "app.name myapp\napp.version 1.0";
    c.bench_function("props_ok", |b| b.iter(|| r.repair(black_box(ok))));
    c.bench_function("props_bad", |b| b.iter(|| r.repair(black_box(bad))));
}

fn bench_env(c: &mut Criterion) {
    let mut r = key_value::EnvRepairer::new();
    let ok = "DATABASE_URL=host\nAPI_KEY=secret";
    let bad = "DATABASE_URL host\nAPI_KEY secret";
    c.bench_function("env_ok", |b| b.iter(|| r.repair(black_box(ok))));
    c.bench_function("env_bad", |b| b.iter(|| r.repair(black_box(bad))));
}

fn bench_format_detection(c: &mut Criterion) {
    let inputs = [
        ("json", r#"{"key":"value"}"#),
        ("yaml", "key: value"),
        ("md", "# Header"),
        ("xml", "<root></root>"),
        ("toml", "[sec]\nkey='val'"),
        ("csv", "a,b\nc,d"),
        ("ini", "[sec]\nkey=val"),
        ("diff", "--- a\n+++ b"),
        ("env", "KEY=val"),
        ("properties", "key=val"),
    ];
    for (name, content) in &inputs {
        c.bench_function(&format!("detect_{}", name), |b| {
            b.iter(|| detect_format(black_box(content)))
        });
    }
}

fn bench_large_json(c: &mut Criterion) {
    let mut r = json::JsonRepairer::new();
    let mut large = String::from(r#"{"users":["#);
    for i in 0..1000 {
        if i > 0 { large.push(','); }
        large.push_str(&format!(
            r#"{{"id":{},"name":"User {}","email":"user{}@example.com","active":{}}}"#,
            i, i, i, i % 2 == 0
        ));
    }
    large.push_str("]}");
    c.bench_function("json_1000_items", |b| b.iter(|| r.repair(black_box(&large))));
}

criterion_group!(
    benches,
    bench_json,
    bench_yaml,
    bench_markdown,
    bench_xml,
    bench_toml,
    bench_csv,
    bench_ini,
    bench_diff,
    bench_properties,
    bench_env,
    bench_format_detection,
    bench_large_json
);
criterion_main!(benches);
