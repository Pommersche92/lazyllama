# LazyLlama Tests

This document describes the comprehensive test suite for LazyLlama.

## Test Structure

### Unit Tests

Unit tests have been extracted into separate modular files in the `tests/unit/` directory for better organization:

- **`tests/unit/test_app.rs`**: 33 tests for App logic, cursor navigation, model management, text selection, and clipboard operations
- **`tests/unit/test_ui.rs`**: 13 tests for text parsing, history rendering, markdown processing
- **`tests/unit/test_utils.rs`**: 10 tests for filesystem operations, history storage
- **`tests/unit/test_main.rs`**: 12 tests for event handling, key combinations
- **`tests/test_unit.rs`**: Central entry point for all unit tests

#### Total: 68 unit tests

### Integration Tests

`tests/integration_tests.rs` contains 7 end-to-end tests that check the interaction between modules.

### Performance Tests

`benches/performance.rs` contains benchmarks for performance-critical functions.

## Running Tests

### Run all tests

```bash
cargo test
```

### Unit tests (all modularized tests)

```bash
# All unit tests
cargo test --test test_unit

# Or run the full test suite (includes unit, integration, doc tests)
cargo test
```

### Unit tests for a specific module

```bash
# App tests (33 tests - includes selection and clipboard)
cargo test --test test_unit unit::test_app

# UI tests (13 tests)
cargo test --test test_unit unit::test_ui

# Utils tests (10 tests) 
cargo test --test test_unit unit::test_utils

# Main tests (12 tests)
cargo test --test test_unit unit::test_main
```

### Integration tests

```bash
cargo test --test integration_tests
```

### Performance tests (benchmarks)

```bash
# Run all benchmarks with detailed output
cargo test --release --benches -- --nocapture --test-threads=1

# Run benchmarks without output details
cargo test --release --benches

# Note: These use #[test] instead of #[bench] for stable Rust compatibility
# Traditional 'cargo bench' won't work as expected
```

### Test output with details

```bash
cargo test -- --nocapture
```

### Tests with debug output

```bash
RUST_LOG=debug cargo test
```

## Test Categories

### Functionality Tests

- ✅ Text input and editing
- ✅ Cursor navigation (character, word, line)
- ✅ Text selection (character, word, line, bidirectional)
- ✅ Clipboard operations (copy, paste, with selection)
- ✅ Model switching and buffer management
- ✅ History parsing and markdown rendering
- ✅ Filesystem operations
- ✅ Unicode and emoji support
- ✅ Multiline input handling

### Edge Case Tests

- ✅ Empty inputs and boundary values
- ✅ Very long strings and histories
- ✅ Invalid Unicode sequences
- ✅ Filesystem errors and permissions
- ✅ Memory limits and performance boundaries

### Error Handling Tests

- ✅ Graceful degradation on errors
- ✅ Recovery after network problems
- ✅ Robustness with corrupted data

## Test Dependencies

The tests use the following dev dependencies:

```toml
[dev-dependencies]
tempfile = "3.25"     # Temporary files for FS tests
tokio-test = "0.4.5"  # Async test utilities
```

## Continuous Testing

### Pre-Commit Hook Setup

```bash
# Git hook for automatic tests before commits
echo '#!/bin/bash\ncargo test' > .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

### GitHub Actions (Example)

```yaml
name: Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --all-features
      - run: cargo test --test integration_tests
```

## Test Configuration

### Environment Variables for Tests

- `LAZY_LLAMA_TEST_TIMEOUT=30` - Timeout for integration tests
- `RUST_LOG=debug` - Detailed logging output
- `LAZY_LLAMA_SKIP_SLOW_TESTS=1` - Skips slow tests

### Feature Flags

```bash
# Tests with all features
cargo test --all-features

# Tests without optional features
cargo test --no-default-features

# Tests with specific features
cargo test --features "integration_tests bench"
```

## Performance Benchmarks

The benchmarks measure:

- **String Operations**: Character insertion, Unicode handling
- **Text Parsing**: History parsing, code block recognition
- **Cursor Navigation**: Character/byte index conversion
- **Memory Operations**: HashMap performance, allocations

### Running Benchmarks

```bash
# All benchmarks with detailed timing output
cargo test --release --benches -- --nocapture --test-threads=1

# All benchmarks (summary only)
cargo test --release --benches

# Specific benchmark test
cargo test --release --benches bench_string_operations -- --nocapture
```

**Note:** Benchmarks use `#[test]` attributes for stable Rust compatibility.
Traditional `cargo bench` (which requires unstable `#[bench]`) is not used.

## Test Maintenance

### Adding New Tests

1. **Unit tests**: Add to appropriate file in `tests/unit/` directory
   - App functionality: `tests/unit/test_app.rs`
   - UI functionality: `tests/unit/test_ui.rs`
   - Utils functionality: `tests/unit/test_utils.rs`
   - Main/Event handling: `tests/unit/test_main.rs`
