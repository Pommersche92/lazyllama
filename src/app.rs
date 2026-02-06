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

//! Application logic and state management.
//!
//! This module contains the [`App`] structure that manages the entire state
//! of the Terminal UI as well as communication with Ollama AI models.
//!
//! The core functionality includes:
//! - Model discovery and management
//! - Per-model buffer isolation (input, history, scroll position)
//! - Streaming response handling
//! - State persistence across model switches

use anyhow::Result;
use ollama_rs::{generation::completion::request::GenerationRequest, Ollama};
use ratatui::{backend::CrosstermBackend, widgets::ListState, Terminal};
use std::collections::HashMap;
use std::io;
use std::time::Instant;
use tokio_stream::StreamExt;

/// Main application state container for LazyLlama.
///
/// This structure holds all the necessary state for the Terminal UI including
/// model information, user input, conversation history, and UI state like
/// scrolling and loading indicators. Each AI model maintains separate buffers
/// for input text, conversation history, and scroll position to provide
/// seamless switching between different models.
pub struct App {
    /// List of locally available Ollama models discovered from the API.
    pub models: Vec<String>,
    /// State of the model selection list widget (currently selected index).
    pub list_state: ListState,
    /// Current text in the input field for the active model.
    pub input: String,
    /// Complete conversation history as a string for the active model.
    pub history: String,
    /// Separate input buffers maintained for each LLM model.
    pub model_inputs: HashMap<String, String>,
    /// Separate conversation histories maintained for each LLM model.
    pub model_histories: HashMap<String, String>,
    /// Separate scroll positions maintained for each LLM model.
    pub model_scrolls: HashMap<String, u16>,
    /// Current vertical scroll position in the conversation history.
    pub scroll: u16,
    /// Flag indicating whether the view should automatically scroll to the bottom.
    pub autoscroll: bool,
    /// Indicates whether an AI query is currently being processed.
    pub is_loading: bool,
    /// Instance of the Ollama client for API communication.
    pub ollama: Ollama,
    /// Timestamp of application start (used for UI animations like spinner).
    pub start_time: Instant,
}

impl App {
    /// Creates a new instance of the application and initializes the model list.
    ///
    /// This constructor performs the following initialization steps:
    /// 1. Creates a default Ollama client instance
    /// 2. Initializes all application state with default values
    /// 3. Sets up empty HashMaps for per-model buffer management
    /// 4. Automatically discovers and caches available models
    /// 5. Selects the first model if any are available
    ///
    /// # Returns
    ///
    /// A new `App` instance with populated model list and initialized state.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use lazyllama::app::App;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let app = App::new().await;
    ///     println!("Found {} models", app.models.len());
    /// }
    /// ```
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

    /// Refreshes the list of locally available AI models from Ollama.
    ///
    /// This method queries the Ollama API to discover all locally installed models
    /// and updates the internal model list. For any new models discovered, it
    /// initializes empty buffer entries (input text, conversation history, and
    /// scroll position). If models are available and none is currently selected,
    /// it automatically selects the first model and loads its buffers.
    ///
    /// # Behavior
    ///
    /// - Queries Ollama's `/api/tags` endpoint for local models
    /// - Preserves existing buffer data for known models
    /// - Initializes empty buffers for newly discovered models
    /// - Auto-selects first model if no selection exists
    /// - Loads buffers for the currently selected model
    ///
    /// # Error Handling
    ///
    /// Silently handles Ollama API errors by leaving the model list unchanged.
    /// This prevents the application from crashing if Ollama is temporarily
    /// unavailable or returns an error.
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

    /// Saves the current UI state to the per-model buffer storage.
    ///
    /// This method preserves the current application state (input text, conversation
    /// history, and scroll position) by storing it in the model-specific HashMaps.
    /// This allows seamless switching between models without losing context.
    ///
    /// # Behavior
    ///
    /// - Retrieves the currently selected model from `list_state`
    /// - Stores current `input` text in `model_inputs` HashMap
    /// - Stores current `history` string in `model_histories` HashMap
    /// - Stores current `scroll` position in `model_scrolls` HashMap
    /// - Does nothing if no model is currently selected
    ///
    /// # Usage
    ///
    /// Should be called before:
    /// - Switching to a different model
    /// - Modifying input text or scroll position
    /// - Application shutdown to preserve final state
    pub fn save_current_model_buffers(&mut self) {
        if let Some(index) = self.list_state.selected() {
            if let Some(model) = self.models.get(index) {
                self.model_inputs.insert(model.clone(), self.input.clone());
                self.model_histories.insert(model.clone(), self.history.clone());
                self.model_scrolls.insert(model.clone(), self.scroll);
            }
        }
    }

