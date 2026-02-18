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
use std::env;
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
    /// Separate cursor positions maintained for each LLM model.
    pub model_cursors: HashMap<String, usize>,
    /// Separate text selections maintained for each LLM model.
    pub model_selections: HashMap<String, Option<usize>>,
    /// Separate conversation histories maintained for each LLM model.
    pub model_histories: HashMap<String, String>,
    /// Separate scroll positions maintained for each LLM model.
    pub model_scrolls: HashMap<String, u16>,
    /// Current vertical scroll position in the conversation history.
    pub scroll: u16,
    /// Current vertical scroll position in the input field.
    pub input_scroll: u16,
    /// Current cursor position in the input field (character index).
    pub cursor_pos: usize,
    /// Starting position of text selection (None if no selection).
    pub selection_start: Option<usize>,
    /// Flag indicating whether the view should automatically scroll to the bottom.
    pub autoscroll: bool,
    /// Indicates whether an AI query is currently being processed.
    pub is_loading: bool,
    /// Instance of the Ollama client for API communication.
    pub ollama: Ollama,
    /// Timestamp of application start (used for UI animations like spinner).
    pub start_time: Instant,
    /// Timestamp of last cursor blink toggle.
    pub last_cursor_blink: Instant,
    /// Whether the input cursor should be visible this frame.
    pub cursor_visible: bool,
    /// Enables on-screen debug info when true.
    pub debug_keys: bool,
    /// Last key event debug string (when enabled).
    pub debug_last_key: Option<String>,
    /// Frame counter for render debugging.
    pub render_count: u64,
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
        let debug_keys = env::var("LAZYLLAMA_DEBUG_KEYS")
            .map(|v| v != "0" && v.to_lowercase() != "false")
            .unwrap_or(false);
        let mut app = App {
            models: Vec::new(),
            list_state: ListState::default(),
            input: String::new(),
            cursor_pos: 0,
            selection_start: None,
            history: String::new(),
            model_inputs: HashMap::new(),
            model_cursors: HashMap::new(),
            model_selections: HashMap::new(),
            model_histories: HashMap::new(),
            model_scrolls: HashMap::new(),
            scroll: 0,
            input_scroll: 0,
            autoscroll: true,
            is_loading: false,
            ollama,
            start_time: Instant::now(),
            last_cursor_blink: Instant::now(),
            cursor_visible: true,
            debug_keys,
            debug_last_key: None,
            render_count: 0,
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
                self.model_cursors.entry(model.clone()).or_insert(0);
                self.model_selections.entry(model.clone()).or_insert(None);
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
                self.model_cursors.insert(model.clone(), self.cursor_pos);
                self.model_selections.insert(model.clone(), self.selection_start);
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
                self.cursor_pos = *self.model_cursors.get(model).unwrap_or(&0);
                self.selection_start = *self.model_selections.get(model).unwrap_or(&None);
                self.history = self.model_histories.get(model).cloned().unwrap_or_default();
                self.scroll = *self.model_scrolls.get(model).unwrap_or(&0);
                self.clamp_cursor();
            }
        }
    }

    /// Inserts a character at the current cursor position.
    ///
    /// This method performs a character-aware insertion (not byte-based),
    /// advances the cursor by one character, and resets the blink timer
    /// so the caret remains visible after input. If there is an active
    /// selection, it deletes the selection first.
    pub fn insert_char(&mut self, c: char) {
        // Delete selection first if it exists
        if self.selection_start.is_some() {
            self.delete_selection();
        }
        
        let byte_idx = self.char_index_to_byte_index(self.cursor_pos);
        self.input.insert(byte_idx, c);
        self.cursor_pos = self.cursor_pos.saturating_add(1);
        self.reset_cursor_blink();
    }

    /// Deletes the character immediately before the cursor.
    ///
    /// This is the standard Backspace behavior: it removes one character to
    /// the left of the caret, shifts the cursor left by one, and resets the
    /// blink timer.
    pub fn backspace(&mut self) {
        if self.cursor_pos == 0 {
            return;
        }
        let remove_idx = self.cursor_pos - 1;
        let byte_idx = self.char_index_to_byte_index(remove_idx);
        self.input.remove(byte_idx);
        self.cursor_pos = self.cursor_pos.saturating_sub(1);
        self.reset_cursor_blink();
    }

    /// Deletes the word immediately before the cursor.
    ///
    /// Word boundaries use `is_word_char` rules, treating non-alphanumeric
    /// characters (except underscore) as separators. Leading separators to
    /// the left of the caret are removed along with the word.
    pub fn delete_word_left(&mut self) {
        if self.cursor_pos == 0 {
            return;
        }
        let chars: Vec<char> = self.input.chars().collect();
        let mut i = self.cursor_pos.min(chars.len());

        while i > 0 && !Self::is_word_char(chars[i - 1]) {
            i -= 1;
        }
        while i > 0 && Self::is_word_char(chars[i - 1]) {
            i -= 1;
        }

        if i != self.cursor_pos {
            let start = self.char_index_to_byte_index(i);
            let end = self.char_index_to_byte_index(self.cursor_pos);
            self.input.replace_range(start..end, "");
            self.cursor_pos = i;
            self.reset_cursor_blink();
        }
    }

    /// Deletes the character at the cursor position.
    ///
    /// This is the standard Delete behavior: it removes the character under
    /// the caret (to the right), leaving the cursor position unchanged.
    pub fn delete_forward(&mut self) {
        let len = self.input.chars().count();
        if self.cursor_pos >= len {
            return;
        }
        let byte_idx = self.char_index_to_byte_index(self.cursor_pos);
        self.input.remove(byte_idx);
        self.reset_cursor_blink();
    }

    /// Deletes the word immediately after the cursor.
    ///
    /// Word boundaries use `is_word_char` rules, treating non-alphanumeric
    /// characters (except underscore) as separators. Leading separators to
    /// the right of the caret are removed along with the word.
    pub fn delete_word_right(&mut self) {
        let chars: Vec<char> = self.input.chars().collect();
        let len = chars.len();
        if self.cursor_pos >= len {
            return;
        }
        let mut i = self.cursor_pos.min(len);

        while i < len && !Self::is_word_char(chars[i]) {
            i += 1;
        }
        while i < len && Self::is_word_char(chars[i]) {
            i += 1;
        }

        if i != self.cursor_pos {
            let start = self.char_index_to_byte_index(self.cursor_pos);
            let end = self.char_index_to_byte_index(i);
            self.input.replace_range(start..end, "");
            self.reset_cursor_blink();
        }
    }

    /// Moves the cursor one character to the left.
    ///
    /// No-op if already at the beginning of the input. Resets the blink
    /// timer to keep the caret visible after navigation. Clears any active selection.
    pub fn move_cursor_left(&mut self) {
        self.clear_selection();
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            self.reset_cursor_blink();
        }
    }

    /// Moves the cursor one character to the right.
    ///
    /// No-op if already at the end of the input. Resets the blink timer to
    /// keep the caret visible after navigation. Clears any active selection.
    pub fn move_cursor_right(&mut self) {
        self.clear_selection();
        let len = self.input.chars().count();
        if self.cursor_pos < len {
            self.cursor_pos += 1;
            self.reset_cursor_blink();
        }
    }

    /// Moves the cursor to the start of the input.
    ///
    /// This is the Home key behavior. Resets the blink timer if the cursor
    /// position changes. Clears any active selection.
    pub fn move_cursor_home(&mut self) {
        self.clear_selection();
        if self.cursor_pos != 0 {
            self.cursor_pos = 0;
            self.reset_cursor_blink();
        }
    }

    /// Moves the cursor to the end of the input.
    ///
    /// This is the End key behavior. Resets the blink timer if the cursor
    /// position changes. Clears any active selection.
    pub fn move_cursor_end(&mut self) {
        self.clear_selection();
        let len = self.input.chars().count();
        if self.cursor_pos != len {
            self.cursor_pos = len;
            self.reset_cursor_blink();
        }
    }

    /// Moves the cursor one line up in multiline input.
    ///
    /// Attempts to preserve the column position when moving to the previous line.
    /// If the previous line is shorter, moves to its end. Clears any active selection.
    pub fn move_cursor_up(&mut self) {
        self.clear_selection();
        let chars: Vec<char> = self.input.chars().collect();
        
        // Find current line start and column position
        let mut line_start = 0;
        let mut col = 0;
        for i in 0..self.cursor_pos.min(chars.len()) {
            if chars[i] == '\n' {
                line_start = i + 1;
                col = 0;
            } else {
                col += 1;
            }
        }
        
        // If we're on the first line, do nothing
        if line_start == 0 {
            return;
        }
        
        // Find previous line start
        let mut prev_line_start = 0;
        for i in 0..line_start - 1 {
            if chars[i] == '\n' {
                prev_line_start = i + 1;
            }
        }
        
        // Calculate new cursor position on previous line
        let prev_line_len = line_start - prev_line_start - 1; // -1 for the newline
        let new_cursor_pos = prev_line_start + col.min(prev_line_len);
        
        if new_cursor_pos != self.cursor_pos {
            self.cursor_pos = new_cursor_pos;
            self.reset_cursor_blink();
        }
    }

    /// Moves the cursor one line down in multiline input.
    ///
    /// Attempts to preserve the column position when moving to the next line.
    /// If the next line is shorter, moves to its end. Clears any active selection.
    pub fn move_cursor_down(&mut self) {
        self.clear_selection();
        let chars: Vec<char> = self.input.chars().collect();
        let len = chars.len();
        
        // Find current line start and column position
        let mut line_start = 0;
        let mut col = 0;
        for i in 0..self.cursor_pos.min(len) {
            if chars[i] == '\n' {
                line_start = i + 1;
                col = 0;
            } else {
                col += 1;
            }
        }
        
        // Find next line start (current line end)
        let mut next_line_start = None;
        for i in line_start..len {
            if chars[i] == '\n' {
                next_line_start = Some(i + 1);
                break;
            }
        }
        
        // If there's no next line, do nothing
        let next_line_start = match next_line_start {
            Some(pos) => pos,
            None => return,
        };
        
        // Find next line end
        let mut next_line_end = len;
        for i in next_line_start..len {
            if chars[i] == '\n' {
                next_line_end = i;
                break;
            }
        }
        
        // Calculate new cursor position on next line
        let next_line_len = next_line_end - next_line_start;
        let new_cursor_pos = next_line_start + col.min(next_line_len);
        
        if new_cursor_pos != self.cursor_pos {
            self.cursor_pos = new_cursor_pos;
            self.reset_cursor_blink();
        }
    }

    /// Moves the cursor one word to the left.
    ///
    /// Word boundaries use `is_word_char` rules, treating non-alphanumeric
    /// characters (except underscore) as separators. Leading separators to
    /// the left are skipped before landing on the previous word boundary.
    /// Clears any active selection.
    pub fn move_cursor_word_left(&mut self) {
        self.clear_selection();
        if self.cursor_pos == 0 {
            return;
        }
        let chars: Vec<char> = self.input.chars().collect();
        let mut i = self.cursor_pos.min(chars.len());

        while i > 0 && !Self::is_word_char(chars[i - 1]) {
            i -= 1;
        }
        while i > 0 && Self::is_word_char(chars[i - 1]) {
            i -= 1;
        }

        if i != self.cursor_pos {
            self.cursor_pos = i;
            self.reset_cursor_blink();
        }
    }

    /// Moves the cursor one word to the right.
    ///
    /// Word boundaries use `is_word_char` rules, treating non-alphanumeric
    /// characters (except underscore) as separators. Leading separators to
    /// the right are skipped before landing on the next word boundary.
    /// Clears any active selection.
    pub fn move_cursor_word_right(&mut self) {
        self.clear_selection();
        let chars: Vec<char> = self.input.chars().collect();
        let len = chars.len();
        let mut i = self.cursor_pos.min(len);

        while i < len && !Self::is_word_char(chars[i]) {
            i += 1;
        }
        while i < len && Self::is_word_char(chars[i]) {
            i += 1;
        }

        if i != self.cursor_pos {
            self.cursor_pos = i;
            self.reset_cursor_blink();
        }
    }

    /// Toggles cursor blink state when enough time has elapsed.
    ///
    /// Returns `true` when a toggle occurs so the caller can trigger a
    /// redraw; otherwise returns `false` to avoid unnecessary updates.
    pub fn update_cursor_blink(&mut self) -> bool {
        if self.last_cursor_blink.elapsed().as_millis() >= 500 {
            self.cursor_visible = !self.cursor_visible;
            self.last_cursor_blink = Instant::now();
            return true;
        }
        false
    }

    pub fn reset_cursor_blink(&mut self) {
        self.cursor_visible = true;
        self.last_cursor_blink = Instant::now();
    }

    pub fn clamp_cursor(&mut self) {
        let len = self.input.chars().count();
        if self.cursor_pos > len {
            self.cursor_pos = len;
        }
    }

    pub fn char_index_to_byte_index(&self, char_index: usize) -> usize {
        self.input
            .char_indices()
            .nth(char_index)
            .map(|(idx, _)| idx)
            .unwrap_or_else(|| self.input.len())
    }

    pub fn is_word_char(c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }

    /// Clears the current text selection.
    ///
    /// This method removes any active selection by setting `selection_start` to `None`.
    /// After calling this method, no text will be highlighted in the input field.
    ///
    /// # Behavior
    ///
    /// - Sets `selection_start` to `None`
    /// - Does nothing if no selection is active
    /// - Does not affect cursor position or input content
    ///
    /// # Usage
    ///
    /// This is typically called automatically when:
    /// - Moving the cursor without Shift modifier
    /// - Inserting new text
    /// - Deleting text
    pub fn clear_selection(&mut self) {
        self.selection_start = None;
    }

    /// Starts a text selection at the current cursor position.
    ///
    /// This internal method initializes a new text selection by recording the
    /// current cursor position as the selection anchor. If a selection is already
    /// active, this method does nothing, preserving the original anchor point.
    ///
    /// # Behavior
    ///
    /// - Records current `cursor_pos` as `selection_start` if no selection exists
    /// - Preserves existing `selection_start` if selection is already active
    /// - Does not affect cursor position or input content
    ///
    /// # Usage
    ///
    /// Called automatically by cursor movement methods when Shift is pressed:
    /// - `move_cursor_left_with_selection`
    /// - `move_cursor_right_with_selection`
    /// - Word-wise and line-wise selection methods
    fn start_selection(&mut self) {
        if self.selection_start.is_none() {
            self.selection_start = Some(self.cursor_pos);
        }
    }

    /// Returns the selection range as (start, end) with start <= end.
    ///
    /// This method calculates the normalized selection range where the start
    /// position is always less than or equal to the end position, regardless
    /// of the selection direction (forward or backward).
    ///
    /// # Returns
    ///
    /// - `Some((start, end))`: A tuple with start <= end if selection is active
    /// - `None`: If no selection is active
    ///
    /// # Behavior
    ///
    /// The range is always normalized:
    /// - If `selection_start <= cursor_pos`: Returns `(selection_start, cursor_pos)`
    /// - If `selection_start > cursor_pos`: Returns `(cursor_pos, selection_start)`
    ///
    /// # Character Indexing
    ///
    /// The returned indices are character positions (not byte positions),
    /// ensuring proper handling of multi-byte Unicode characters.
    ///
    /// # Usage
    ///
    /// Used internally by:
    /// - `get_selected_text()`: To extract the text substring
    /// - `delete_selection()`: To determine which text to delete
    /// - UI rendering: To highlight the selected portion
    pub fn get_selection_range(&self) -> Option<(usize, usize)> {
        self.selection_start.map(|start| {
            if start <= self.cursor_pos {
                (start, self.cursor_pos)
            } else {
                (self.cursor_pos, start)
            }
        })
    }

    /// Returns the selected text, or None if no selection exists.
    ///
    /// Extracts the substring between the selection start and end positions,
    /// properly handling Unicode characters by operating on character indices
    /// rather than byte positions.
    ///
    /// # Returns
    ///
    /// - `Some(String)`: The selected text if an active selection exists
    /// - `None`: If no selection is active or selection is empty
    ///
    /// # Character Safety
    ///
    /// This method operates on Unicode characters, not bytes:
    /// - Multi-byte characters (e.g., emojis, accented letters) are handled correctly
    /// - Selection boundaries respect character boundaries
    /// - No risk of splitting multi-byte characters
    ///
    /// # Usage
    ///
    /// Used by:
    /// - `copy_selection()`: To get the text to copy to clipboard
    /// - UI rendering: To display selection information
    /// - Testing: To verify selection behavior
    pub fn get_selected_text(&self) -> Option<String> {
        self.get_selection_range().map(|(start, end)| {
            let chars: Vec<char> = self.input.chars().collect();
            chars[start..end].iter().collect()
        })
    }

    /// Deletes the currently selected text and clears the selection.
    ///
    /// This internal method removes the text within the active selection range,
    /// repositions the cursor to the start of the removed text, and clears the
    /// selection state. The operation is performed safely with respect to
    /// character boundaries.
    ///
    /// # Behavior
    ///
    /// 1. Calculates normalized selection range (start <= end)
    /// 2. Converts character indices to byte indices
    /// 3. Removes the substring using `replace_range`
    /// 4. Positions cursor at the start of the deleted range
    /// 5. Clears the selection state
    /// 6. Resets cursor blink timer
    ///
    /// # Character Safety
    ///
    /// - Uses character-based indices internally
    /// - Converts to byte indices only for the actual deletion
    /// - Preserves multi-byte character integrity
    ///
    /// # Side Effects
    ///
    /// - Modifies `self.input` by removing text
    /// - Updates `self.cursor_pos`
    /// - Clears `self.selection_start`
    /// - Resets cursor blink state
    ///
    /// # Usage
    ///
    /// Called automatically by:
    /// - `insert_char()`: Before inserting new text
    /// - `insert_text_at_cursor()`: Before pasting
    fn delete_selection(&mut self) {
        if let Some((start, end)) = self.get_selection_range() {
            let start_byte = self.char_index_to_byte_index(start);
            let end_byte = self.char_index_to_byte_index(end);
            self.input.replace_range(start_byte..end_byte, "");
            self.cursor_pos = start;
            self.clear_selection();
            self.reset_cursor_blink();
        }
    }

    /// Inserts text at the cursor position, replacing any selected text.
    ///
    /// This method provides smart text insertion behavior: if a selection is
    /// active, it first deletes the selected text, then inserts the new text
    /// at the cursor position. This matches standard text editor behavior.
    ///
    /// # Arguments
    ///
    /// * `text` - The string to insert at the cursor position
    ///
    /// # Behavior
    ///
    /// 1. If selection exists: Deletes selected text first
    /// 2. Inserts the provided text at the current cursor position
    /// 3. Advances cursor by the number of characters inserted
    /// 4. Resets cursor blink timer to keep cursor visible
    ///
    /// # Character Handling
    ///
    /// - Properly counts Unicode characters (not bytes)
    /// - Cursor advances by character count, not byte length
    /// - Supports multi-byte characters and emojis
    ///
    /// # Use Cases
    ///
    /// - Pasting text from clipboard (`paste_from_clipboard`)
    /// - Inserting multi-character strings
    /// - Replacing selected text with new content
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Replace selected "world" with "Rust"
    /// app.insert_text_at_cursor("Rust"); // "hello Rust" instead of "hello world"
    /// ```
    pub fn insert_text_at_cursor(&mut self, text: &str) {
        // Delete selection first if it exists
        if self.selection_start.is_some() {
            self.delete_selection();
        }
        
        // Insert text at cursor
        let byte_idx = self.char_index_to_byte_index(self.cursor_pos);
        self.input.insert_str(byte_idx, text);
        self.cursor_pos += text.chars().count();
        self.reset_cursor_blink();
    }

    /// Moves the cursor one character to the left while extending the selection.
    ///
    /// This method implements the Shift+Left keyboard shortcut behavior,
    /// allowing users to select text character by character in the backward
    /// direction. If no selection exists, it creates one starting from the
    /// current cursor position.
    ///
    /// # Behavior
    ///
    /// 1. Starts a new selection if none exists (anchors at current position)
    /// 2. Moves cursor one character to the left
    /// 3. Extends or contracts the selection based on direction
    /// 4. Resets cursor blink timer for visibility
    ///
    /// # Selection Dynamics
    ///
    /// - **No selection**: Creates selection from cursor position
    /// - **Expanding left**: Increases selection size
    /// - **Collapsing right**: Decreases selection size
    /// - **At start**: No-op if cursor is at position 0
    ///
    /// # Keyboard Mapping
    ///
    /// - Primary: `Shift + Left Arrow`
    pub fn move_cursor_left_with_selection(&mut self) {
        self.start_selection();
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            self.reset_cursor_blink();
        }
    }

    /// Moves the cursor one character to the right while extending the selection.
    ///
    /// This method implements the Shift+Right keyboard shortcut behavior,
    /// allowing users to select text character by character in the forward
    /// direction. If no selection exists, it creates one starting from the
    /// current cursor position.
    ///
    /// # Behavior
    ///
    /// 1. Starts a new selection if none exists (anchors at current position)
    /// 2. Moves cursor one character to the right
    /// 3. Extends or contracts the selection based on direction
    /// 4. Resets cursor blink timer for visibility
    ///
    /// # Selection Dynamics
    ///
    /// - **No selection**: Creates selection from cursor position
    /// - **Expanding right**: Increases selection size
    /// - **Collapsing left**: Decreases selection size
    /// - **At end**: No-op if cursor is at last position
    ///
    /// # Keyboard Mapping
    ///
    /// - Primary: `Shift + Right Arrow`
    pub fn move_cursor_right_with_selection(&mut self) {
        self.start_selection();
        let len = self.input.chars().count();
        if self.cursor_pos < len {
            self.cursor_pos += 1;
            self.reset_cursor_blink();
        }
    }

    /// Moves the cursor one word to the left while extending the selection.
    ///
    /// This method implements the Ctrl+Shift+Left keyboard shortcut, enabling
    /// users to select text word by word in the backward direction. It uses
    /// the same word boundary detection as `move_cursor_word_left`, treating
    /// alphanumeric characters and underscores as word constituents.
    ///
    /// # Behavior
    ///
    /// 1. Starts a new selection if none exists
    /// 2. Skips over non-word characters (spaces, punctuation)
    /// 3. Jumps to the beginning of the previous word
    /// 4. Extends or contracts selection accordingly
    ///
    /// # Word Boundaries
    ///
    /// Uses `is_word_char` for detection:
    /// - **Word characters**: Letters, digits, underscore
    /// - **Separators**: Spaces, punctuation, special characters
    ///
    /// # Edge Cases
    ///
    /// - At start of input: No-op
    /// - Multiple separators: Skips all before landing on word
    ///
    /// # Keyboard Mapping
    ///
    /// - Primary: `Ctrl + Shift + Left Arrow`
    pub fn move_cursor_word_left_with_selection(&mut self) {
        self.start_selection();
        if self.cursor_pos == 0 {
            return;
        }
        let chars: Vec<char> = self.input.chars().collect();
        let mut i = self.cursor_pos.min(chars.len());

        while i > 0 && !Self::is_word_char(chars[i - 1]) {
            i -= 1;
        }
        while i > 0 && Self::is_word_char(chars[i - 1]) {
            i -= 1;
        }

        if i != self.cursor_pos {
            self.cursor_pos = i;
            self.reset_cursor_blink();
        }
    }

    /// Moves the cursor one word to the right while extending the selection.
    ///
    /// This method implements the Ctrl+Shift+Right keyboard shortcut, enabling
    /// users to select text word by word in the forward direction. It uses
    /// the same word boundary detection as `move_cursor_word_right`, treating
    /// alphanumeric characters and underscores as word constituents.
    ///
    /// # Behavior
    ///
    /// 1. Starts a new selection if none exists
    /// 2. Skips over non-word characters (spaces, punctuation)
    /// 3. Jumps to the end of the next word
    /// 4. Extends or contracts selection accordingly
    ///
    /// # Word Boundaries
    ///
    /// Uses `is_word_char` for detection:
    /// - **Word characters**: Letters, digits, underscore
    /// - **Separators**: Spaces, punctuation, special characters
    ///
    /// # Edge Cases
    ///
    /// - At end of input: No-op
    /// - Multiple separators: Skips all before landing on word
    ///
    /// # Keyboard Mapping
    ///
    /// - Primary: `Ctrl + Shift + Right Arrow`
    pub fn move_cursor_word_right_with_selection(&mut self) {
        self.start_selection();
        let chars: Vec<char> = self.input.chars().collect();
        let len = chars.len();
        let mut i = self.cursor_pos.min(len);

        while i < len && !Self::is_word_char(chars[i]) {
            i += 1;
        }
        while i < len && Self::is_word_char(chars[i]) {
            i += 1;
        }

        if i != self.cursor_pos {
            self.cursor_pos = i;
            self.reset_cursor_blink();
        }
    }

    /// Moves the cursor to the start of the input while extending the selection.
    ///
    /// This method implements the Shift+Home keyboard shortcut, allowing users
    /// to select all text from the current cursor position to the beginning of
    /// the input field in a single action.
    ///
    /// # Behavior
    ///
    /// 1. Starts a new selection if none exists
    /// 2. Moves cursor to position 0 (start of input)
    /// 3. Selection spans from original position to start
    /// 4. Resets cursor blink timer if position changes
    ///
    /// # Use Cases
    ///
    /// - Quick selection of text from cursor to start
    /// - Useful for replacing beginning of input
    /// - Efficient bulk text selection
    ///
    /// # Keyboard Mapping
    ///
    /// - Primary: `Shift + Home`
    /// - Alternative: `Shift + Ctrl + Home` (also supported)
    pub fn move_cursor_home_with_selection(&mut self) {
        self.start_selection();
        if self.cursor_pos != 0 {
            self.cursor_pos = 0;
            self.reset_cursor_blink();
        }
    }

    /// Moves the cursor to the end of the input while extending the selection.
    ///
    /// This method implements the Shift+End keyboard shortcut, allowing users
    /// to select all text from the current cursor position to the end of the
    /// input field in a single action.
    ///
    /// # Behavior
    ///
    /// 1. Starts a new selection if none exists
    /// 2. Moves cursor to end of input (last character position)
    /// 3. Selection spans from original position to end
    /// 4. Resets cursor blink timer if position changes
    ///
    /// # Use Cases
    ///
    /// - Quick selection of text from cursor to end
    /// - Useful for replacing end of input
    /// - Efficient bulk text selection
    ///
    /// # Keyboard Mapping
    ///
    /// - Primary: `Shift + End`
    /// - Alternative: `Shift + Ctrl + End` (also supported)
    pub fn move_cursor_end_with_selection(&mut self) {
        self.start_selection();
        let len = self.input.chars().count();
        if self.cursor_pos != len {
            self.cursor_pos = len;
            self.reset_cursor_blink();
        }
    }

    /// Copies the currently selected text to the system clipboard.
    ///
    /// This method implements the copy-to-clipboard functionality, allowing
    /// users to copy selected text for use in other applications or for pasting
    /// later within LazyLlama. It uses the `arboard` crate for cross-platform
    /// clipboard access.
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Text successfully copied to clipboard
    /// - `Err`: If no text is selected or clipboard access fails
    ///
    /// # Error Cases
    ///
    /// - **No selection**: Returns error "No text selected"
    /// - **Clipboard unavailable**: System clipboard access denied
    /// - **Platform issues**: OS-specific clipboard errors
    ///
    /// # Behavior
    ///
    /// 1. Checks if selection exists via `get_selected_text()`
    /// 2. Creates new clipboard instance
    /// 3. Writes selected text to clipboard
    /// 4. Maintains selection state (does not clear)
    ///
    /// # Platform Support
    ///
    /// - **Windows**: Uses Win32 clipboard API
    /// - **Linux**: Uses X11/Wayland clipboard
    /// - **macOS**: Uses NSPasteboard
    ///
    /// # Keyboard Mapping
    ///
    /// - Primary: `Ctrl + Shift + C`
    ///
    /// # Usage
    ///
    /// ```ignore
    /// if let Err(e) = app.copy_selection() {
    ///     // Handle error (e.g., no selection)
    /// }
    /// ```
    pub fn copy_selection(&self) -> Result<()> {
        if let Some(text) = self.get_selected_text() {
            let mut clipboard = arboard::Clipboard::new()?;
            clipboard.set_text(text)?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("No text selected"))
        }
    }

    /// Pastes text from the system clipboard at the cursor position.
    ///
    /// This method implements paste-from-clipboard functionality, retrieving
    /// text from the system clipboard and inserting it at the current cursor
    /// position. If a selection is active, the selected text is replaced by
    /// the pasted content, matching standard text editor behavior.
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Text successfully pasted from clipboard
    /// - `Err`: If clipboard access fails or clipboard is empty
    ///
    /// # Error Cases
    ///
    /// - **Empty clipboard**: No text content available
    /// - **Clipboard unavailable**: System clipboard access denied
    /// - **Platform issues**: OS-specific clipboard errors
    ///
    /// # Behavior
    ///
    /// 1. Creates new clipboard instance
    /// 2. Retrieves text from clipboard
    /// 3. Calls `insert_text_at_cursor()` to insert text
    /// 4. Automatically replaces selection if active
    /// 5. Advances cursor to end of pasted text
    ///
    /// # Selection Handling
    ///
    /// - **With selection**: Replaces selected text with clipboard content
    /// - **Without selection**: Inserts at cursor position
    ///
    /// # Platform Support
    ///
    /// - **Windows**: Uses Win32 clipboard API
    /// - **Linux**: Uses X11/Wayland clipboard
    /// - **macOS**: Uses NSPasteboard
    ///
    /// # Keyboard Mapping
    ///
    /// - Primary: `Ctrl + Shift + V`
    ///
    /// # Usage
    ///
    /// ```ignore
    /// if let Err(e) = app.paste_from_clipboard() {
    ///     // Handle error (e.g., clipboard empty)
    /// }
    /// ```
    pub fn paste_from_clipboard(&mut self) -> Result<()> {
        let mut clipboard = arboard::Clipboard::new()?;
        let text = clipboard.get_text()?;
        self.insert_text_at_cursor(&text);
        Ok(())
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
        if self.models.is_empty() {
            return;
        }
        
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
        if self.models.is_empty() {
            return;
        }
        
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
            self.cursor_pos = 0;
            
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