2. **Integration tests**: Add to `tests/integration_tests.rs`
3. **Benchmarks**: Add to `benches/performance.rs`

### Test Structure Conventions

```rust
// For unit tests in tests/unit/*.rs
use lazyllama::app::App;  // Import from library

#[test]
fn test_function_name() {
    // Arrange
    let input = "test data";

    // Act
    let result = function_under_test(input);

    // Assert
    assert_eq!(result, expected_output);
}
```

### Mock Strategies

- **Filesystem**: `tempfile` for temporary directories
- **Network**: Mock clients without real API calls
- **Terminal**: Simulated events without real I/O

## Debugging Tests

### Test-specific Logging

```rust
#[test]
fn debug_test() {
    env_logger::init();
    log::debug!("Test debug information");
    // ... test code
}
```

### Test Isolation

Each test runs in an isolated environment:

- Own temporary directories
- No shared global state
- Deterministic results

## Known Test Limitations

### Platform-specific Tests

- Filesystem tests may react differently on various OS
- Unicode handling varies between platforms
- Memory benchmarks are hardware-dependent

### External Dependencies

- Tests avoid real Ollama API calls
- Terminal tests use mocks
- Network tests are stubbed

## Reporting

### Coverage Reports

```bash
# With tarpaulin (Linux/macOS)
cargo install cargo-tarpaulin
cargo tarpaulin --out Html

# With grcov (all platforms)
cargo install grcov
CARGO_INCREMENTAL=0 RUSTFLAGS='-C instrument-coverage' cargo test
grcov . --binary-path ./target/debug/ -s . -t html --branch --ignore-not-existing -o ./coverage/
```

### Test Metrics

- **Total Tests**: 78 tests (68 unit + 7 integration + 3 doc tests)
- **Code Coverage**: Target 80%+
- **Performance Regression**: Max 5% deviation
- **Memory Usage**: Under defined limits

### Test Distribution

- **Unit Tests**: 68 tests across 4 modules
  - App logic: 33 tests
    - Basic editing: 6 tests (insert, backspace, delete)
    - Cursor navigation: 6 tests (char, word, home/end)
    - Text selection: 11 tests (all selection modes)
    - Clipboard operations: 3 tests (copy, paste, integration)
    - Model management: 3 tests
    - Unicode handling: 2 tests
    - Internal utilities: 3 tests
  - UI functionality: 13 tests
  - Utils operations: 10 tests
  - Event handling: 12 tests
- **Integration Tests**: 7 end-to-end tests
- **Documentation Tests**: 3 doc tests

## Maintenance Schedule

- **Weekly**: Check performance benchmarks
- **At Releases**: Run complete test suite
- **After Dependencies Updates**: Compatibility tests
- **Quarterly**: Test suite review and optimization

## Test Coverage Details

### Text Selection Tests (11 tests)

Comprehensive coverage of text selection features:

1. **`test_clear_selection`**: Clearing selection state
2. **`test_get_selection_range`**: Range calculation and normalization
3. **`test_get_selected_text`**: Text extraction with Unicode support
4. **`test_move_cursor_left_with_selection`**: Character-wise leftward selection
5. **`test_move_cursor_right_with_selection`**: Character-wise rightward selection
6. **`test_move_cursor_word_left_with_selection`**: Word-wise leftward selection
7. **`test_move_cursor_word_right_with_selection`**: Word-wise rightward selection
8. **`test_move_cursor_home_with_selection`**: Selection to start of input
9. **`test_move_cursor_end_with_selection`**: Selection to end of input
10. **`test_bidirectional_selection`**: Forward and backward selection equivalence
11. **`test_movement_clears_selection`**: Selection clearing on normal movement

### Clipboard Tests (3 tests)

Clipboard integration with graceful fallback:

1. **`test_copy_selection`**: Copy selected text to clipboard
   - Error handling for no selection
   - Graceful handling of clipboard unavailability (CI/headless)
2. **`test_paste_from_clipboard`**: Paste clipboard content
   - Insert at cursor position
   - Replace active selection
   - Graceful handling of clipboard unavailability
3. **`test_insert_text_at_cursor`**: Text insertion with selection replacement
   - Insert without selection
   - Replace selection on insert
   - Unicode text handling

### Integration Tests (2 tests)

1. **`test_typing_clears_selection`**: Typing behavior with active selection
2. **`test_movement_clears_selection`**: Movement clearing selection state

### Unicode and Boundary Tests

All selection and clipboard tests include:

- Multi-byte UTF-8 character handling (emojis, umlauts)
- Boundary condition testing (start/end of input)
- Empty input handling
- Bidirectional selection support

### Test Reliability Notes

**Clipboard Tests**: These tests are designed to work in both interactive and CI environments:

- Gracefully handle clipboard unavailability
- Test function behavior without requiring actual clipboard access
- Platform-agnostic implementation using `arboard` crate
