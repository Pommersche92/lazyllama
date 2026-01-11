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

//! Logik und Zustandsverwaltung der Anwendung.
//!
//! Dieses Modul enthält die [`App`]-Struktur, die den gesamten Zustand
//! der Terminal-UI sowie die Kommunikation mit Ollama verwaltet.

use ollama_rs::{generation::completion::request::GenerationRequest, Ollama};
use ratatui::{backend::CrosstermBackend, widgets::ListState, Terminal};
use std::io;
use std::time::Instant;
use tokio_stream::StreamExt;
use anyhow::Result;

/// Hält den gesamten Zustand der LazyLlama Anwendung.
pub struct App {
    /// Liste der verfügbaren lokalen Ollama-Modelle.
    pub models: Vec<String>,
    /// Zustand der Modellauswahl-Liste (aktueller Index).
    pub list_state: ListState,
    /// Der aktuelle Text im Eingabefeld.
    pub input: String,
    /// Der gesamte bisherige Chatverlauf als String.
    pub history: String,
    /// Die aktuelle vertikale Scroll-Position im Verlauf.
    pub scroll: u16,
    /// Flag, ob die Ansicht automatisch zum Ende springen soll.
    pub autoscroll: bool,
    /// Gibt an, ob gerade eine Anfrage an die KI läuft.
    pub is_loading: bool,
    /// Die Instanz des Ollama-Clients.
    pub ollama: Ollama,
    /// Zeitpunkt des Programmstarts (für Animationen).
    pub start_time: Instant,
}

impl App {
    /// Erstellt eine neue Instanz der Anwendung und lädt die Modellliste.
    pub async fn new() -> Self {
        let ollama = Ollama::default();
        let mut app = App {
            models: Vec::new(),
            list_state: ListState::default(),
            input: String::new(),
            history: String::new(),
            scroll: 0,
            autoscroll: true,
            is_loading: false,
            ollama,
            start_time: Instant::now(),
        };
        app.refresh_models().await;
        app
    }

    /// Aktualisiert die Liste der lokal verfügbaren KI-Modelle über Ollama.
    pub async fn refresh_models(&mut self) {
        if let Ok(models) = self.ollama.list_local_models().await {
            self.models = models.into_iter().map(|m| m.name).collect::<Vec<String>>();
            if !self.models.is_empty() {
                self.list_state.select(Some(0));
            }
        }
    }

    /// Sendet die aktuelle Eingabe an das gewählte Modell und streamt die Antwort.
    ///
    /// Die Antwort wird während des Empfangs direkt in `self.history` geschrieben
    /// und das Terminal wird bei jedem neuen Token aktualisiert.
    pub async fn send_query(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        if let Some(i) = self.list_state.selected() {
            let model = self.models[i].clone();
            let prompt = self.input.clone();

            self.history.push_str(&format!("\nYOU: {}\n\nAI: ", prompt));
            self.input.clear();
            self.is_loading = true;
            self.autoscroll = true;

            let request = GenerationRequest::new(model, prompt);
            let mut stream = self.ollama.generate_stream(request).await?;

            while let Some(res) = stream.next().await {
                if let Ok(responses) = res {
                    for resp in responses {
                        self.history.push_str(&resp.response);
                    }
                    terminal.draw(|f| crate::ui::ui(f, self))?;
                }
            }
            self.history.push_str("\n---\n");
            self.is_loading = false;
        }
        Ok(())
    }
}
