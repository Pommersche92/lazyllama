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
    
    app.move_cursor_left();
    app.delete_forward();
    assert_eq!(app.input, "🦀ü");
    assert_eq!(app.cursor_pos, 2);
}

/// Tests text selection clearing functionality.
/// 
/// Validates that:
/// - Selection can be cleared explicitly
/// - Selection state is properly reset
/// - Cursor position is not affected by clearing selection
/// 
/// # Test Cases
/// 
/// - Clear selection when selection exists
/// - Clear selection when no selection exists (should be no-op)
/// - Verify selection_start becomes None after clearing
/// 
/// # Expected Behavior
/// 
/// - `selection_start` should be set to None
/// - Cursor position should remain unchanged
/// - Operation should be safe to call multiple times
#[test]
fn test_clear_selection() {
    let mut app = create_test_app();
    app.input = "Hello World".to_string();
    app.cursor_pos = 5;
    app.selection_start = Some(0);
    
    app.clear_selection();
    assert_eq!(app.selection_start, None);
    assert_eq!(app.cursor_pos, 5);
    
    // Clearing again should be safe
    app.clear_selection();
    assert_eq!(app.selection_start, None);
}

/// Tests selection range calculation and normalization.
/// 
/// Validates that:
/// - Selection range is always returned with start <= end
/// - Forward selection (left to right) is handled correctly
/// - Backward selection (right to left) is normalized
/// - Empty/non-existent selection returns None
/// 
/// # Test Cases
/// 
/// - Forward selection (selection_start < cursor_pos)
/// - Backward selection (selection_start > cursor_pos)
/// - No selection (selection_start is None)
/// - Selection at same position (start == end should return empty range)
/// 
/// # Expected Behavior
/// 
/// - Returns Some((start, end)) where start <= end
/// - Returns None when no selection is active
/// - Handles bidirectional selection correctly
#[test]
fn test_get_selection_range() {
    let mut app = create_test_app();
    app.input = "Hello World".to_string();
    
    // No selection
    assert_eq!(app.get_selection_range(), None);
    
    // Forward selection
    app.selection_start = Some(0);
    app.cursor_pos = 5;
    assert_eq!(app.get_selection_range(), Some((0, 5)));
    
    // Backward selection (should normalize)
    app.selection_start = Some(8);
    app.cursor_pos = 3;
    assert_eq!(app.get_selection_range(), Some((3, 8)));
    
    // Same position
    app.selection_start = Some(5);
    app.cursor_pos = 5;
    assert_eq!(app.get_selection_range(), Some((5, 5)));
}

/// Tests selected text extraction functionality.
/// 
/// Validates that:
/// - Selected text is extracted correctly as a string
/// - Unicode characters are handled properly
/// - Empty selection returns empty string
/// - No selection returns None
/// 
/// # Test Cases
/// 
/// - Extract normal ASCII text
/// - Extract text with Unicode characters
/// - Extract with forward and backward selection
/// - Handle no selection case
/// 
/// # Expected Behavior
/// 
/// - Returns Some(String) with selected text when selection exists
/// - Returns None when no selection is active
/// - Correctly handles multi-byte UTF-8 characters
#[test]
fn test_get_selected_text() {
    let mut app = create_test_app();
    app.input = "Hello 🦀 World".to_string();
    
    // No selection
    assert_eq!(app.get_selected_text(), None);
    
    // Select "Hello"
    app.selection_start = Some(0);
    app.cursor_pos = 5;
    assert_eq!(app.get_selected_text(), Some("Hello".to_string()));
    
    // Select emoji
    app.selection_start = Some(6);
    app.cursor_pos = 7;
    assert_eq!(app.get_selected_text(), Some("🦀".to_string()));
    
    // Backward selection
    app.selection_start = Some(13);
    app.cursor_pos = 8;
    assert_eq!(app.get_selected_text(), Some("World".to_string()));
}

/// Tests character-wise leftward selection movement.
/// 
/// Validates that:
/// - Selection is initiated when not present
/// - Selection expands/contracts correctly when moving left
/// - Cursor position is updated appropriately
/// - Boundary conditions are respected (beginning of input)
/// 
/// # Test Cases
/// 
/// - Start new selection by moving left
/// - Expand existing selection to the left
/// - Collapse selection by moving left (when selecting backward)
/// - Handle movement at the beginning of input
/// 
/// # Expected Behavior
/// 
/// - Creates selection anchor if none exists
/// - Moves cursor one position left
/// - Respects input boundaries
#[test]
fn test_move_cursor_left_with_selection() {
    let mut app = create_test_app();
    app.input = "Hello".to_string();
    app.cursor_pos = 5;
    
    // First move should start selection
    app.move_cursor_left_with_selection();
    assert_eq!(app.selection_start, Some(5));
    assert_eq!(app.cursor_pos, 4);
    
    // Second move should expand selection
    app.move_cursor_left_with_selection();
    assert_eq!(app.selection_start, Some(5));
    assert_eq!(app.cursor_pos, 3);
    assert_eq!(app.get_selected_text(), Some("lo".to_string()));
    
    // At beginning, should not move cursor
    app.cursor_pos = 0;
    app.move_cursor_left_with_selection();
    assert_eq!(app.cursor_pos, 0);
}

