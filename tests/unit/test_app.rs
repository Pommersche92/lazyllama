//! Unit tests for the App module (src/app.rs)
//! 
//! These tests verify the core functionality of the App structure,
//! including text input, cursor navigation, model management,
//! and buffer administration.
//!
//! ## Test Coverage
//!
//! - **Text Input Operations**: Character insertion, deletion, cursor movement
//! - **Unicode Handling**: Proper handling of multi-byte characters
//! - **Model Management**: Model switching, buffer isolation, state persistence
//! - **Cursor Navigation**: Word-wise movement, boundary detection, position tracking
//! - **Input Validation**: Edge case handling, boundary conditions
//!
//! ## Test Strategy
//!
//! - Uses mock App instances to avoid external dependencies
//! - Tests individual operations in isolation
//! - Validates state consistency after operations
//! - Ensures proper handling of edge cases and boundary conditions

use std::collections::HashMap;
use std::time::{Duration, Instant};
use ratatui::widgets::ListState;
use ollama_rs::Ollama;
use lazyllama::app::App;


/// Creates a test App instance without Ollama API calls
/// 
/// This helper function creates a minimal App instance suitable for unit testing
/// without requiring external dependencies or network access.
/// 
/// # Returns
/// 
/// A fully initialized App instance with:
/// - Two test models in the model list
/// - Default selected model (index 0)
/// - Empty input and history buffers
/// - Default cursor and scroll positions
/// - Mock Ollama client (no network calls)
/// - Current timestamp for timing-sensitive operations
/// 
/// # Usage
/// 
/// ```ignore
/// let mut app = create_test_app();
/// app.insert_char('H');
/// assert_eq!(app.input, "H");
/// ```
fn create_test_app() -> App {
    App {
        models: vec!["test_model_1".to_string(), "test_model_2".to_string()],
        list_state: {
            let mut state = ListState::default();
            state.select(Some(0));
            state
        },
        input: String::new(),
        cursor_pos: 0,
        history: String::new(),
        model_inputs: HashMap::new(),
        model_cursors: HashMap::new(),
        model_histories: HashMap::new(),
        model_scrolls: HashMap::new(),
        scroll: 0,
        autoscroll: true,
        is_loading: false,
        ollama: Ollama::default(),
        start_time: Instant::now(),
        last_cursor_blink: Instant::now(),
        cursor_visible: true,
        debug_keys: false,
        debug_last_key: None,
        render_count: 0,
    }
}

/// Tests character insertion functionality in the input buffer.
/// 
/// Validates that:
/// - ASCII characters are inserted correctly at cursor position
/// - Unicode characters (emojis, accented letters) are handled properly
/// - Cursor position is updated correctly after insertion
/// - String length and content integrity are maintained
/// 
/// # Test Cases
/// 
/// - Sequential ASCII character insertion
/// - Unicode character insertion (ö, emoji)
/// - Cursor position tracking after each insertion
/// 
/// # Expected Behavior
/// 
/// - Characters should appear at the correct position in the string
/// - Cursor should advance by one position per character
/// - Unicode characters should be treated as single units
#[test]
fn test_insert_char() {
    let mut app = create_test_app();
    
    app.insert_char('H');
    assert_eq!(app.input, "H");
    assert_eq!(app.cursor_pos, 1);
    
    app.insert_char('e');
    assert_eq!(app.input, "He");
    assert_eq!(app.cursor_pos, 2);
    
    // Test Unicode characters
    app.insert_char('ö');
    assert_eq!(app.input, "Heö");
    assert_eq!(app.cursor_pos, 3);
}

/// Tests backspace functionality for text deletion.
/// 
/// Validates that:
/// - Characters are removed correctly from the cursor position
/// - Cursor position is updated appropriately after deletion
/// - Boundary conditions are handled (beginning of string)
/// - String integrity is maintained after deletion
/// 
/// # Test Cases
/// 
/// - Normal backspace operation in middle of text
/// - Backspace at the beginning of text (should be no-op)
/// - Cursor position updates after deletion
/// 
/// # Expected Behavior
/// 
/// - Character before cursor should be removed
/// - Cursor should move back by one position
/// - No operation should occur when cursor is at position 0
#[test]
fn test_backspace() {
    let mut app = create_test_app();
    app.input = "Hello".to_string();
    app.cursor_pos = 5;
    
    app.backspace();
    assert_eq!(app.input, "Hell");
    assert_eq!(app.cursor_pos, 4);
    
    // Test at beginning of string
    app.cursor_pos = 0;
    app.backspace();
    assert_eq!(app.input, "Hell");
    assert_eq!(app.cursor_pos, 0);
}

