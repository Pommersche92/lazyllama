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
//! - Automatic session logging
//!
//! ## Usage
//!
//! Run the application and use the following controls:
//! - `Ctrl+Q`: Quit the application
//! - `Ctrl+C`: Clear current model's chat history
//! - `Ctrl+S`: Toggle autoscroll mode
//! - `Arrow Keys`: Switch between AI models
//! - `Page Up/Down`: Manual scrolling
//! - `Enter`: Send message to AI
//!
//! Each AI model maintains separate input buffers, chat histories, and scroll positions.

mod app;
mod ui;
mod utils;

use crate::app::App;
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};

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
/// The main loop processes the following key combinations:
/// - `Ctrl+Q`: Graceful application exit
/// - `Ctrl+C`: Clear current model's buffer
/// - `Ctrl+S`: Toggle autoscroll behavior
/// - `Up/Down Arrow`: Switch between AI models with buffer persistence
/// - `Page Up/Down`: Manual scrolling with autoscroll disable
/// - `Enter`: Send query to selected AI model
/// - `Backspace`: Delete characters from input
/// - `Character keys`: Add text to input buffer
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
                let is_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
                match (key.code, is_ctrl) {
                    (KeyCode::Char('q'), true) => should_quit = true,
                    (KeyCode::Char('c'), true) => {
                        // Lösche nur den aktuellen Modell-Buffer
                        app.history.clear();
                        app.scroll = 0;
                        app.autoscroll = true;
                        app.save_current_model_buffers();
                    }
                    (KeyCode::Char('s'), true) => app.autoscroll = !app.autoscroll,
                    (KeyCode::Up, _) => {
                        app.select_previous_model();
                    }
                    (KeyCode::Down, _) => {
                        app.select_next_model();
                    }
                    (KeyCode::PageUp, _) => {
                        app.autoscroll = false;
                        app.scroll = app.scroll.saturating_sub(5);
                    }
                    (KeyCode::PageDown, _) => {
                        app.autoscroll = false;
                        app.scroll = app.scroll.saturating_add(5);
                    }
                    (KeyCode::Enter, _) => {
                        if !app.input.is_empty() && !app.is_loading {
                            app.send_query(&mut terminal).await?;
                        }
                    }
                    (KeyCode::Char(c), false) => {
                        app.input.push(c);
                    }
                    (KeyCode::Backspace, _) => {
                        app.input.pop();
                    }
                    _ => {}
                }
                
                // Only redraw after an actual event occurred
                terminal.draw(|f| ui::ui(f, &mut app))?;
            }
        } else if app.is_loading {
            // Redraw during loading for spinner animation
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