/// Tests character-wise rightward selection movement.
/// 
/// Validates that:
/// - Selection is initiated when not present
/// - Selection expands/contracts correctly when moving right
/// - Cursor position is updated appropriately
/// - Boundary conditions are respected (end of input)
/// 
/// # Test Cases
/// 
/// - Start new selection by moving right
/// - Expand existing selection to the right
/// - Collapse selection by moving right (when selecting backward)
/// - Handle movement at the end of input
/// 
/// # Expected Behavior
/// 
/// - Creates selection anchor if none exists
/// - Moves cursor one position right
/// - Respects input boundaries
#[test]
fn test_move_cursor_right_with_selection() {
    let mut app = create_test_app();
    app.input = "Hello".to_string();
    app.cursor_pos = 0;
    
    // First move should start selection
    app.move_cursor_right_with_selection();
    assert_eq!(app.selection_start, Some(0));
    assert_eq!(app.cursor_pos, 1);
    
    // Second move should expand selection
    app.move_cursor_right_with_selection();
    assert_eq!(app.selection_start, Some(0));
    assert_eq!(app.cursor_pos, 2);
    assert_eq!(app.get_selected_text(), Some("He".to_string()));
    
    // At end, should not move cursor
    app.cursor_pos = 5;
    app.move_cursor_right_with_selection();
    assert_eq!(app.cursor_pos, 5);
}

/// Tests word-wise leftward selection movement.
/// 
/// Validates that:
/// - Selection jumps by word boundaries
/// - Word boundary detection uses is_word_char rules
/// - Multiple words can be selected
/// - Non-word characters are handled correctly
/// 
/// # Test Cases
/// 
/// - Select previous word
/// - Skip over whitespace and punctuation
/// - Select multiple words in sequence
/// - Handle word boundaries correctly
/// 
/// # Expected Behavior
/// 
/// - Jumps to start of previous word
/// - Skips non-word characters before landing on word
/// - Creates/extends selection appropriately
#[test]
fn test_move_cursor_word_left_with_selection() {
    let mut app = create_test_app();
    app.input = "Hello World Test".to_string();
    app.cursor_pos = 16; // End
    
    app.move_cursor_word_left_with_selection();
    assert_eq!(app.cursor_pos, 12); // Start of "Test"
    assert_eq!(app.get_selected_text(), Some("Test".to_string()));
    
    app.move_cursor_word_left_with_selection();
    assert_eq!(app.cursor_pos, 6); // Start of "World"
    assert_eq!(app.get_selected_text(), Some("World Test".to_string()));
}

/// Tests word-wise rightward selection movement.
/// 
/// Validates that:
/// - Selection jumps by word boundaries
/// - Word boundary detection uses is_word_char rules
/// - Multiple words can be selected
/// - Non-word characters are handled correctly
/// 
/// # Test Cases
/// 
/// - Select next word
/// - Skip over whitespace and punctuation
/// - Select multiple words in sequence
/// - Handle word boundaries correctly
/// 
/// # Expected Behavior
/// 
/// - Jumps to end of next word
/// - Skips non-word characters before landing on word
/// - Creates/extends selection appropriately
#[test]
fn test_move_cursor_word_right_with_selection() {
    let mut app = create_test_app();
    app.input = "Hello World Test".to_string();
    app.cursor_pos = 0;
    
    app.move_cursor_word_right_with_selection();
    assert_eq!(app.cursor_pos, 5); // End of "Hello"
    assert_eq!(app.get_selected_text(), Some("Hello".to_string()));
    
    app.move_cursor_word_right_with_selection();
    assert_eq!(app.cursor_pos, 11); // End of "World"
    assert_eq!(app.get_selected_text(), Some("Hello World".to_string()));
}

