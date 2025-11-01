# Build Optimization Guide

## Overview

AnyRepair is optimized for minimal binary size while maintaining full functionality. This guide explains the build optimizations and how to use them.

## Binary Sizes

### Release Build (Optimized for Size)

```
anyrepair CLI:    1.5 MB
anyrepair-mcp:    1.4 MB
```

### Debug Build

```
anyrepair CLI:    ~25 MB
anyrepair-mcp:    ~24 MB
```

## Build Profiles

### 1. Release Profile (Default)

**Purpose**: Balanced optimization for size and performance

**Configuration**:
```toml
[profile.release]
opt-level = "z"           # Optimize for size
lto = true                # Enable Link Time Optimization
codegen-units = 1         # Single codegen unit for better optimization
strip = true              # Strip symbols from binary
panic = "abort"           # Abort on panic (smaller binary)
```

**Build Command**:
```bash
cargo build --release
```

**Binary Location**:
```
target/release/anyrepair
target/release/anyrepair-mcp
```

### 2. Distribution Profile (Maximum Optimization)

**Purpose**: Maximum size reduction for distribution

**Configuration**:
```toml
[profile.dist]
inherits = "release"
opt-level = "z"
lto = "fat"               # Fat LTO for maximum optimization
codegen-units = 1
strip = true
panic = "abort"
```

**Build Command**:
```bash
cargo build --profile dist
```

**Binary Location**:
```
target/dist/anyrepair
target/dist/anyrepair-mcp
```

### 3. Debug Profile

**Purpose**: Fast compilation for development

**Build Command**:
```bash
cargo build
```

**Binary Location**:
```
target/debug/anyrepair
target/debug/anyrepair-mcp
```

## Optimization Techniques

### 1. Size Optimization (`opt-level = "z"`)

- Minimizes code size
- Slightly slower than `opt-level = "3"`
- Ideal for distribution binaries

### 2. Link Time Optimization (LTO)

**Release Profile**: `lto = true` (thin LTO)
- Faster compilation
- Good size reduction
- ~28 seconds build time

**Distribution Profile**: `lto = "fat"` (fat LTO)
- Slower compilation
- Maximum size reduction
- ~27 seconds build time

### 3. Single Codegen Unit (`codegen-units = 1`)

- Better optimization across compilation units
- Slower compilation
- Smaller binary size

### 4. Symbol Stripping (`strip = true`)

- Removes debug symbols
- Reduces binary size by ~20%
- No impact on functionality

### 5. Panic Abort (`panic = "abort"`)

- Aborts on panic instead of unwinding
- Smaller binary
- Faster panic handling

## Build Performance

### Compilation Times

| Profile | Time | Size | Use Case |
|---------|------|------|----------|
| Debug | 5s | 25 MB | Development |
| Release | 28s | 1.5 MB | Production |
| Dist | 27s | 1.5 MB | Distribution |

### Size Comparison

| Profile | anyrepair | anyrepair-mcp | Total |
|---------|-----------|---------------|-------|
| Debug | 13 MB | 12 MB | 25 MB |
| Release | 1.5 MB | 1.4 MB | 2.9 MB |
| Dist | 1.5 MB | 1.4 MB | 2.9 MB |

**Size Reduction**: 94% smaller than debug build!

## Building for Production

### Step 1: Build Release Binary

```bash
cargo build --release
```

### Step 2: Verify Binary Size

```bash
ls -lh target/release/anyrepair
ls -lh target/release/anyrepair-mcp
```

### Step 3: Test Binary

```bash
./target/release/anyrepair --help
./target/release/anyrepair-mcp
```

### Step 4: Deploy

```bash
cp target/release/anyrepair /usr/local/bin/
cp target/release/anyrepair-mcp /usr/local/bin/
```

## Building for Distribution

### Step 1: Build Distribution Binary

```bash
cargo build --profile dist
```

### Step 2: Create Distribution Package

```bash
# macOS
tar -czf anyrepair-macos.tar.gz target/dist/anyrepair target/dist/anyrepair-mcp

# Linux
tar -czf anyrepair-linux.tar.gz target/dist/anyrepair target/dist/anyrepair-mcp

# Windows
zip anyrepair-windows.zip target/dist/anyrepair.exe target/dist/anyrepair-mcp.exe
```

### Step 3: Verify Package

```bash
tar -tzf anyrepair-macos.tar.gz
```

## Advanced Optimization

### Further Size Reduction (Optional)

If you need even smaller binaries, consider:

1. **UPX Compression** (Unix only):
```bash
upx --best --lzma target/release/anyrepair
```

2. **Strip Additional Symbols**:
```bash
strip -s target/release/anyrepair
```

3. **Remove Unused Dependencies**:
```bash
cargo tree --duplicates
```

### Benchmarking

Compare build sizes:

```bash
# Debug build
cargo build
du -sh target/debug/anyrepair

# Release build
cargo build --release
du -sh target/release/anyrepair

# Distribution build
cargo build --profile dist
du -sh target/dist/anyrepair
```

## Performance Impact

### Runtime Performance

- Release and distribution builds have identical runtime performance
- Minimal overhead from size optimizations
- No functional differences

### Memory Usage

- Smaller binaries use less memory when loaded
- Streaming repair still uses configurable buffer sizes
- No impact on memory efficiency

## Troubleshooting

### Binary Still Large

1. Check for debug symbols:
```bash
file target/release/anyrepair
```

2. Verify strip is enabled:
```bash
grep "strip = true" Cargo.toml
```

3. Check for unused dependencies:
```bash
cargo tree --unused
```

### Build Takes Too Long

1. Use release profile instead of dist:
```bash
cargo build --release
```

2. Disable LTO temporarily:
```bash
cargo build --release -Z build-std=std --config profile.release.lto=false
```

### Binary Not Working

1. Verify it's not corrupted:
```bash
./target/release/anyrepair --version
```

2. Check dependencies:
```bash
ldd target/release/anyrepair
```

## Configuration Reference

### Cargo.toml Profile Settings

```toml
[profile.release]
opt-level = "z"           # Size optimization (0-3, z for size)
lto = true                # Link Time Optimization (true, false, "thin", "fat")
codegen-units = 1         # Parallel codegen units (1-256)
strip = true              # Strip symbols (true/false)
panic = "abort"           # Panic behavior (unwind/abort)
```

### Environment Variables

```bash
# Verbose compilation
RUSTFLAGS="-v" cargo build --release

# Show optimization details
RUSTFLAGS="-C opt-level=z" cargo build --release

# Use all CPU cores
CARGO_BUILD_JOBS=8 cargo build --release
```

## Best Practices

1. **Always use release builds for production**
   - 94% smaller than debug
   - Better performance
   - No debug overhead

2. **Test release binaries before deployment**
   - Ensure functionality
   - Verify performance
   - Check size

3. **Use distribution profile for packaging**
   - Maximum optimization
   - Consistent builds
   - Reproducible results

4. **Monitor binary size in CI/CD**
   - Catch size regressions
   - Track optimization effectiveness
   - Alert on bloat

## Summary

AnyRepair is optimized for minimal binary size:

- ✅ Release build: 1.5 MB (94% smaller than debug)
- ✅ Distribution build: 1.5 MB (maximum optimization)
- ✅ Fast compilation: 28 seconds
- ✅ Full functionality maintained
- ✅ Production-ready

Use `cargo build --release` for production deployments!
