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
//! Ein leichtgewichtiger TUI-Client für Ollama-KI-Modelle.
//! Erlaubt das Streamen von Antworten, Syntax-Highlighting für Code und automatisches Logging.

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

/// Hauptfunktion: Initialisiert das Terminal und startet den Event-Loop.
#[tokio::main]
async fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new().await;
    let mut should_quit = false;

    while !should_quit {
        terminal.draw(|f| ui::ui(f, &mut app))?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                let is_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
                match (key.code, is_ctrl) {
                    (KeyCode::Char('q'), true) => should_quit = true,
                    (KeyCode::Char('c'), true) => { app.history.clear(); app.scroll = 0; app.autoscroll = true; },
                    (KeyCode::Char('s'), true) => app.autoscroll = !app.autoscroll,
                    (KeyCode::Up, _) => {
                        let i = match app.list_state.selected() {
                            Some(i) => if i == 0 { app.models.len() - 1 } else { i - 1 },
                            None => 0,
                        };
                        app.list_state.select(Some(i));
                    }
                    (KeyCode::Down, _) => {
                        let i = match app.list_state.selected() {
                            Some(i) => if i >= app.models.len() - 1 { 0 } else { i + 1 },
                            None => 0,
                        };
                        app.list_state.select(Some(i));
                    }
                    (KeyCode::PageUp, _) => { app.autoscroll = false; app.scroll = app.scroll.saturating_sub(5); }
                    (KeyCode::PageDown, _) => { app.autoscroll = false; app.scroll = app.scroll.saturating_add(5); }
                    (KeyCode::Enter, _) => {
                        if !app.input.is_empty() && !app.is_loading {
                            app.send_query(&mut terminal).await?;
                        }
                    }
                    (KeyCode::Char(c), false) => app.input.push(c),
                    (KeyCode::Backspace, _) => { app.input.pop(); }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    utils::save_history_to_file(&app.history)?;
    Ok(())
}
