# Mutation testing (`cargo-mutants`)

Scoped mutation testing covers critical repair logic under `src/json.rs`, `json_util.rs`, `repairer_base.rs`, `format_detection.rs`, and `pattern.rs`. Configuration: [`.cargo/mutants.toml`](../.cargo/mutants.toml).

## Prerequisites

```bash
cargo install cargo-mutants
```

**Disk:** default runs copy the tree into `/tmp` and need several GB free. If builds fail with `No space left on device`, free space (`cargo clean`) or use **in-place** mode (see below).

## Recommended commands

**Single file, in-place, library unit tests only** (works on tight disks; do not combine with `-j`):

```bash
cargo mutants --no-config --in-place -f 'src/repairer_base.rs' -- --lib
cargo mutants --no-config --in-place -f 'src/format_detection.rs' -- --lib
```

**Full scoped run** (when disk allows; uses `.cargo/mutants.toml`):

```bash
cargo mutants --in-place -f 'src/repairer_base.rs' -f 'src/format_detection.rs' -f 'src/pattern.rs' -- --lib
```

Output is written under `mutants.out/` (gitignored) unless `--in-place` is used.

## Interpreting results

| Result | Meaning |
|--------|---------|
| **caught** | A test failed — good coverage for that mutant |
| **missed** | No test failed — add a regression test (see `mutation_guard_*` in `format_detection.rs`) |
| **unviable** | Mutant did not compile or could not run |
| **FAILED baseline** | Fix build/tests before trusting the run |

If a run aborts mid **in-place** session, restore touched sources from git:

```bash
git checkout -- src/
```

## Status (2026-11-13)

- `src/repairer_base.rs`: 27 mutants — **24 caught**, 3 unviable, **0 missed** (`--in-place`, `--lib`)
- `src/format_detection.rs`: partial run surfaced **7 missed** mutants; addressed with `mutation_guard_*` unit tests

Re-run `format_detection` after adding tests to confirm **0 missed**.
