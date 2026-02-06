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

use anyhow::Result;
use ollama_rs::{generation::completion::request::GenerationRequest, Ollama};
use ratatui::{backend::CrosstermBackend, widgets::ListState, Terminal};
use std::collections::HashMap;
use std::io;
use std::time::Instant;
use tokio_stream::StreamExt;

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
    /// Separate Input-Buffer für jedes LLM-Modell.
    pub model_inputs: HashMap<String, String>,
    /// Separate History-Buffer für jedes LLM-Modell.
    pub model_histories: HashMap<String, String>,
    /// Separate Scroll-Positionen für jedes LLM-Modell.
    pub model_scrolls: HashMap<String, u16>,
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
            model_inputs: HashMap::new(),
            model_histories: HashMap::new(),
            model_scrolls: HashMap::new(),
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
            
            // Initialisiere Buffer für neue Modelle
            for model in &self.models {
                self.model_inputs.entry(model.clone()).or_insert_with(String::new);
                self.model_histories.entry(model.clone()).or_insert_with(String::new);
                self.model_scrolls.entry(model.clone()).or_insert(0);
            }
            
            if !self.models.is_empty() {
                self.list_state.select(Some(0));
                self.load_current_model_buffers();
            }
        }
    }

    /// Speichert die aktuellen Buffer in den per-Modell-Speicher.
    pub fn save_current_model_buffers(&mut self) {
        if let Some(index) = self.list_state.selected() {
            if let Some(model) = self.models.get(index) {
                self.model_inputs.insert(model.clone(), self.input.clone());
                self.model_histories.insert(model.clone(), self.history.clone());
                self.model_scrolls.insert(model.clone(), self.scroll);
            }
        }
    }

    /// Lädt die Buffer für das aktuell gewählte Modell.
    pub fn load_current_model_buffers(&mut self) {
        if let Some(index) = self.list_state.selected() {
            if let Some(model) = self.models.get(index) {
                self.input = self.model_inputs.get(model).cloned().unwrap_or_default();
                self.history = self.model_histories.get(model).cloned().unwrap_or_default();
                self.scroll = *self.model_scrolls.get(model).unwrap_or(&0);
            }
        }
    }

    /// Wechselt zum nächsten Modell (mit Pfeiltaste nach unten).
    pub fn select_next_model(&mut self) {
        self.save_current_model_buffers();
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.models.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.load_current_model_buffers();
    }

    /// Wechselt zum vorherigen Modell (mit Pfeiltaste nach oben).
    pub fn select_previous_model(&mut self) {
        self.save_current_model_buffers();
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.models.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.load_current_model_buffers();
    }

    /// Sendet die aktuelle Eingabe an das gewählte Modell und streamt die Antwort.
    ///
    /// Die Antwort wird während des Empfangs direkt in `self.history` geschrieben
    /// und das Terminal wird bei jedem neuen Token aktualisiert.
    pub async fn send_query(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> Result<()> {
        if let Some(i) = self.list_state.selected() {
            let model = self.models[i].clone();
            let prompt = self.input.clone();

            self.history.push_str(&format!("\nYOU: {}\n\nAI: ", prompt));
            self.input.clear();
            
            // Speichere die aktualisierten Buffer für das aktuelle Modell
            self.save_current_model_buffers();
            
            self.is_loading = true;
            self.autoscroll = true;

            let request = GenerationRequest::new(model.clone(), prompt);
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
            
            // Speichere die finale History für dieses Modell
            self.save_current_model_buffers();
        }
        Ok(())
    }
}
