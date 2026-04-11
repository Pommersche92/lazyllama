<div align="center">
  <img src="icon.png" alt="LazyLlama icon" width="128" /><br><br>
  <h1>LazyLlama</h1>
</div>

**LazyLlama** is a lightweight, fast Terminal User Interface (TUI) client for [Ollama](https://ollama.com/). It is designed for running local AI models with minimal overhead and intuitive, Emacs-inspired controls directly in your terminal.

## ✨ Features

* **Real-time Streaming:** Responses are generated live, providing immediate feedback.
* **GitHub-Flavored Markdown:** Comprehensive support for markdown formatting including bold, italic, strikethrough, inline code, headers, lists, task lists, blockquotes, links, and horizontal rules.
* **Syntax Highlighting:** Code blocks with language tags are automatically syntax-highlighted using `syntect` with support for 100+ programming languages (Rust, Python, JavaScript, TypeScript, Go, C++, Java, and many more).
* **Smart Scrolling:** * `AUTOSCROLL`: Automatically follows the AI output.
  * `MANUAL SCROLL`: Locks the view (🔒) when you use PageUp/Down, allowing you to read previous messages undisturbed.
* **Model Management:** Easily switch between installed Ollama models using Ctrl+arrow keys with **separate input/output buffers per model**.
* **Smart Buffer Management:** Each LLM maintains its own chat history, input text, and scroll position.
* **Automatic Logging:** Every chat session is automatically saved as a text file in `~/.local/share/lazyllama/` (both combined and per-model histories).
* **Performance:** Built with Rust and Ratatui for ultra-low latency and minimal resource footprint.

## 🚀 Installation

### From crates.io (Recommended)

```bash
cargo install lazyllama
```

### Arch Linux (AUR)

```bash
# Source build
yay -S lazyllama

# Pre-built binary
yay -S lazyllama-bin
```

### Flatpak (All Linux Distributions)

```bash
flatpak install flathub app.pommersche.LazyLlama
flatpak run app.pommersche.LazyLlama
```

### Windows

Download the latest Windows x64 ZIP from [GitHub Releases](https://github.com/Pommersche92/lazyllama/releases) and extract it to a directory in your PATH.

**Note:** Ollama must be installed and running separately. Visit [ollama.com](https://ollama.com/) for installation instructions.

### Build from Source

#### Prerequisites

* [Rust](https://rustup.rs/) (Stable)
* [Ollama](https://ollama.com/) (must be running in the background)

#### Steps

1. Clone the repository:

   ```bash
   git clone https://github.com/Pommersche92/lazyllama.git
   cd lazyllama
   ```

2. Install it system-wide:

   ```bash
   cargo install --path .
   ```

## ⌨️ Controls

| Key | Action |
| --- | --- |
| `Enter` | Send message / Re-activate Autoscroll |
| `Shift` + `Enter` | Insert newline (multiline input) |
| `C-q` | Quit application safely |
| `C-o` | Open settings dialog (theme selection) |
| `C-c` | Clear chat history |
| `C-s` | Manually toggle Autoscroll |
| `Ctrl` + `Shift` + `C` | Copy selected text to clipboard |
| `Ctrl` + `Shift` + `V` | Paste text from clipboard |
| `Ctrl` + `↑` / `↓` | **Switch between AI Models** (loads separate buffers per model) |
| `↑` / `↓` | Move cursor up/down between lines in multiline input |
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

## ⚙️ Configuration

LazyLlama stores user preferences in a configuration file that persists between sessions.

### Settings Dialog

Press `Ctrl+O` to open the settings dialog where you can customize:

* **Syntax Highlighting Theme**: Choose from 10 curated themes optimized for both dark and light terminal backgrounds
  * **Dark themes**: Ocean Dark (default), Monokai, Solarized Dark, Dracula, Nord
  * **Light themes**: Ocean Light, Solarized Light, GitHub, Monokai Light, Gruvbox Light

Settings are automatically saved when you apply changes and will be restored on the next application start.

### Config File Location

Settings are stored in:
* **Linux/macOS**: `~/.config/lazyllama/settings.toml`
* **Windows**: `%APPDATA%\lazyllama\settings.toml`

You can manually edit the config file if needed. Example:

```toml
[settings]
syntax_theme = "SolarizedDark"
```

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

LazyLlama features a comprehensive test suite with 88 tests covering all functionality:

* **Unit Tests**: 72 modularized tests for individual components
* **Integration Tests**: 7 end-to-end tests for component interaction
* **Doc Tests**: 9 documentation tests (6 active, 3 ignored)
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
| **String Operations** | Character Insertion | 384 ns | ⚡⚡ |
| | Unicode Insertion (🦀) | 163 ns | ⚡⚡ |
| | Pattern Matching | 7.55 µs | ⚡ |
| **Text Parsing** | History Parsing | 2.11 µs | ⚡ |
| | Line Iteration | 7.23 µs | ⚡ |
| | Character Counting | 206 ns | ⚡⚡ |
| **Cursor Operations** | Index Conversion | 1 ns | ⚡⚡⚡ |
| | Word Boundary Detection | 2.81 µs | ⚡ |
| **Memory** | HashMap Lookups | 6.97 µs | ⚡ |
| | Vec Pre-allocation | 576 ns | ⚡⚡ |
| **Real-World** | Large History (100KB) | 66.2 µs | ✅ |
| | Model Switching | 13.0 µs | ✅ |

**Legend**: ⚡⚡⚡ < 10 ns | ⚡⚡ < 1 µs | ⚡ < 10 µs | ✅ < 100 µs

*Benchmark results from 2 March 2026 · Release build · `cargo test --release --benches`*

All benchmarks run on optimized release builds. Run `cargo test --release --benches` to execute the full benchmark suite.

For detailed testing information, test structure, and maintenance guidelines, see [TESTING.md](TESTING.md).

## 📄 License

This project is licensed under the **GPL-2.0-or-later**. See the [LICENSE](LICENSE) file for details.

## 📝 Changelog

### v0.5.2 - April 2026

* **🎨 Windows Icon Embedding**: Application icon now properly embedded in Windows executables
  * Added `build.rs` to compile icon resources into `.exe` files
  * Icons display correctly in Windows Explorer and taskbar
  * Automatic generation of `icon.ico` from `icon.png` during build process
* **🖼️ AppImage Icon Integration**: Improved icon handling for AppImage packages
  * Uses project root `icon.png` for all AppImage builds
  * Fallback to generated placeholder if icon is missing
  * Better visual identification in application launchers
* **🔧 Build Infrastructure**: Enhanced release scripts with icon generation
  * Automatic `.ico` conversion using ImageMagick before Windows cross-compilation
  * Streamlined icon workflow across all distribution formats
* **📝 GitHub-Flavored Markdown Support**: Feature-complete markdown rendering
  * Added `pulldown-cmark` for standards-compliant markdown parsing
  * **Inline formatting**: Bold (`**text**`), italic (`*text*`), strikethrough (`~~text~~`), inline code (`` `code` ``)
  * **Headers**: All levels (`#` to `######`) with visual hierarchy
  * **Lists**: Unordered (`-`, `*`, `+`) and ordered (`1.`, `2.`, etc.)
  * **Task lists**: Unchecked (`- [ ]`) and checked (`- [x]`) items with visual indicators (☐/☑)
  * **Blockquotes**: `> quote` with green styling and italic formatting
  * **Horizontal rules**: `---`, `***`, `___` rendered as visual separators
  * **Links**: `[text](url)` with blue underlined text (URL hidden in TUI)
  * Preserves existing YOU:/AI: label coloring and code block framing
* **🎨 Syntax Highlighting**: Language-aware code block highlighting
  * Added `syntect` library for professional-grade syntax highlighting
  * Supports 100+ programming languages with accurate tokenization
  * Uses base16-ocean.dark theme optimized for terminal readability
  * Automatically detects language from code fence tags (e.g., `` ```rust ``, `` ```python ``, `` ```js ``)
  * Comprehensive language tag mapping: supports common aliases (js→JavaScript, py→Python, rs→Rust, cs→C#, etc.)
  * Highlights keywords, strings, comments, functions, types, and more
  * Preserves code block framing with language labels

### v0.5.1 - March 2026

* **🐛 Scroll Fix**: Correctly calculate visual scroll height when `Wrap` is active
  * `total_lines` is now computed as the sum of wrapped visual rows per logical line
  * Fixes issue where the scroll area ended too early after multiple messages, making the last responses unreachable
  * Auto-scroll now reliably positions at the true bottom of the conversation

### v0.5.0 - February 2026

* **📦 GitHub Releases**: Automated release pipeline with pre-built binaries for multiple platforms
  * Linux x64 tarballs for easy installation on any distribution
  * Windows x64 ZIP archives with native binaries
  * AppImage packages for universal Linux compatibility
* **🐧 AppImage Support**: Portable single-file executables that run on any Linux distribution
  * No installation required - just download and run
  * Includes all dependencies in a self-contained package
* **🪟 Windows Binary Distribution**: Official Windows builds available through GitHub Releases
  * Pre-compiled executables for x64 architecture
  * No Rust toolchain required for Windows users
* **📦 AUR Packages**: Official Arch Linux User Repository packages
  * `lazyllama`: Build from source via crates.io
  * `lazyllama-bin`: Pre-built binary package for faster installation
* **🌐 Website Redesign**: Complete overhaul of project website with modern glassmorphism design
  * Responsive layout optimized for all devices
  * Custom typography with Satoshi font
  * Interactive code examples and installation guides
* **🤖 Automated Build Pipeline**: Comprehensive release automation scripts
  * One-command release workflow to all platforms
  * Automatic version management and checksums
  * Integrated testing before deployment
* **📚 Documentation Improvements**: Enhanced documentation across the project
  * Detailed build script documentation in `scripts/README.md`
  * Comprehensive testing guide in `TESTING.md`
  * Updated installation instructions for all platforms
* **🔧 Build Infrastructure**: New AppImage build system replacing Flatpak
  * Simplified build process with `linuxdeploy`
  * Faster compilation and smaller package size
  * Better integration with GitHub release workflow

### v0.4.1 - February 2026

* **🔼 Extended Input Height**: Input field now expands up to 5 lines (increased from 4)
  * Maximum height: 5 lines, giving more space for longer prompts
* **📍 Smart Cursor Scrolling**: Input field automatically scrolls to keep cursor visible
  * When cursor moves beyond visible area, the input scrolls automatically
  * Smooth navigation in long multiline inputs
* **⌨️ Improved Keybindings**:
  * `Ctrl` + `↑` / `↓`: Switch between AI models (changed from plain arrow keys)
  * `↑` / `↓`: Move cursor up/down between lines in multiline input (new feature)
  * More intuitive navigation in multiline text editing

### v0.4.0 - February 2026

* **📏 Dynamic Input Height**: Input field now automatically expands from 1 to 5 lines based on content
  * Grows dynamically when pressing `Shift+Enter` to add newlines
  * Minimum height: 1 line, Maximum height: 5 lines
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
* **🔄 Smart Model Switching**: Ctrl+Arrow keys now seamlessly switch between models while preserving individual states  
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
