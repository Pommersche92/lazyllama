//! Central unit test collection for LazyLlama
//! 
//! This file imports all unit test modules and makes them available
//! as a unified test binary.
//!
//! ## Test Module Organization
//!
//! - **unit/**: Contains individual module unit tests
//!   - `test_app.rs`: Tests for application logic and state management
//!   - `test_ui.rs`: Tests for user interface rendering and text processing
//!   - `test_utils.rs`: Tests for utility functions and file operations  
//!   - `test_main.rs`: Tests for main application entry point and event handling
//!
//! ## Running Unit Tests
//!
//! ```bash
//! cargo test test_unit
//! ```
//!
//! Or run specific module tests:
//! ```bash
//! cargo test unit::test_app
//! ```

mod unit;