    /// Loads the stored state for the currently selected model.
    ///
    /// This method restores the application state (input text, conversation history,
    /// and scroll position) from the model-specific buffer storage. This enables
    /// each AI model to maintain its own independent conversation context.
    ///
    /// # Behavior
    ///
    /// - Retrieves the currently selected model from `list_state`
    /// - Loads stored `input` text from `model_inputs` HashMap (empty if not found)
    /// - Loads stored `history` from `model_histories` HashMap (empty if not found)
    /// - Loads stored `scroll` position from `model_scrolls` HashMap (0 if not found)
    /// - Updates current application state with the loaded values
    /// - Does nothing if no model is currently selected
    ///
    /// # Usage
    ///
    /// Should be called after:
    /// - Switching to a different model
    /// - Initial model selection during startup
    /// - Model list refresh if a model was previously selected
    pub fn load_current_model_buffers(&mut self) {
        if let Some(index) = self.list_state.selected() {
            if let Some(model) = self.models.get(index) {
                self.input = self.model_inputs.get(model).cloned().unwrap_or_default();
                self.history = self.model_histories.get(model).cloned().unwrap_or_default();
                self.scroll = *self.model_scrolls.get(model).unwrap_or(&0);
            }
        }
    }

    /// Switches to the next model in the list (Down arrow key behavior).
    ///
    /// This method implements circular navigation through the model list,
    /// wrapping from the last model back to the first. It preserves the
    /// current model's state before switching and loads the target model's
    /// stored state.
    ///
    /// # Behavior
    ///
    /// 1. Saves current model's state via `save_current_model_buffers()`
    /// 2. Calculates next index with wraparound (last → first)
    /// 3. Updates `list_state` selection to new index
    /// 4. Loads the new model's state via `load_current_model_buffers()`
    ///
    /// # Model Selection Logic
    ///
    /// - If at last model: wraps to index 0 (first model)
    /// - Otherwise: increments to next index
    /// - If no model selected: selects index 0
    /// - Handles empty model list gracefully
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

    /// Switches to the previous model in the list (Up arrow key behavior).
    ///
    /// This method implements circular navigation through the model list,
    /// wrapping from the first model back to the last. It preserves the
    /// current model's state before switching and loads the target model's
    /// stored state.
    ///
    /// # Behavior
    ///
    /// 1. Saves current model's state via `save_current_model_buffers()`
    /// 2. Calculates previous index with wraparound (first → last)
    /// 3. Updates `list_state` selection to new index
    /// 4. Loads the new model's state via `load_current_model_buffers()`
    ///
    /// # Model Selection Logic
    ///
    /// - If at first model (index 0): wraps to last model
    /// - Otherwise: decrements to previous index
    /// - If no model selected: selects index 0
    /// - Handles empty model list gracefully
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

    /// Sends the current input to the selected model and streams the response.
    ///
    /// This method handles the complete query lifecycle including prompt formatting,
    /// API communication, real-time response streaming, and UI updates. The response
    /// is written directly to `self.history` as tokens are received, providing
    /// immediate visual feedback to the user.
    ///
    /// # Arguments
    ///
    /// * `terminal` - Mutable reference to the terminal backend for real-time UI updates
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on successful completion or an `anyhow::Error` if the API
    /// request fails, streaming encounters an error, or terminal drawing fails.
    ///
    /// # Behavior
    ///
    /// 1. **Validation**: Ensures a model is selected before proceeding
    /// 2. **Formatting**: Adds user prompt to conversation history with "YOU:" label
    /// 3. **State Management**: Clears input field and saves current buffers
    /// 4. **UI Updates**: Sets loading state and enables autoscroll
    /// 5. **Streaming**: Sends request to Ollama and processes response tokens
    /// 6. **Real-time Display**: Updates terminal display for each received token
    /// 7. **Completion**: Adds separator and saves final state
    ///
    /// # Error Handling
    ///
    /// - Gracefully handles API connection errors
    /// - Continues processing partial responses if streaming is interrupted
    /// - Ensures loading state is cleared even on errors
    /// - Preserves conversation history even if request fails
    ///
    /// # Side Effects
    ///
    /// - Modifies `self.history` with formatted conversation
    /// - Clears `self.input` field
    /// - Updates `self.is_loading` state
    /// - Triggers terminal redraws for real-time display
    /// - Saves state to model-specific buffers
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
