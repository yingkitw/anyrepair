//! Example: Minimal JSON repair with AnyRepair
//!
//! Run with: cargo run --example repair_json
//!
//! Demonstrates the core library API: `anyrepair::repair` and
//! `anyrepair::repair_with_explanations` for fixing malformed JSON.

fn main() {
    let malformed = r#"{"name": "Alice", "age": 30,}"#;

    // Simple repair — auto-detects format when none is given.
    let repaired = anyrepair::repair(malformed).expect("repair failed");
    println!("Repaired JSON:\n{repaired}\n");

    // Repair with a specific format and see which strategies were applied.
    let (repaired, strategies) =
        anyrepair::repair_with_explanations(malformed, "json").expect("repair failed");
    println!("With explanations:");
    for name in &strategies {
        println!("  - {name}");
    }
    println!("\nResult:\n{repaired}");
}
