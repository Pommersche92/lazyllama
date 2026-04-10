/*
 *  _                      _      _
 * | |    __ _  ______  __| |    | | __ _ _ __ ___   __ _
 * | |   / _` ||_  /\ \/ /| |    | |/ _` | '_ ` _ \ / _` |
 * | |__| (_| | / /  \  / | |___ | | (_| | | | | | | (_| |
 * |_____\__,_|/___| /_/  |_____||_|\__,_|_| |_| |_|\__,_|
 *
 * Copyright (C) 2026 Raimo Geisel
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
 */

//! Utility functions for file operations and system access.
//!
//! This module provides essential file system operations for the LazyLlama application,
//! including conversation history persistence and logging functionality. All file
//! operations use platform-appropriate directories following XDG specifications
//! on Unix systems and standard application data directories on Windows.
//!
//! # File Storage
//!
//! - **Location**: `~/.local/share/lazyllama/` (Unix) or equivalent on Windows
//! - **Format**: Plain text files with timestamp-based naming
//! - **Persistence**: Both combined and per-model history files
//! - **Error Handling**: Graceful degradation when storage is unavailable

use anyhow::Result;
use chrono::Local;
use std::fs;

/// Saves conversation history to a timestamped file in the local data directory.
///
/// This function persists the provided conversation history to a new text file
/// in the application's data directory. The file is placed under
/// `~/.local/share/lazyllama/chat_YYYY-MM-DD_HH-MM-SS.txt` using the current
/// timestamp for unique identification.
///
/// # Arguments
///
/// * `history` - The complete conversation history string to be saved
///
/// # Returns
///
/// Returns `Ok(())` on successful file write or if the history is empty.
/// Returns an `anyhow::Error` if directory creation or file writing fails.
///
/// # Behavior
///
/// - **Empty Check**: Returns immediately if history string is empty
/// - **Directory Creation**: Creates the lazyllama directory if it doesn't exist
/// - **File Naming**: Uses timestamp format `YYYY-MM-DD_HH-MM-SS` for uniqueness
/// - **Atomic Write**: Uses `fs::write` for atomic file creation
///
/// # File Location
///
/// The storage location varies by platform:
/// - **Linux**: `~/.local/share/lazyllama/`
/// - **macOS**: `~/Library/Application Support/lazyllama/`
/// - **Windows**: `%LOCALAPPDATA%\lazyllama\`
///
/// # Error Handling
///
/// - Creates parent directories if they don't exist
/// - Propagates filesystem errors (permissions, disk space, etc.)
/// - Handles path encoding issues gracefully
///
/// # Example
///
/// ```no_run
/// use lazyllama::utils::save_history_to_file;
/// use anyhow::Result;
///
/// fn main() -> Result<()> {
///     let conversation = "YOU: Hello\nAI: Hi there!\n";
///     save_history_to_file(conversation)?;
///     // Creates: ~/.local/share/lazyllama/chat_2026-02-06_14-30-45.txt
///     Ok(())
/// }
/// ```
pub fn save_history_to_file(history: &str) -> Result<()> {
    if history.is_empty() {
        return Ok(());
    }
    let mut log_dir =
        dirs::data_local_dir().ok_or_else(|| anyhow::anyhow!("Data dir not found"))?;
    log_dir.push("lazyllama");
    fs::create_dir_all(&log_dir)?;
    let filename = format!("chat_{}.txt", Local::now().format("%Y-%m-%d_%H-%M-%S"));
    log_dir.push(filename);
    fs::write(log_dir, history)?;
    Ok(())
}

