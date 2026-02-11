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
| `C-q` | Quit application safely |
| `C-c` | Clear chat history |
| `C-s` | Manually toggle Autoscroll |
| `↑` / `↓` | **Switch between AI Models** (loads separate buffers per model) |
| `PgUp` / `PgDn` | Scroll history (activates Manual Mode) |
| `←` / `→` | Move cursor left/right in the input field |
| `Home` / `End` | Jump to start/end of the input line |
| `Ctrl` + `←` / `→` | Move cursor word-wise |
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

LazyLlama features a comprehensive test suite with 63 tests covering all functionality:

* **Unit Tests**: 53 modularized tests for individual components
* **Integration Tests**: 7 end-to-end tests for component interaction
* **Performance Benchmarks**: Continuous performance monitoring

### Running Tests

```bash
# Run all tests (unit, integration, doc tests)
cargo test

# Run only unit tests
cargo test --test test_unit

# Run benchmarks
cargo bench
```

For detailed testing information, test structure, and maintenance guidelines, see [TESTING.md](TESTING.md).

## 📄 License

This project is licensed under the **GPL-2.0-or-later**. See the [LICENSE](LICENSE) file for details.

## 📝 Changelog

### v0.4.0 - February 2026

* **📦 Dependency Updates**: Updated all dependencies to their latest versions
  * **ratatui**: 0.26 → 0.30.0 (TUI framework with improved frame API)
  * **crossterm**: 0.27 → 0.29.0 (terminal manipulation library)
  * **dirs**: 5.0 → 6.0.0 (platform-specific directory paths)
  * **tokio-stream**: 0.1 → 0.1.18 (async stream utilities)
  * **tempfile**: 3.8 → 3.25.0 (dev dependency for test file management)
  * **tokio-test**: 0.4 → 0.4.5 (dev dependency for async testing)
* **🔄 API Migration**: Updated code to use `frame.area()` instead of deprecated `frame.size()` method
* **🧪 Comprehensive Test Suite**: Added 63 tests for robust code quality assurance
  * **Unit Tests**: 53 modularized tests extracted to separate files for better maintainability
    * App functionality: 18 tests for cursor navigation, model management, text editing
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
