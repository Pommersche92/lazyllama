# 🦙 LazyLlama

**LazyLlama** is a lightweight, fast Terminal User Interface (TUI) client for [Ollama](https://ollama.com/). It is designed for running local AI models with minimal overhead and intuitive, Emacs-inspired controls directly in your terminal.

## ✨ Features

* **Real-time Streaming:** Responses are generated live, providing immediate feedback.
* **Markdown Support:** Automatic formatting for headers, lists, and bold text.
* **Code Highlighting:** Syntax blocks are visually separated with custom borders and background colors.
* **Smart Scrolling:** * `AUTOSCROLL`: Automatically follows the AI output.
    * `MANUAL SCROLL`: Locks the view (🔒) when you use PageUp/Down, allowing you to read previous messages undisturbed.
* **Model Management:** Easily switch between installed Ollama models using arrow keys.
* **Automatic Logging:** Every chat session is automatically saved as a text file in `~/.local/share/lazyllama/`.
* **Performance:** Built with Rust and Ratatui for ultra-low latency and minimal resource footprint.

## 🚀 Installation

### Prerequisites
* [Rust](https://rustup.rs/) (Stable)
* [Ollama](https://ollama.com/) (must be running in the background)

### Build from Source
1. Clone the repository:
   ```bash
   git clone [https://github.com/Pommersche92/lazyllama.git](https://github.com/Pommersche92/lazyllama.git)
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
| `C-q` | Quit application safely |
| `C-c` | Clear chat history |
| `C-s` | Manually toggle Autoscroll |
| `↑` / `↓` | Select AI Model |
| `PgUp` / `PgDn` | Scroll history (activates Manual Mode) |

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

## 📄 License

This project is licensed under the **GPL-2.0-or-later**. See the [LICENSE](LICENSE) file for details.

---

*Developed with ❤️ in the black forest.*