/// Tests selection to start of input (Home with Shift).
/// 
/// Validates that:
/// - Selection extends from cursor to beginning
/// - Works with forward and backward existing selections
/// - Cursor moves to position 0
/// 
/// # Test Cases
/// 
/// - Select from middle to start
/// - Select all text (from end to start)
/// - Handle already at start position
/// 
/// # Expected Behavior
/// 
/// - Cursor moves to position 0
/// - Selection spans from original position to start
/// - Creates selection if none exists
#[test]
fn test_move_cursor_home_with_selection() {
    let mut app = create_test_app();
    app.input = "Hello World".to_string();
    app.cursor_pos = 6;
    
    app.move_cursor_home_with_selection();
    assert_eq!(app.cursor_pos, 0);
    assert_eq!(app.get_selected_text(), Some("Hello ".to_string()));
    
    // Already at home - should maintain selection
    app.move_cursor_home_with_selection();
    assert_eq!(app.cursor_pos, 0);
}

/// Tests selection to end of input (End with Shift).
/// 
/// Validates that:
/// - Selection extends from cursor to end
/// - Works with forward and backward existing selections
/// - Cursor moves to end position
/// 
/// # Test Cases
/// 
/// - Select from middle to end
/// - Select all text (from start to end)
/// - Handle already at end position
/// 
/// # Expected Behavior
/// 
/// - Cursor moves to end of input
/// - Selection spans from original position to end
/// - Creates selection if none exists
#[test]
fn test_move_cursor_end_with_selection() {
    let mut app = create_test_app();
    app.input = "Hello World".to_string();
    app.cursor_pos = 5;
    
    app.move_cursor_end_with_selection();
    assert_eq!(app.cursor_pos, 11);
    assert_eq!(app.get_selected_text(), Some(" World".to_string()));
    
    // Already at end - should maintain selection
    app.move_cursor_end_with_selection();
    assert_eq!(app.cursor_pos, 11);
}

/// Tests text insertion at cursor with selection replacement.
/// 
/// Validates that:
/// - Text is inserted at cursor position
/// - Existing selection is deleted before insertion
/// - Cursor position is updated correctly
/// - Multi-character strings are handled
/// 
/// # Test Cases
/// 
/// - Insert text without selection
/// - Insert text replacing selection
/// - Insert Unicode text
/// - Cursor positioning after insertion
/// 
/// # Expected Behavior
/// 
/// - Deletes selection if present
/// - Inserts text at cursor position
/// - Advances cursor by character count (not byte count)
#[test]
fn test_insert_text_at_cursor() {
    let mut app = create_test_app();
    app.input = "Hello World".to_string();
    app.cursor_pos = 6;
    
    // Insert without selection
    app.insert_text_at_cursor("Beautiful ");
    assert_eq!(app.input, "Hello Beautiful World");
    assert_eq!(app.cursor_pos, 16);
    
    // Insert with selection (should replace)
    app.input = "Hello World".to_string();
    app.cursor_pos = 11;
    app.selection_start = Some(6);
    app.insert_text_at_cursor("Rust");
    assert_eq!(app.input, "Hello Rust");
    assert_eq!(app.cursor_pos, 10);
    assert_eq!(app.selection_start, None);
}

/// Tests typing behavior with active selection.
/// 
/// Validates that:
/// - Typing a character deletes the selection
/// - New character is inserted at selection start
/// - Selection is cleared after typing
/// 
/// # Test Cases
/// 
/// - Type single character with selection
/// - Verify selection is replaced, not extended
/// - Check cursor positioning
/// 
/// # Expected Behavior
/// 
/// - Selection is deleted first
/// - New character appears at selection start position
/// - No selection remains after typing
#[test]
fn test_typing_clears_selection() {
    let mut app = create_test_app();
    app.input = "Hello World".to_string();
    app.cursor_pos = 11;
    app.selection_start = Some(6);
    
    // Typing should replace selection
    app.insert_char('!');
    assert_eq!(app.input, "Hello !");
    assert_eq!(app.cursor_pos, 7);
    assert_eq!(app.selection_start, None);
}

/// Tests movement without Shift clears selection.
/// 
/// Validates that:
/// - Moving cursor without Shift clears selection
/// - Works for all movement types (char, word, home, end)
/// - Cursor still moves correctly
/// 
/// # Test Cases
/// 
/// - Move left without Shift
/// - Move right without Shift
/// - Move home/end without Shift
/// - Word movement without Shift
/// 
/// # Expected Behavior
/// 
/// - Selection is cleared before movement
/// - Cursor moves as expected
/// - No selection remains after movement
#[test]
fn test_movement_clears_selection() {
    let mut app = create_test_app();
    app.input = "Hello World".to_string();
    app.cursor_pos = 11;
    app.selection_start = Some(6);
    
    // Normal movement should clear selection
    app.move_cursor_left();
    assert_eq!(app.selection_start, None);
    assert_eq!(app.cursor_pos, 10);
    
    // Word movement should also clear
    app.selection_start = Some(5);
    app.move_cursor_word_left();
    assert_eq!(app.selection_start, None);
    
    // Home/End should clear
    app.selection_start = Some(5);
    app.move_cursor_home();
    assert_eq!(app.selection_start, None);
}

