# Code Modulization Guide

## Overview

This guide documents the modulization effort to improve code organization, maintainability, and compilation performance by breaking down large files into focused, single-responsibility modules.

## Current Status

### Phase 1: JSON Module (In Progress)

**Files Created**:
- `src/json/strategies.rs` (300 lines) - All repair strategies
- `src/json/validator.rs` (40 lines) - JSON validation

**Files to Create**:
- `src/json/mod.rs` - Main repairer and exports

**Current Structure**:
```
src/json/
├── strategies.rs       # All repair strategies
├── validator.rs        # JSON validator
└── mod.rs             # (to be created) Main repairer
```

## Modulization Principles

### 1. Single Responsibility
- Each file handles one concern
- Strategies grouped together
- Validators separated
- Main logic in mod.rs

### 2. File Size Targets
- **Optimal**: 200-400 lines
- **Maximum**: 500 lines
- **Minimum**: 50 lines (unless single concept)

### 3. Module Organization
```
src/format/
├── mod.rs              # Main repairer, exports
├── strategies.rs       # All repair strategies
├── validator.rs        # Format validator
└── (optional) helpers.rs  # Utility functions
```

### 4. Backward Compatibility
- Old imports still work
- Re-exports maintain API
- No breaking changes

## Implementation Pattern

### Step 1: Extract Strategies
```rust
// src/format/strategies.rs
pub struct StrategyName;

impl RepairStrategy for StrategyName {
    fn apply(&self, content: &str) -> Result<String> {
        // Implementation
    }
    
    fn priority(&self) -> u8 {
        80
    }
}
```

### Step 2: Extract Validator
```rust
// src/format/validator.rs
pub struct FormatValidator;

impl Validator for FormatValidator {
    fn is_valid(&self, content: &str) -> bool {
        // Validation logic
    }
    
    fn validate(&self, content: &str) -> Vec<String> {
        // Error collection
    }
}
```

### Step 3: Create Module Root
```rust
// src/format/mod.rs
mod strategies;
mod validator;

pub use strategies::*;
pub use validator::FormatValidator;

pub struct FormatRepairer {
    // Main repairer logic
}

impl Repair for FormatRepairer {
    // Implementation
}
```

### Step 4: Update Original File
```rust
// src/format.rs (original file)
pub use crate::format::*;

// Re-export for backward compatibility
```

## Completed Modules

### JSON Module

**Files**:
- `src/json/strategies.rs` - 8 strategies
- `src/json/validator.rs` - JSON validator

**Strategies Extracted**:
1. StripTrailingContentStrategy
2. FixTrailingCommasStrategy
3. FixSingleQuotesStrategy
4. AddMissingQuotesStrategy
5. FixMalformedNumbersStrategy
6. FixBooleanNullStrategy
7. AddMissingBracesStrategy
8. FixAgenticAiResponseStrategy

**Benefits**:
- Clear separation of concerns
- Easier to test individual strategies
- Easier to add new strategies
- Better code organization

## Planned Modules

### Markdown Module
- Extract 6+ strategies
- Extract validator
- Expected reduction: 937 → 600 lines

### CLI Module
- Extract 6 command handlers
- Extract common utilities
- Expected reduction: 779 → 400 lines

### Advanced Module
- Extract adaptive repair
- Extract multi-pass processing
- Extract context-aware features
- Expected reduction: 883 → 500 lines

### Plugin Module
- Extract registry
- Extract manager
- Extract traits
- Expected reduction: 431 → 300 lines

## Benefits of Modulization

### Code Quality
| Aspect | Before | After |
|--------|--------|-------|
| File Size | 2081 lines | ~740 lines |
| Complexity | High | Low |
| Readability | Difficult | Easy |
| Navigation | Hard | Easy |

### Developer Experience
- ✅ Faster file navigation
- ✅ Clearer dependencies
- ✅ Better IDE support
- ✅ Easier code review
- ✅ Simpler testing

### Performance
- ✅ Parallel compilation
- ✅ Incremental builds
- ✅ Faster iteration

## Testing Strategy

