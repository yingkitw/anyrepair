# Code Modulization Plan

## Current State Analysis

### Large Files (>800 lines)
1. **json.rs** (2081 lines)
   - RegexCache, JsonRepairer, JsonValidator
   - 10+ repair strategies
   - Needs: Extract strategies into separate module

2. **markdown.rs** (937 lines)
   - MarkdownRepairer, MarkdownValidator
   - 6+ repair strategies
   - Needs: Extract strategies into separate module

3. **advanced.rs** (883 lines)
   - Advanced repair features
   - Multiple strategy implementations
   - Needs: Split into focused modules

4. **main.rs** (779 lines)
   - CLI interface
   - Multiple subcommands
   - Needs: Extract command handlers into separate modules

### Medium Files (500-800 lines)
- yaml.rs (574)
- ini.rs (517)
- csv.rs (511)
- toml.rs (481)
- xml.rs (440)
- plugin.rs (431)
- context_parser.rs (389)
- confidence_scorer.rs (373)
- enhanced_json.rs (354)
- config.rs (352)
- plugin_config.rs (342)
- custom_rules.rs (340)

## Proposed Modulization

### 1. JSON Module Restructuring

**Current**: `src/json.rs` (2081 lines)

**Proposed Structure**:
```
src/json/
├── mod.rs              (main repairer, exports)
├── strategies.rs       (all repair strategies)
├── validator.rs        (JSON validator)
└── regex_cache.rs      (regex patterns and caching)
```

**Benefits**:
- Strategies isolated for easier maintenance
- Regex patterns centralized
- Validator separated for clarity
- Each file ~400-500 lines

### 2. Markdown Module Restructuring

**Current**: `src/markdown.rs` (937 lines)

**Proposed Structure**:
```
src/markdown/
├── mod.rs              (main repairer, exports)
├── strategies.rs       (all repair strategies)
└── validator.rs        (markdown validator)
```

**Benefits**:
- Strategies organized
- Validator separated
- Each file ~300-400 lines

### 3. Advanced Features Restructuring

**Current**: `src/advanced.rs` (883 lines)

**Proposed Structure**:
```
src/advanced/
├── mod.rs              (exports)
├── adaptive_repair.rs  (adaptive strategies)
├── multi_pass.rs       (multi-pass processing)
└── context_aware.rs    (context-aware features)
```

**Benefits**:
- Features logically grouped
- Easier to extend
- Each file ~250-300 lines

### 4. CLI Restructuring

**Current**: `src/main.rs` (779 lines)

**Proposed Structure**:
```
src/cli/
├── mod.rs              (CLI setup, main entry)
├── repair_cmd.rs       (repair command)
├── batch_cmd.rs        (batch command)
├── rules_cmd.rs        (rules management)
├── plugins_cmd.rs      (plugins management)
├── stream_cmd.rs       (streaming command)
└── validate_cmd.rs     (validation command)
```

**Benefits**:
- Commands isolated
- Easier to test
- Easier to extend
- Each file ~100-150 lines

### 5. Plugin System Restructuring

**Current**: `src/plugin.rs` (431 lines)

**Proposed Structure**:
```
src/plugin/
├── mod.rs              (exports)
├── registry.rs         (plugin registry)
├── manager.rs          (plugin manager)
└── traits.rs           (plugin traits)
```

**Benefits**:
- Concerns separated
- Registry isolated
- Manager focused
- Each file ~100-150 lines

## Implementation Priority

### Phase 1 (High Priority)
1. JSON module restructuring (largest file)
2. Markdown module restructuring
3. CLI restructuring (main.rs)

### Phase 2 (Medium Priority)
1. Advanced features restructuring
2. Plugin system restructuring
3. Config system review

### Phase 3 (Low Priority)
1. Format-specific modules (YAML, XML, etc.)
2. Utility modules
3. Documentation updates

## Modulization Guidelines

### File Size Targets
- **Optimal**: 200-400 lines
- **Maximum**: 500 lines
- **Minimum**: 50 lines (unless single concept)

### Module Organization
- One primary struct/trait per file
- Related implementations together
- Tests at module level
- Clear public API

### Naming Conventions
- `mod.rs` - Module root and exports
- `{feature}.rs` - Feature implementation
- `{type}.rs` - Type-specific code
- `strategies.rs` - Strategy implementations

### Import Organization
- Standard library imports first
- External crate imports second
- Internal crate imports third
- Blank line between groups

## Benefits of Modulization

### Code Quality
- ✅ Easier to understand
- ✅ Easier to maintain
- ✅ Easier to test
- ✅ Easier to extend

### Developer Experience
- ✅ Faster navigation
- ✅ Clearer dependencies
- ✅ Better IDE support
- ✅ Easier code review

### Performance
- ✅ Faster compilation (parallel)
- ✅ Better incremental builds
- ✅ Clearer optimization targets

## Migration Strategy

### Step 1: Create New Module Structure
- Create directories
- Create mod.rs files
- Set up exports

### Step 2: Move Code
- Move implementations
- Update imports
- Verify compilation

### Step 3: Update Tests
- Update test imports
- Verify all tests pass
- Add new tests if needed

### Step 4: Documentation
- Update README
- Update ARCHITECTURE
- Add module-level docs

## Backward Compatibility

### Public API
- All public exports maintained
- No breaking changes
- Re-exports in old locations

### Internal API
- Can be reorganized
- Tests updated accordingly
- No external impact

## Timeline Estimate

- Phase 1: 2-3 hours
- Phase 2: 2-3 hours
- Phase 3: 1-2 hours
- Testing & verification: 1-2 hours
- **Total**: 6-10 hours

## Success Criteria

- ✅ All files < 500 lines
- ✅ All tests passing
- ✅ No breaking changes
- ✅ Improved code clarity
- ✅ Faster compilation
- ✅ Better IDE support