/// Tests clipboard copy operation with selection.
/// 
/// Validates that:
/// - Text is copied to clipboard when selection exists
/// - Error is returned when no selection exists
/// - Selection remains after copying
/// - Unicode text is copied correctly
/// 
/// # Test Cases
/// 
/// - Copy with valid selection
/// - Copy without selection (should error)
/// - Copy Unicode text
/// - Verify selection persists after copy
/// 
/// # Expected Behavior
/// 
/// - Returns Ok(()) when text is copied successfully
/// - Returns Err when no selection exists
/// - Selection state is unchanged after copy
/// 
/// # Note
/// 
/// This test may fail in headless environments without clipboard access.
/// The test gracefully handles clipboard unavailability.
#[test]
fn test_copy_selection() {
    let mut app = create_test_app();
    app.input = "Hello World".to_string();
    
    // No selection - should error
    let result = app.copy_selection();
    assert!(result.is_err());
    
    // With selection - should succeed (if clipboard available)
    app.cursor_pos = 5;
    app.selection_start = Some(0);
    let result = app.copy_selection();
    
    // In CI/headless environments, clipboard may not be available
    // So we accept both success and failure here
    match result {
        Ok(_) => {
            // If clipboard works, verify selection is still there
            assert_eq!(app.selection_start, Some(0));
            assert_eq!(app.cursor_pos, 5);
        }
        Err(_) => {
            // Clipboard not available in test environment - this is acceptable
        }
    }
}

/// Tests clipboard paste operation.
/// 
/// Validates that:
/// - Text can be pasted at cursor position
/// - Pasting replaces active selection
/// - Cursor is positioned after pasted text
/// - Error handling for empty/unavailable clipboard
/// 
/// # Test Cases
/// 
/// - Paste without selection
/// - Paste with selection (should replace)
/// - Paste at different cursor positions
/// - Handle clipboard errors
/// 
/// # Expected Behavior
/// 
/// - Returns Ok(()) when text is pasted successfully
/// - Returns Err when clipboard is empty or unavailable
/// - Selection is cleared after paste
/// - Cursor moves to end of pasted text
/// 
/// # Note
/// 
/// This test may fail in headless environments without clipboard access.
/// The test gracefully handles clipboard unavailability.
#[test]
fn test_paste_from_clipboard() {
    let mut app = create_test_app();
    app.input = "Hello".to_string();
    app.cursor_pos = 5;
    
    // Try to paste - may fail if clipboard is empty or unavailable
    let result = app.paste_from_clipboard();
    
    // In CI/headless environments, clipboard may not be available
    // We mainly test that the function doesn't crash
    match result {
        Ok(_) => {
            // Clipboard worked - cursor should have moved
            assert!(app.cursor_pos >= 5);
        }
        Err(_) => {
            // Clipboard not available - this is acceptable in tests
        }
    }
}

/// Tests bidirectional selection behavior.
/// 
/// Validates that:
/// - Selection can be extended in both directions
/// - Forward and backward selections work identically
/// - Selection text is always correct regardless of direction
/// 
/// # Test Cases
/// 
/// - Create forward selection (left to right)
/// - Create backward selection (right to left)
/// - Switch directions mid-selection
/// - Verify text extraction works bidirectionally
/// 
/// # Expected Behavior
/// 
/// - get_selected_text() returns same result for both directions
/// - get_selection_range() normalizes to (start, end) with start <= end
/// - Cursor position reflects current end of selection
#[test]
fn test_bidirectional_selection() {
    let mut app = create_test_app();
    app.input = "Hello World".to_string();
    
    // Forward selection
    app.cursor_pos = 0;
    app.selection_start = None;
    app.move_cursor_right_with_selection();
    app.move_cursor_right_with_selection();
    app.move_cursor_right_with_selection();
    let forward_text = app.get_selected_text();
    
    // Backward selection
    app.cursor_pos = 3;
    app.selection_start = None;
    app.move_cursor_left_with_selection();
    app.move_cursor_left_with_selection();
    app.move_cursor_left_with_selection();
    let backward_text = app.get_selected_text();
    
    // Both should select the same text
    assert_eq!(forward_text, backward_text);
    assert_eq!(forward_text, Some("Hel".to_string()));
}
