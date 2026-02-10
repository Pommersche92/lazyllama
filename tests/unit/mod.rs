//! Unit test modules for LazyLlama
//!
//! This module exposes all individual unit test suites for different
//! components of the LazyLlama application.
//!
//! ## Module Structure
//!
//! - `test_app`: Application logic, state management, and model interaction
//! - `test_ui`: User interface rendering, text parsing, and display formatting
//! - `test_utils`: File system operations, logging, and utility functions
//! - `test_main`: Main application entry point and event loop testing

pub mod test_app;
pub mod test_ui;
pub mod test_utils;
pub mod test_main;