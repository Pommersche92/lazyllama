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

//! # LazyLlama
//!
//! A lightweight Terminal User Interface (TUI) client for Ollama AI models.
//! Provides real-time streaming responses, syntax highlighting for code blocks,
//! and automatic logging of chat sessions.
//!
//! ## Features
//! 
//! - Real-time streaming of AI model responses
//! - Markdown and code syntax highlighting
//! - Smart scrolling with autoscroll and manual modes
//! - Model switching with separate buffer management per model
//! - Configurable keybindings via TOML config file
//! - Automatic session logging
//!
//! ## Usage
//!
//! Run the application and use the following controls:
//! - `Ctrl+Q`: Quit the application
//! - `Ctrl+C`: Clear current model's chat history
//! - `Ctrl+S`: Toggle autoscroll mode
//! - `Ctrl+Arrow Up/Down`: Switch between AI models
//! - `Arrow Up/Down`: Move cursor between lines in multiline input
//! - `Page Up/Down`: Manual scrolling
//! - `Enter`: Send message to AI
//!
//! ## Default Keybindings
//!
//! All keybindings are configurable via the settings.toml config file.
//! See the [documentation](https://lazyllama.app/docs/keybindings) for details.
//!
//! Each AI model maintains separate input buffers, chat histories, and scroll positions.

mod app;
mod ui;
mod utils;

use crate::app::App;
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};

/// Matches an incoming key event against a configured keybinding action.
///
/// Returns `true` if the event matches the given key combination.
fn matches_key(key: &crossterm::event::KeyEvent, kc: &app::KeyCombination) -> bool {
    let is_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let is_shift = key.modifiers.contains(KeyModifiers::SHIFT);
    
    if kc.ctrl != is_ctrl {
        return false;
    }
    
    let code_matches = match key.code {
        KeyCode::Char(c) => {
            if kc.shift == is_shift {
                // Case-sensitive match
                c.to_string() == kc.key_code || c.to_string().to_uppercase() == kc.key_code.to_uppercase()
            } else if kc.shift && !is_shift {
                // Binding has Shift but event doesn't - allow case match
                c.to_uppercase().to_string() == kc.key_code.to_uppercase()
            } else {
                c.to_string().to_lowercase() == kc.key_code.to_lowercase()
            }
        }
        KeyCode::Enter => kc.key_code.eq_ignore_ascii_case("Enter") || kc.key_code.eq_ignore_ascii_case("Return"),
        KeyCode::Backspace => kc.key_code.eq_ignore_ascii_case("Backspace"),
        KeyCode::Delete => kc.key_code.eq_ignore_ascii_case("Delete"),
        KeyCode::Up => kc.key_code.eq_ignore_ascii_case("Up"),
        KeyCode::Down => kc.key_code.eq_ignore_ascii_case("Down"),
        KeyCode::Left => kc.key_code.eq_ignore_ascii_case("Left"),
        KeyCode::Right => kc.key_code.eq_ignore_ascii_case("Right"),
        KeyCode::Home => kc.key_code.eq_ignore_ascii_case("Home"),
        KeyCode::End => kc.key_code.eq_ignore_ascii_case("End"),
        KeyCode::PageUp => kc.key_code.eq_ignore_ascii_case("PageUp"),
        KeyCode::PageDown => kc.key_code.eq_ignore_ascii_case("PageDown"),
        KeyCode::Esc => kc.key_code.eq_ignore_ascii_case("Esc") || kc.key_code.eq_ignore_ascii_case("Escape"),
        _ => false,
    };
    
    code_matches && (kc.shift == is_shift || !kc.shift)
}

