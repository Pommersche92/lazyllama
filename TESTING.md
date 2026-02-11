# LazyLlama Tests

This document describes the comprehensive test suite for LazyLlama.

## Test Structure

### Unit Tests

Unit tests have been extracted into separate modular files in the `tests/unit/` directory for better organization:

- **`tests/unit/test_app.rs`**: 18 tests for App logic, cursor navigation, model management
- **`tests/unit/test_ui.rs`**: 13 tests for text parsing, history rendering, markdown processing  
- **`tests/unit/test_utils.rs`**: 10 tests for filesystem operations, history storage
- **`tests/unit/test_main.rs`**: 12 tests for event handling, key combinations
- **`tests/test_unit.rs`**: Central entry point for all unit tests

#### Total: 53 unit tests

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
# App tests (18 tests)
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
# Run benchmarks
cargo bench

# Run specific benchmark groups
cargo bench string_operations
cargo bench text_parsing
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
- ✅ Model switching and buffer management
- ✅ History parsing and markdown rendering
- ✅ Filesystem operations
- ✅ Unicode and emoji support

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
# All benchmarks
cargo bench

# Specific benchmark group
cargo bench string_operations
cargo bench text_parsing
```

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

- **Total Tests**: 63 tests (53 unit + 7 integration + 3 doc tests)
- **Code Coverage**: Target 80%+
- **Performance Regression**: Max 5% deviation
- **Memory Usage**: Under defined limits

### Test Distribution

- **Unit Tests**: 53 tests across 4 modules
  - App logic: 18 tests
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
