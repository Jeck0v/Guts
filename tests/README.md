# Tests Organization

This directory contains all tests for the Guts Git implementation, organized by execution order and type.

## 🏗️ Test Structure

### 1. Plumbing Commands (01-04) - Run First
These test the low-level Git functionality that other commands depend on:

- `01_plumbing_hash_object.rs` - Test `guts hash-object` command
- `02_plumbing_cat_file.rs` - Test `guts cat-file` command  
- `03_plumbing_write_tree.rs` - Test `guts write-tree` command
- `04_plumbing_commit_tree.rs` - Test `guts commit-tree` command

### 2. Porcelain Commands (05-09) - Run Second  
These test the user-facing Git commands:

- `05_porcelain_init.rs` - Test `guts init` command
- `06_porcelain_add.rs` - Test `guts add` command
- `07_porcelain_commit.rs` - Test `guts commit` command

### 3. Integration Tests (10+) - Run Last
These test complete workflows and interactions between commands:

- `10_integration_workflow.rs` - Complete Git workflow test (init → add → commit → modify → commit)

### 4. Manual Tests (`manual/`) - Not run in CI
These require human interaction or are for performance testing:

- `manual/test_manual_tui.rs` - TUI functionality tests
- Add more manual tests here as needed

## 🚀 Running Tests

### All automated tests (CI/CD):
```bash
cargo test
```

### Only plumbing tests:
```bash
cargo test --test "01_*" --test "02_*" --test "03_*" --test "04_*"
```

### Only porcelain tests:
```bash  
cargo test --test "05_*" --test "06_*" --test "07_*"
```

### Only integration tests:
```bash
cargo test --test "10_*"
```

### Manual tests (ignored by default):
```bash
cargo test --test manual -- --ignored
```

## 📝 Test Naming Convention

- **Prefix numbers** ensure execution order
- **Category names** make purpose clear:
  - `01-04`: `plumbing_*` for low-level commands
  - `05-09`: `porcelain_*` for user commands  
  - `10+`: `integration_*` for workflow tests
- **Manual tests** go in `manual/` directory

## ✅ Test Requirements

### All tests should:
- ✅ Use temporary directories (`assert_fs::TempDir`)
- ✅ Clean up automatically (temp dirs auto-delete)
- ✅ Have descriptive assertion messages
- ✅ Test both success and error conditions
- ✅ Be deterministic (no flaky tests)

### Integration tests should:
- ✅ Test realistic user workflows
- ✅ Verify state transitions (untracked → staged → committed)
- ✅ Test command interactions
- ✅ Use representative file structures

## 🎯 GitHub Actions Integration

Tests run in this order in CI:
1. **Plumbing tests** - Foundation functionality
2. **Porcelain tests** - User-facing commands  
3. **Integration tests** - Complete workflows

Manual tests are excluded from CI using the `#[ignore]` attribute.