/// Main entry point for the LazyLlama application.
///
/// Initializes the terminal interface, sets up the event loop, and handles user input.
/// The function configures crossterm for raw mode terminal input, creates a ratatui
/// terminal backend, and manages the application lifecycle including proper cleanup
/// and history saving upon exit.
///
/// # Returns
///
/// Returns `Ok(())` on successful execution or an `anyhow::Error` if initialization
/// or runtime errors occur.
///
/// # Event Handling
///
/// The main loop processes key events using the configured keybindings from the
/// settings.toml config file. All keybindings can be customized by the user.
///
/// # Error Handling
///
/// Properly handles terminal setup/teardown and ensures cleanup even on errors.
#[tokio::main]
async fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new().await;
    let mut should_quit = false;

    // Initial draw
    terminal.draw(|f| ui::ui(f, &mut app))?;

    while !should_quit {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Windows-specific fix: Only process KeyPress events to prevent double input
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                if app.debug_keys {
                    app.debug_last_key = Some(format!("{:?}", key));
                }
                
                let kb = &app.settings.keybindings;
                
                // Handle settings dialog navigation first if it's open
                if app.show_settings_dialog {
                    let themes = app::SyntaxTheme::all();
                    let bindings = app::KeyBindings::all();
                    let theme_count = themes.len();
                    let binding_count = bindings.len();
                    
                    // Get the list of selectable indices (themes + keybindings, skips headers/spacers)
                    let selectable = ui::get_settings_selectable_indices(theme_count, binding_count);
                    let selectable_count = selectable.len();
                    
                    // Convert current selection to its position within the selectable list
                    let current_sel_pos = selectable.iter().position(|&i| i == app.settings_selection)
                        .unwrap_or(0);
                    
                    // Theme indices in selectable list: 0..theme_count-1
                    // Binding indices in selectable list: theme_count..theme_count+binding_count-1
                    let theme_sel_end = theme_count.saturating_sub(1); // last theme index in selectable list
                    let binding_sel_start = theme_count; // first binding index in selectable list
                    
                    // If we're recording a keybinding, capture the next key
                    if let Some(recording_name) = &app.settings_recording_binding.clone() {
                        // Esc cancels recording
                        if matches_key(&key, &kb.close_dialog) {
                            app.settings_recording_binding = None;
                        } else {
                            // Capture the key event as a new binding
                            let kc = key_event_to_key_combination(&key);
                            app.settings.keybindings.set(recording_name, kc);
                            app.settings_recording_binding = None;
                            // Save settings to disk
                            let _ = utils::save_settings(&app.settings);
                        }
                        terminal.draw(|f| ui::ui(f, &mut app))?;
                        continue;
                    }
                    
                    if matches_key(&key, &kb.close_dialog) {
                        app.show_settings_dialog = false;
                        continue;
                    } else if matches_key(&key, &kb.dialog_up) {
                        if current_sel_pos > 0 {
                            app.settings_selection = selectable[current_sel_pos - 1];
                        } else {
                            // Wrap to last selectable item
                            app.settings_selection = selectable[selectable_count - 1];
                        }
                    } else if matches_key(&key, &kb.dialog_down) {
                        if current_sel_pos + 1 < selectable_count {
                            app.settings_selection = selectable[current_sel_pos + 1];
                        } else {
                            // Wrap to first selectable item
                            app.settings_selection = selectable[0];
                        }
                    } else if matches_key(&key, &kb.dialog_apply) {
                        // Check if we're on a theme item
                        if current_sel_pos <= theme_sel_end {
                            if let Some(theme) = themes.get(current_sel_pos) {
                                app.settings.syntax_theme = theme.clone();
                                // Save settings to disk
                                let _ = utils::save_settings(&app.settings);
                            }
                        }
                        // Check if we're on a keybinding item
                        else if current_sel_pos >= binding_sel_start {
                            let binding_idx = current_sel_pos - binding_sel_start;
                            if let Some((name, _)) = bindings.get(binding_idx) {
                                app.settings_recording_binding = Some(name.to_string());
                            }
                        }
                    }
                    // Redraw after dialog event
                    terminal.draw(|f| ui::ui(f, &mut app))?;
                    continue; // Ignore other keys when dialog is open
                }
                
                // -- Quit --
                if matches_key(&key, &kb.quit) {
                    should_quit = true;
                }
                // -- Clear history --
                else if matches_key(&key, &kb.clear_history) {
                    app.history.clear();
                    app.scroll = 0;
                    app.autoscroll = true;
                    app.save_current_model_buffers();
                }
                // -- Copy selection --
                else if matches_key(&key, &kb.copy) {
                    if let Err(e) = app.copy_selection() {
                        if app.debug_keys {
                            app.debug_last_key = Some(format!("Copy failed: {}", e));
                        }
                    }
                }
                // -- Paste from clipboard --
                else if matches_key(&key, &kb.paste) {
                    if let Err(e) = app.paste_from_clipboard() {
                        if app.debug_keys {
                            app.debug_last_key = Some(format!("Paste failed: {}", e));
                        }
                    }
                }
                // -- Toggle autoscroll --
                else if matches_key(&key, &kb.toggle_autoscroll) {
                    app.autoscroll = !app.autoscroll;
                }
                // -- Open settings --
                else if matches_key(&key, &kb.open_settings) {
                    app.show_settings_dialog = !app.show_settings_dialog;
                    if app.show_settings_dialog {
                        // Start with the first selectable item (first theme, index 1)
                        app.settings_selection = 1;
                    }
                }
                // -- Cursor word left with selection (Ctrl+Shift+Left) --
                else if matches_key(&key, &kb.cursor_word_left_select) {
                    app.move_cursor_word_left_with_selection();
                }
                // -- Cursor word right with selection (Ctrl+Shift+Right) --
                else if matches_key(&key, &kb.cursor_word_right_select) {
                    app.move_cursor_word_right_with_selection();
                }
                // -- Cursor left with selection (Shift+Left) --
                else if matches_key(&key, &kb.cursor_left_select) {
                    app.move_cursor_left_with_selection();
                }
                // -- Cursor right with selection (Shift+Right) --
                else if matches_key(&key, &kb.cursor_right_select) {
                    app.move_cursor_right_with_selection();
                }
                // -- Home with selection (Shift+Home) --
                else if matches_key(&key, &kb.cursor_home_select) {
                    app.move_cursor_home_with_selection();
                }
                // -- End with selection (Shift+End) --
                else if matches_key(&key, &kb.cursor_end_select) {
                    app.move_cursor_end_with_selection();
                }
                // -- Cursor word left (Ctrl+Left) --
                else if matches_key(&key, &kb.cursor_word_left) {
                    app.move_cursor_word_left();
                }
                // -- Cursor word right (Ctrl+Right) --
                else if matches_key(&key, &kb.cursor_word_right) {
                    app.move_cursor_word_right();
                }
                // -- Home --
                else if matches_key(&key, &kb.cursor_home) {
                    app.move_cursor_home();
                }
                // -- End --
                else if matches_key(&key, &kb.cursor_end) {
                    app.move_cursor_end();
                }
                // -- Select previous model (Ctrl+Up) --
                else if matches_key(&key, &kb.select_previous_model) {
                    app.select_previous_model();
                }
                // -- Select next model (Ctrl+Down) --
                else if matches_key(&key, &kb.select_next_model) {
                    app.select_next_model();
                }
                // -- Cursor up --
                else if matches_key(&key, &kb.cursor_up) {
                    app.move_cursor_up();
                }
                // -- Cursor down --
                else if matches_key(&key, &kb.cursor_down) {
                    app.move_cursor_down();
                }
                // -- Page Up --
                else if matches_key(&key, &kb.page_up) {
                    app.autoscroll = false;
                    app.scroll = app.scroll.saturating_sub(5);
                }
                // -- Page Down --
                else if matches_key(&key, &kb.page_down) {
                    app.autoscroll = false;
                    app.scroll = app.scroll.saturating_add(5);
                }
                // -- Cursor left --
                else if matches_key(&key, &kb.cursor_left) {
                    app.move_cursor_left();
                }
                // -- Cursor right --
                else if matches_key(&key, &kb.cursor_right) {
                    app.move_cursor_right();
                }
                // -- Delete word left (Ctrl+Backspace) --
                else if matches_key(&key, &kb.delete_word_left) {
                    app.delete_word_left();
                }
                // -- Delete word right (Ctrl+Delete) --
                else if matches_key(&key, &kb.delete_word_right) {
                    app.delete_word_right();
                }
                // -- Delete forward --
                else if matches_key(&key, &kb.delete_forward) {
                    app.delete_forward();
                }
                // -- Backspace --
                else if matches_key(&key, &kb.backspace) {
                    app.backspace();
                }
                // -- Insert newline (Shift+Enter) --
                else if matches_key(&key, &kb.insert_newline) {
                    app.insert_char('\n');
                }
                // -- Send query (Enter) --
                else if matches_key(&key, &kb.send_query) {
                    if !app.input.is_empty() && !app.is_loading {
                        app.send_query(&mut terminal).await?;
                    }
                }
                // -- Character input (any other printable character) --
                else if let KeyCode::Char(c) = key.code {
                    if !is_system_key(&key) {
                        app.insert_char(c);
                    }
                }
                
                // Only redraw after an actual event occurred
                terminal.draw(|f| ui::ui(f, &mut app))?;
            }
        } else if app.is_loading {
            // Redraw during loading for spinner animation
            terminal.draw(|f| ui::ui(f, &mut app))?;
        } else if app.update_cursor_blink() {
            terminal.draw(|f| ui::ui(f, &mut app))?;
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    
    // Speichere die aktuellen Buffer vor dem Beenden
    app.save_current_model_buffers();
    
    // Speichere sowohl die allgemeine History als auch die modellspezifischen Histories
    utils::save_history_to_file(&app.history)?;
    utils::save_model_histories(&app.model_histories)?;
    Ok(())
}

/// Converts a crossterm KeyEvent into a KeyCombination for storage.
///
/// This is used when recording keybindings in the settings dialog.
fn key_event_to_key_combination(key: &crossterm::event::KeyEvent) -> app::KeyCombination {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);
    
    let key_code = match key.code {
        KeyCode::Char(c) => {
            if ctrl || shift {
                c.to_uppercase().to_string()
            } else {
                c.to_string()
            }
        }
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Delete => "Delete".to_string(),
        KeyCode::Up => "Up".to_string(),
        KeyCode::Down => "Down".to_string(),
        KeyCode::Left => "Left".to_string(),
        KeyCode::Right => "Right".to_string(),
        KeyCode::Home => "Home".to_string(),
        KeyCode::End => "End".to_string(),
        KeyCode::PageUp => "PageUp".to_string(),
        KeyCode::PageDown => "PageDown".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        _ => return app::KeyCombination::parse("C-q").unwrap(), // fallback
    };
    
    app::KeyCombination { key_code, ctrl, shift }
}

/// Returns true if this is a system-level key that should not be inserted as text.
fn is_system_key(key: &crossterm::event::KeyEvent) -> bool {
    let is_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    is_ctrl
}
