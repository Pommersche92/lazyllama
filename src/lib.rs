//! # LazyLlama Library
//! 
//! A comprehensive library interface for LazyLlama's core modules, primarily designed
//! to enable testing and potential future code reuse. This library crate exposes the
//! same modules that the main binary uses, but organized as a library structure.
//! 
//! ## Important Architecture Note
//! 
//! **The main binary (`src/main.rs`) does NOT use this library crate directly.**
//! Instead, it declares modules inline using `mod app;`, `mod ui;`, and `mod utils;`.
//! This is a common Rust pattern for binary applications that want to keep the
//! main executable lightweight while still providing library access for testing.
//! 
//! ## Purpose and Usage
//! 
//! This library crate serves specific purposes:
//! 
//! ### 1. **Testing Infrastructure**
//! 
//! The primary purpose of this library is to enable comprehensive testing:
//! - **Unit tests** can import individual modules: `use lazyllama::app::App;`
//! - **Integration tests** can test module interactions: `use lazyllama::*;`
//! - **Benchmark tests** can measure performance of specific functions
//! 
//! Without this library structure, testing internal functions and structs would 
//! be much more difficult since binary crates don't expose their internal modules.
//! 
//! ### 2. **Future Code Reuse**
//! 
//! While not currently used by main.rs, this library enables:
//! - Other applications to use LazyLlama components as dependencies
//! - Creating additional binaries that share the same core functionality
//! - Building plugins or extensions that interact with LazyLlama's modules
//! 
//! ### 3. **Documentation Generation**
//! 
//! The library structure allows `cargo doc` to generate comprehensive API
//! documentation for all public interfaces, making the codebase more accessible
//! to contributors and potential users.
//! 
//! ## Module Structure
//! 
//! The library exposes the same three modules that main.rs uses directly:
//! 
//! ### [`app`] - Application State and Logic
//! 
//! Contains the core `App` struct and all application state management:
//! - Model management and switching logic
//! - Per-model buffer isolation (input, history, cursor positions)
//! - Ollama API communication and streaming response handling
//! - User input processing and text manipulation
//! - Application lifecycle management
//! 
//! ### [`ui`] - Terminal User Interface Rendering
//! 
//! Handles all visual presentation and rendering logic:
//! - Terminal layout management using Ratatui
//! - Markdown parsing and syntax highlighting for code blocks
//! - Text styling and color application
//! - Conversation history formatting and display
//! - Model selection interface rendering
//! 
//! ### [`utils`] - Utility Functions and File Operations
//! 
//! Provides essential utility functions for:
//! - Conversation history persistence to local files
//! - Cross-platform file system operations
//! - Data directory management
//! - Error handling and file naming utilities
//! 
//! ## Current Architecture: Binary with Library Interface
//! 
//! ```text
//! src/main.rs (binary)
//! ├── mod app;          ← Direct module inclusion
//! ├── mod ui;           ← Direct module inclusion
//! └── mod utils;        ← Direct module inclusion
//! 
//! src/lib.rs (library for testing)
//! ├── pub mod app;      ← Same modules, but public for external access
//! ├── pub mod ui;       ← Same modules, but public for external access
//! └── pub mod utils;    ← Same modules, but public for external access
//! 
//! tests/ (integration tests)
//! └── use lazyllama::*; ← Uses the library interface
//! ```
//! 
//! ## Testing Usage Example
//! 
//! ```rust,no_run
//! // This is how tests use the library:
//! use lazyllama::app::App;
//! use lazyllama::ui::parse_history;
//! 
//! #[test]
//! fn test_app_functionality() {
//!     let mut app = create_test_app();
//!     app.insert_char('H');
//!     assert_eq!(app.input, "H");
//! }
//! ```
//! 
//! This dual structure (binary with inline modules + library interface) is a
//! common Rust pattern that provides the best of both worlds: a lean binary 
//! executable and comprehensive testing capabilities.

pub mod app;
pub mod ui;
pub mod utils;