/// Tests forward delete functionality.
/// 
/// Validates that:
/// - Character at cursor position is deleted correctly
/// - Cursor position remains unchanged after deletion
/// - Boundary conditions are handled (end of string)
/// - String content and length are updated properly
/// 
/// # Test Cases
/// 
/// - Forward deletion in middle of text
/// - Forward deletion at end of text (should be no-op)
/// - Cursor position stability during deletion
/// 
/// # Expected Behavior
/// 
/// - Character at cursor position should be removed
/// - Cursor position should remain the same
/// - No operation should occur when cursor is at end of string
#[test]
fn test_delete_forward() {
    let mut app = create_test_app();
    app.input = "Hello".to_string();
    app.cursor_pos = 2;
    
    app.delete_forward();
    assert_eq!(app.input, "Helo");
    assert_eq!(app.cursor_pos, 2);
    
    // Test at end of string
    app.cursor_pos = 4;
    app.delete_forward();
    assert_eq!(app.input, "Helo");
    assert_eq!(app.cursor_pos, 4);
}

/// Tests leftward cursor movement functionality.
/// 
/// Validates that:
/// - Cursor moves correctly to the left
/// - Boundary conditions are respected (beginning of text)
/// - Character positions are calculated accurately
/// - Unicode characters are handled as single units
/// 
/// # Test Cases
/// 
/// - Normal leftward movement in middle of text
/// - Movement at beginning of text (should be no-op)
/// - Position tracking and boundary detection
/// 
/// # Expected Behavior
/// 
/// - Cursor should move one position to the left
/// - Movement should stop at position 0
/// - Text content should remain unchanged
#[test]
fn test_move_cursor_left() {
    let mut app = create_test_app();
    app.input = "Test".to_string();
    app.cursor_pos = 2;
    
    app.move_cursor_left();
    assert_eq!(app.cursor_pos, 1);
    
    // Test at beginning
    app.cursor_pos = 0;
    app.move_cursor_left();
    assert_eq!(app.cursor_pos, 0);
}

#[test]
fn test_move_cursor_right() {
    let mut app = create_test_app();
    app.input = "Test".to_string();
    app.cursor_pos = 2;
    
    app.move_cursor_right();
    assert_eq!(app.cursor_pos, 3);
    
    // Test at end
    app.cursor_pos = 4;
    app.move_cursor_right();
    assert_eq!(app.cursor_pos, 4);
}

#[test]
fn test_move_cursor_home_end() {
    let mut app = create_test_app();
    app.input = "Hello World".to_string();
    app.cursor_pos = 5;
    
    app.move_cursor_home();
    assert_eq!(app.cursor_pos, 0);
    
    app.move_cursor_end();
    assert_eq!(app.cursor_pos, 11);
}

#[test]
fn test_word_navigation() {
    let mut app = create_test_app();
    app.input = "Hello World Test".to_string();
    app.cursor_pos = 16;
    
    // Test word left navigation
    app.move_cursor_word_left();
    assert_eq!(app.cursor_pos, 12); // Beginning of "Test"
    
    app.move_cursor_word_left();
    assert_eq!(app.cursor_pos, 6); // Beginning of "World"
    
    app.move_cursor_word_left();
    assert_eq!(app.cursor_pos, 0); // Beginning of "Hello"
    
    // Test word right navigation
    app.move_cursor_word_right();
    assert_eq!(app.cursor_pos, 5); // End of "Hello"
    
    app.move_cursor_word_right();
    assert_eq!(app.cursor_pos, 11); // End of "World"
    
    app.move_cursor_word_right();
    assert_eq!(app.cursor_pos, 16); // End of "Test"
}

#[test]
fn test_delete_word_left() {
    let mut app = create_test_app();
    app.input = "Hello World Test".to_string();
    app.cursor_pos = 16;
    
    app.delete_word_left();
    assert_eq!(app.input, "Hello World ");
    assert_eq!(app.cursor_pos, 12);
    
    app.delete_word_left();
    assert_eq!(app.input, "Hello ");
    assert_eq!(app.cursor_pos, 6);
}

#[test]
fn test_delete_word_right() {
    let mut app = create_test_app();
    app.input = "Hello World Test".to_string();
    app.cursor_pos = 0;
    
    app.delete_word_right();
    assert_eq!(app.input, " World Test");
    assert_eq!(app.cursor_pos, 0);
    
    app.delete_word_right();
    assert_eq!(app.input, " Test");
    assert_eq!(app.cursor_pos, 0);
}

#[test]
fn test_is_word_char() {
    assert!(App::is_word_char('a'));
    assert!(App::is_word_char('Z'));
    assert!(App::is_word_char('5'));
    assert!(App::is_word_char('_'));
    assert!(!App::is_word_char(' '));
    assert!(!App::is_word_char('.'));
    assert!(!App::is_word_char('-'));
}