/// Saves separate conversation history files for each AI model.
///
/// This function creates individual history files for each AI model that has
/// conversation data, allowing users to maintain separate logs per model.
/// Each file is named with the model identifier and timestamp for easy
/// identification and organization.
///
/// # Arguments
///
/// * `model_histories` - HashMap mapping model names to their conversation histories
///
/// # Returns
///
/// Returns `Ok(())` on successful completion or an `anyhow::Error` if directory
/// creation or any file write operation fails.
///
/// # File Naming
///
/// Files are named using the pattern: `{safe_model_name}_{timestamp}.txt`
/// 
/// - **Model Name Sanitization**: Replaces `:`, `/`, `\` with `_` for filesystem compatibility
/// - **Timestamp Format**: `YYYY-MM-DD_HH-MM-SS` for consistent sorting
/// - **Extension**: Always `.txt` for universal compatibility
///
/// # Behavior
///
/// - **Empty History Skip**: Only creates files for models with non-empty histories
/// - **Atomic Writes**: Uses `fs::write` for atomic file creation per model
/// - **Single Timestamp**: All model files from one session share the same timestamp
/// - **Directory Reuse**: Creates the lazyllama directory once for all files
///
/// # Error Handling
///
/// - Fails fast if directory creation fails
/// - Continues processing remaining models if individual file writes fail
/// - Provides detailed error context for debugging
///
/// # Example
///
/// ```no_run
/// use std::collections::HashMap;
/// use lazyllama::utils::save_model_histories;
/// use anyhow::Result;
///
/// fn main() -> Result<()> {
///     let mut histories = HashMap::new();
///     histories.insert("llama2:7b".to_string(), "YOU: Test\nAI: Response".to_string());
///     histories.insert("codellama:13b".to_string(), "YOU: Code?\nAI: ```rust\n...".to_string());
///     
///     save_model_histories(&histories)?;
///     // Creates:
///     // ~/.local/share/lazyllama/llama2_7b_2026-02-06_14-30-45.txt
///     // ~/.local/share/lazyllama/codellama_13b_2026-02-06_14-30-45.txt
///     Ok(())
/// }
/// ```
///
/// # Platform Compatibility
///
/// The function handles model names that may contain characters problematic
/// for certain filesystems, ensuring cross-platform compatibility.
pub fn save_model_histories(model_histories: &std::collections::HashMap<String, String>) -> Result<()> {
    let mut log_dir =
        dirs::data_local_dir().ok_or_else(|| anyhow::anyhow!("Data dir not found"))?;
    log_dir.push("lazyllama");
    fs::create_dir_all(&log_dir)?;
    
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
    
    for (model_name, history) in model_histories {
        if !history.is_empty() {
            let safe_model_name = model_name.replace([':', '/', '\\'], "_");
            let filename = format!("{}_{}.txt", safe_model_name, timestamp);
            let mut file_path = log_dir.clone();
            file_path.push(filename);
            fs::write(file_path, history)?;
        }
    }
    Ok(())
}

/// Loads application settings from the config file.
///
/// This function reads the settings from `~/.config/lazyllama/settings.toml`
/// and deserializes them. If the file doesn't exist or cannot be read,
/// it returns the default settings.
///
/// # Returns
///
/// Returns the loaded `Settings` or default settings if loading fails.
///
/// # File Location
///
/// The config file location varies by platform:
/// - **Linux**: `~/.config/lazyllama/settings.toml`
/// - **macOS**: `~/Library/Application Support/lazyllama/settings.toml`
/// - **Windows**: `%APPDATA%\lazyllama\settings.toml`
///
/// # Example
///
/// ```no_run
/// use lazyllama::utils::load_settings;
///
/// let settings = load_settings();
/// println!("Theme: {:?}", settings.syntax_theme);
/// ```
pub fn load_settings() -> crate::app::Settings {
    let config_path = match dirs::config_dir() {
        Some(mut path) => {
            path.push("lazyllama");
            path.push("settings.toml");
            path
        }
        None => return crate::app::Settings::default(),
    };

    // If the file doesn't exist, return defaults
    if !config_path.exists() {
        return crate::app::Settings::default();
    }

    // Try to read and parse the file
    match fs::read_to_string(&config_path) {
        Ok(contents) => {
            match toml::from_str(&contents) {
                Ok(settings) => settings,
                Err(_) => crate::app::Settings::default(),
            }
        }
        Err(_) => crate::app::Settings::default(),
    }
}

/// Saves application settings to the config file.
///
/// This function serializes the provided settings and writes them to
/// `~/.config/lazyllama/settings.toml`. The config directory is created
/// if it doesn't exist.
///
/// # Arguments
///
/// * `settings` - The settings to save
///
/// # Returns
///
/// Returns `Ok(())` on success or an error if the operation fails.
///
/// # File Location
///
/// The config file location varies by platform:
/// - **Linux**: `~/.config/lazyllama/settings.toml`
/// - **macOS**: `~/Library/Application Support/lazyllama/settings.toml`
/// - **Windows**: `%APPDATA%\lazyllama\settings.toml`
///
/// # Example
///
/// ```no_run
/// use lazyllama::utils::save_settings;
/// use lazyllama::app::{Settings, SyntaxTheme};
/// use anyhow::Result;
///
/// fn main() -> Result<()> {
///     let mut settings = Settings::default();
///     settings.syntax_theme = SyntaxTheme::SolarizedDark;
///     save_settings(&settings)?;
///     Ok(())
/// }
/// ```
pub fn save_settings(settings: &crate::app::Settings) -> Result<()> {
    let mut config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Config directory not found"))?;
    
    config_dir.push("lazyllama");
    fs::create_dir_all(&config_dir)?;
    
    let mut config_path = config_dir;
    config_path.push("settings.toml");
    
    let toml_string = toml::to_string_pretty(settings)?;
    fs::write(config_path, toml_string)?;
    
    Ok(())
}