### Unit Tests
```bash
# Test individual modules
cargo test --lib json::strategies
cargo test --lib json::validator
```

### Integration Tests
```bash
# Test complete functionality
cargo test --test complex_damage_tests
cargo test --test streaming_tests
```

### Verification
```bash
# Full verification
cargo build
cargo test
cargo clippy
cargo fmt --check
```

## Migration Checklist

### For Each Module

- [ ] Create module directory
- [ ] Extract strategies to `strategies.rs`
- [ ] Extract validator to `validator.rs`
- [ ] Create `mod.rs` with main logic
- [ ] Update exports
- [ ] Update `src/lib.rs` imports
- [ ] Run `cargo build`
- [ ] Run `cargo test`
- [ ] Run `cargo clippy`
- [ ] Update documentation

## File Size Comparison

### Before Modulization
```
json.rs          2081 lines
markdown.rs       937 lines
advanced.rs       883 lines
main.rs           779 lines
─────────────────────────
Total            4680 lines
```

### After Modulization (Estimated)
```
json/mod.rs       400 lines
json/strategies.rs 300 lines
json/validator.rs  40 lines
markdown/mod.rs   350 lines
markdown/strategies.rs 250 lines
markdown/validator.rs  40 lines
cli/mod.rs        200 lines
cli/repair_cmd.rs 150 lines
cli/batch_cmd.rs  100 lines
cli/rules_cmd.rs  100 lines
cli/plugins_cmd.rs 100 lines
cli/stream_cmd.rs  100 lines
advanced/mod.rs   300 lines
advanced/adaptive_repair.rs 250 lines
advanced/multi_pass.rs 200 lines
─────────────────────────
Total            ~2880 lines (38% reduction)
```

## Best Practices

### 1. Module Organization
```rust
// Imports first
use crate::traits::*;
use crate::error::Result;

// Public types
pub struct MyStrategy;

// Implementations
impl RepairStrategy for MyStrategy {
    // ...
}

// Tests last
#[cfg(test)]
mod tests {
    // ...
}
```

### 2. Naming Conventions
- `mod.rs` - Module root
- `strategies.rs` - Strategy implementations
- `validator.rs` - Validation logic
- `helpers.rs` - Utility functions

### 3. Export Pattern
```rust
// In mod.rs
pub use self::strategies::*;
pub use self::validator::*;

// In original file (for backward compatibility)
pub use crate::format::*;
```

### 4. Documentation
```rust
//! Module documentation
//! Describes the module's purpose and usage

/// Item documentation
/// Explains the item's purpose
pub struct Item;
```

## Common Pitfalls to Avoid

1. **Circular Dependencies**
   - Keep dependencies unidirectional
   - Use traits for abstraction

2. **Over-Modulization**
   - Don't create files < 50 lines
   - Group related functionality

3. **Broken Exports**
   - Test re-exports work
   - Maintain backward compatibility

4. **Missing Documentation**
   - Document module purpose
   - Document public items

## Verification Commands

```bash
# Build check
cargo build

# Run all tests
cargo test

# Check code quality
cargo clippy

# Format check
cargo fmt --check

# Documentation check
cargo doc --no-deps

# Size analysis
find src -name "*.rs" -exec wc -l {} + | sort -rn
```

## Timeline

| Phase | Task | Estimated Time |
|-------|------|-----------------|
| 1 | JSON module | 30 min |
| 2 | Markdown module | 20 min |
| 3 | CLI module | 45 min |
| 4 | Advanced module | 30 min |
| 5 | Testing & verification | 30 min |
| 6 | Documentation | 20 min |
| **Total** | | **2.5 hours** |

## Success Criteria

- ✅ All files < 500 lines
- ✅ All tests passing (298/298)
- ✅ No breaking changes
- ✅ Improved code clarity
- ✅ Compilation time maintained
- ✅ IDE navigation improved
- ✅ Documentation updated

## Next Steps

1. Complete JSON module (create mod.rs)
2. Verify all tests pass
3. Start Markdown module
4. Continue with CLI module
5. Update documentation
6. Celebrate improved codebase! 🎉