#[test]
fn test_model_selection_next() {
    let mut app = create_test_app();
    app.models = vec!["model1".to_string(), "model2".to_string(), "model3".to_string()];
    app.list_state.select(Some(0));
    
    // Test normal progression
    app.select_next_model();
    assert_eq!(app.list_state.selected(), Some(1));
    
    app.select_next_model();
    assert_eq!(app.list_state.selected(), Some(2));
    
    // Test wraparound
    app.select_next_model();
    assert_eq!(app.list_state.selected(), Some(0));
}

#[test]
fn test_model_selection_previous() {
    let mut app = create_test_app();
    app.models = vec!["model1".to_string(), "model2".to_string(), "model3".to_string()];
    app.list_state.select(Some(2));
    
    // Test normal progression
    app.select_previous_model();
    assert_eq!(app.list_state.selected(), Some(1));
    
    app.select_previous_model();
    assert_eq!(app.list_state.selected(), Some(0));
    
    // Test wraparound
    app.select_previous_model();
    assert_eq!(app.list_state.selected(), Some(2));
}

#[test]
fn test_model_buffer_save_load() {
    let mut app = create_test_app();
    app.models = vec!["model1".to_string(), "model2".to_string()];
    app.list_state.select(Some(0));
    
    // Set some data for model1
    app.input = "Test input".to_string();
    app.cursor_pos = 5;
    app.history = "Test history".to_string();
    app.scroll = 10;
    
    // Save buffers for model1
    app.save_current_model_buffers();
    
    // Verify buffers are saved
    assert_eq!(app.model_inputs.get("model1"), Some(&"Test input".to_string()));
    assert_eq!(app.model_cursors.get("model1"), Some(&5));
    assert_eq!(app.model_histories.get("model1"), Some(&"Test history".to_string()));
    assert_eq!(app.model_scrolls.get("model1"), Some(&10));
    
    // Change to model2 and set different data
    app.list_state.select(Some(1));
    app.input = "Different input".to_string();
    app.cursor_pos = 8;
    app.history = "Different history".to_string();
    app.scroll = 5;
    
    // Load model1 buffers
    app.list_state.select(Some(0));
    app.load_current_model_buffers();
    
    // Verify model1 data is restored
    assert_eq!(app.input, "Test input");
    assert_eq!(app.cursor_pos, 5);
    assert_eq!(app.history, "Test history");
    assert_eq!(app.scroll, 10);
}

#[test]
fn test_cursor_blink_timing() {
    let mut app = create_test_app();
    
    // Initially cursor should be visible
    assert!(app.cursor_visible);
    
    // Should not update immediately
    assert!(!app.update_cursor_blink());
    assert!(app.cursor_visible);
    
    // Simulate time passage
    app.last_cursor_blink = Instant::now() - Duration::from_millis(600);
    assert!(app.update_cursor_blink());
    assert!(!app.cursor_visible);
    
    // Reset should make cursor visible again
    app.reset_cursor_blink();
    assert!(app.cursor_visible);
}

#[test]
fn test_char_index_to_byte_index() {
    let mut app = create_test_app();
    app.input = "Hëllö Wörld".to_string(); // Contains non-ASCII characters
    
    assert_eq!(app.char_index_to_byte_index(0), 0);    // 'H'
    assert_eq!(app.char_index_to_byte_index(1), 1);    // 'ë' starts at byte 1
    assert_eq!(app.char_index_to_byte_index(2), 3);    // 'l' starts at byte 3 (ë is 2 bytes)
    assert_eq!(app.char_index_to_byte_index(11), app.input.len()); // End of string
}

#[test]
fn test_cursor_clamp() {
    let mut app = create_test_app();
    app.input = "Test".to_string();
    app.cursor_pos = 10; // Beyond string end
    
    app.clamp_cursor();
    assert_eq!(app.cursor_pos, 4); // Should be clamped to string length
}

#[test]
fn test_empty_model_list_handling() {
    let mut app = create_test_app();
    app.models.clear();
    app.list_state.select(None); // Reset selection for empty list
    
    // Should not crash with empty model list
    app.select_next_model();
    app.select_previous_model();
    app.save_current_model_buffers();
    app.load_current_model_buffers();
    
    // Should handle the case gracefully
    assert_eq!(app.models.len(), 0);
}

#[test]
fn test_unicode_text_editing() {
    let mut app = create_test_app();
    
    app.insert_char('🦀'); // Rust crab emoji (4-byte UTF-8)
    app.insert_char('ü');  // Umlaut (2-byte UTF-8)
    app.insert_char('A');  // ASCII (1-byte)
    
    assert_eq!(app.input, "🦀üA");
    assert_eq!(app.cursor_pos, 3);
    
    app.backspace();
    assert_eq!(app.input, "🦀ü");
    assert_eq!(app.cursor_pos, 2);
    
    app.move_cursor_left();
    app.delete_forward();
    assert_eq!(app.input, "🦀");
    assert_eq!(app.cursor_pos, 1);
}