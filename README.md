# 🦙 LazyLlama

**LazyLlama** is a lightweight, fast Terminal User Interface (TUI) client for [Ollama](https://ollama.com/). It is designed for running local AI models with minimal overhead and intuitive, Emacs-inspired controls directly in your terminal.

## ✨ Features

* **Real-time Streaming:** Responses are generated live, providing immediate feedback.
* **Markdown Support:** Automatic formatting for headers, lists, and bold text.
* **Code Highlighting:** Syntax blocks are visually separated with custom borders and background colors.
* **Smart Scrolling:** * `AUTOSCROLL`: Automatically follows the AI output.
  * `MANUAL SCROLL`: Locks the view (🔒) when you use PageUp/Down, allowing you to read previous messages undisturbed.
* **Model Management:** Easily switch between installed Ollama models using arrow keys with **separate input/output buffers per model**.
* **Smart Buffer Management:** Each LLM maintains its own chat history, input text, and scroll position.
* **Automatic Logging:** Every chat session is automatically saved as a text file in `~/.local/share/lazyllama/` (both combined and per-model histories).
* **Performance:** Built with Rust and Ratatui for ultra-low latency and minimal resource footprint.

## 🚀 Installation

### Prerequisites

* [Rust](https://rustup.rs/) (Stable)
* [Ollama](https://ollama.com/) (must be running in the background)

### Build from Source

1. Clone the repository:

   ```bash
   git clone https://github.com/Pommersche92/lazyllama.git
   cd lazyllama
   ```

1. Install it system-wide:

   ```bash
   cargo install --path .
   ```

## ⌨️ Controls

| Key | Action |
| --- | --- |
| `Enter` | Send message / Re-activate Autoscroll |
| `Shift` + `Enter` | Insert newline (multiline input) |
| `C-q` | Quit application safely |
| `C-c` | Clear chat history |
| `C-s` | Manually toggle Autoscroll |
| `Ctrl` + `Shift` + `C` | Copy selected text to clipboard |
| `Ctrl` + `Shift` + `V` | Paste text from clipboard |
| `↑` / `↓` | **Switch between AI Models** (loads separate buffers per model) |
| `PgUp` / `PgDn` | Scroll history (activates Manual Mode) |
| `←` / `→` | Move cursor left/right in the input field |
| `Shift` + `←` / `→` | Select text character-wise |
| `Home` / `End` | Jump to start/end of the input line |
| `Shift` + `Home` / `End` | Select text to start/end of line |
| `Ctrl` + `←` / `→` | Move cursor word-wise |
| `Ctrl` + `Shift` + `←` / `→` | Select text word-wise |
| `Backspace` | Delete character before the cursor |
| `Delete` | Delete character after the cursor |
| `Ctrl` + `Backspace` | Delete previous word |
| `Ctrl` + `Delete` | Delete next word |

Optional debug:

* `LAZYLLAMA_DEBUG_KEYS=1` shows key/scroll/render info in the status bar.

## 🛠 Project Structure

The project follows a modular design for easy maintainability:

* `main.rs`: Entry point and terminal event handling.
* `app.rs`: State management and Ollama API integration.
* `ui.rs`: Rendering logic and Markdown parsing.
* `utils.rs`: File system operations and session logging.

## 📖 Documentation

You can generate the full technical documentation locally:

```bash
cargo doc --no-deps --open
```

## 🧪 Testing

LazyLlama features a comprehensive test suite with 78 tests covering all functionality:

* **Unit Tests**: 68 modularized tests for individual components
* **Integration Tests**: 7 end-to-end tests for component interaction
* **Performance Benchmarks**: Continuous performance monitoring

### Running Tests

```bash
# Run all tests (unit, integration, doc tests)
cargo test

# Run only unit tests
cargo test --test test_unit

# Run benchmarks with detailed output
cargo test --release --benches -- --nocapture --test-threads=1
```

### Performance Benchmarks

LazyLlama is optimized for ultra-low latency with all operations completing well within the 16ms frame budget for 60 FPS rendering:

| Category | Operation | Avg. Time | Performance |
| ---------- | ----------- | ----------- | ------------- |
| **String Operations** | Character Insertion | 323 ns | ⚡ |
| | Unicode Insertion (🦀) | 126 ns | ⚡⚡ |
| | Pattern Matching | 7.45 µs | ✅ |
| **Text Parsing** | History Parsing | 2.10 µs | ⚡ |
| | Line Iteration | 9.06 µs | ✅ |
| | Character Counting | 203 ns | ⚡⚡ |
| **Cursor Operations** | Index Conversion | 1 ns | ⚡⚡⚡ |
| | Word Boundary Detection | 4.28 µs | ⚡ |
| **Memory** | HashMap Lookups | 6.79 µs | ✅ |
| | Vec Pre-allocation | 586 ns | ⚡⚡ |
| **Real-World** | Large History (100KB) | 64.3 µs | ✅ |
| | Model Switching | 12.7 µs | ✅ |

**Legend**: ⚡⚡⚡ < 10 ns | ⚡⚡ < 1 µs | ⚡ < 10 µs | ✅ < 100 µs

All benchmarks run on optimized release builds. Run `cargo test --release --benches` to execute the full benchmark suite.

For detailed testing information, test structure, and maintenance guidelines, see [TESTING.md](TESTING.md).

## 📄 License

This project is licensed under the **GPL-2.0-or-later**. See the [LICENSE](LICENSE) file for details.

## 📝 Changelog

### v0.4.0 - February 2026

* **📏 Dynamic Input Height**: Input field now automatically expands from 1 to 4 lines based on content
  * Grows dynamically when pressing `Shift+Enter` to add newlines
  * Minimum height: 1 line, Maximum height: 4 lines
  * Provides more comfortable editing space for longer prompts
* **✨ Improved Multiline Rendering**: Proper line-by-line rendering with selection and cursor support
  * Each line is rendered independently with correct styling
  * Selection highlighting works correctly across multiple lines
  * Cursor position is accurately displayed within multiline text
* **📝 Multiline Input**: Press `Shift+Enter` to insert line breaks in the input field for multiline messages
* **✂️ Text Selection**: Select text in the input field using keyboard shortcuts
  * `Shift` + `←` / `→`: Select text character by character
  * `Shift` + `Ctrl` + `←` / `→`: Select text word by word
  * `Shift` + `Home` / `End`: Select from cursor to start/end of input
* **📋 Clipboard Operations**: Copy and paste functionality for the input field
  * `Ctrl` + `Shift` + `C`: Copy selected text to clipboard
  * `Ctrl` + `Shift` + `V`: Paste text from clipboard at cursor position
* **🎨 Visual Selection Feedback**: Selected text is highlighted with a blue background for clear visibility
* **🔄 Smart Text Insertion**: Pasting text automatically replaces any active selection
* **📦 Dependency**: Added `arboard` library for cross-platform clipboard access
* **📦 Dependency Updates**: Updated all dependencies to their latest versions
  * **ratatui**: 0.26 → 0.30.0 (TUI framework with improved frame API)
  * **crossterm**: 0.27 → 0.29.0 (terminal manipulation library)
  * **dirs**: 5.0 → 6.0.0 (platform-specific directory paths)
  * **tokio-stream**: 0.1 → 0.1.18 (async stream utilities)
  * **tempfile**: 3.8 → 3.25.0 (dev dependency for test file management)
  * **tokio-test**: 0.4 → 0.4.5 (dev dependency for async testing)
* **🔄 API Migration**: Updated code to use `frame.area()` instead of deprecated `frame.size()` method
* **🧪 Comprehensive Test Suite**: Added 78 tests for robust code quality assurance
  * **Unit Tests**: 68 modularized tests extracted to separate files for better maintainability
    * App functionality: 33 tests for cursor navigation, model management, text editing, selection, clipboard
    * UI components: 13 tests for markdown parsing, syntax highlighting, text rendering
    * Utilities: 10 tests for filesystem operations, history management
    * Event handling: 12 tests for key combinations, terminal integration
  * **Integration Tests**: 7 end-to-end tests for component interaction
  * **Performance Benchmarks**: Continuous monitoring for performance-critical functions
* **🔧 Improved Testability**: Made internal functions public for comprehensive unit testing
* **📚 Enhanced Documentation**: Updated test documentation and added detailed testing guide

### v0.3.0 - February 2026

* **🧹 Clean Chat Redraw**: Clears the chat area before rendering to prevent leftover characters when scrolling
* **⌨️ Input Cursor**: Horizontal cursor navigation with Left/Right and a blinking caret in the input field
* **🏁 Home/End Navigation**: Jump to start/end of the input line
* **🧭 Word-wise Movement**: Ctrl+Left/Right moves by words with smart separators
* **🗑 Word Deletion**: Ctrl+Backspace deletes previous word, Ctrl+Delete deletes next word
* **⌦ Delete Key Support**: Delete removes the character after the cursor
* **🧪 Dev Key Debug Mode**: Optional status bar debug via `LAZYLLAMA_DEBUG_KEYS=1`

### v0.2.0 - February 2026

* **🎯 Per-Model Buffer Management**: Each LLM now maintains separate input buffers, chat histories, and scroll positions
* **🔄 Smart Model Switching**: Arrow keys now seamlessly switch between models while preserving individual states  
* **💾 Enhanced Logging**: Separate history files are saved for each model on application exit
* **🎨 Improved UI**: Model list shows buffer status indicators and current model highlighting
* **🪟 Windows Compatibility**: Fixed double character input issue on Windows by filtering key event types
* **📖 Translated Documentation**: Documentation in source code AI-translated from german to english
  * Please report any gramatical errors, AI weirdness and/or other inaccuracies in the github issues.

### v0.1.0 - Initial Release

* Basic TUI interface for Ollama
* Real-time streaming responses
* Markdown and code highlighting
* Smart scrolling with autoscroll/manual modes
* Model selection and automatic logging

---

*Developed with ❤️ in the black forest.*
