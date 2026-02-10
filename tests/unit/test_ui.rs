//! Unit tests for the UI module (src/ui.rs)
//! 
//! These tests verify the Terminal User Interface functionality,
//! including text parsing, history rendering, markdown processing,
//! and syntax highlighting.
//!
//! ## Test Coverage
//!
//! - **Conversation Parsing**: USER/AI label detection and formatting
//! - **Code Block Processing**: Markdown code block extraction and highlighting
//! - **Syntax Highlighting**: Language-specific code formatting
//! - **Text Styling**: Color application and text decoration
//! - **Edge Case Handling**: Malformed markdown, nested blocks, mixed content
//!
//! ## Test Strategy
//!
//! - Uses real UI parsing functions with mock data
//! - Validates output structure and formatting
//! - Tests visual consistency and readability
//! - Ensures robust handling of complex input scenarios

use ratatui::{
    style::{Color, Modifier, Style},
    text::Text,
};
use lazyllama::ui::{parse_history, process_styled_text, BANNER};

#[test]
fn test_parse_history_simple_conversation() {
    let history = "YOU: Hello\nAI: Hi there!";
    let parsed = parse_history(history);
    
    assert!(parsed.lines.len() >= 2);
    
    // Verify that USER/AI labels are parsed correctly
    let first_line = &parsed.lines[0];
    assert_eq!(first_line.spans.len(), 2); // "YOU:" + rest
    assert_eq!(first_line.spans[0].content, "YOU:");
    assert_eq!(first_line.spans[1].content, " Hello");
    
    let second_line = &parsed.lines[1];
    assert_eq!(second_line.spans.len(), 2); // "AI: " + rest  
    assert_eq!(second_line.spans[0].content, "AI: ");
    assert_eq!(second_line.spans[1].content, " Hi there!");
}

/// Tests conversation parsing with embedded code blocks.
/// 
/// This test validates the complex parsing logic for conversations that
/// contain markdown-style code blocks with syntax highlighting. Tests
/// the integration between conversation parsing and code block detection.
/// 
/// # Test Coverage
/// 
/// - Code block detection within conversations
/// - Language-specific syntax highlighting (rust)
/// - Code block frame rendering (borders, separators)
/// - Text styling for code vs regular content
/// - Multi-line code block handling
/// 
/// # Test Data
/// 
/// Conversation with embedded Rust code:
/// - USER request for code example
/// - AI response containing ```rust code block
/// - Function definition with println! macro
/// 
/// # Expected Behavior
/// 
/// - Code blocks should be framed with decorative borders
/// - Language identifier should be highlighted
/// - Code content should have appropriate styling
/// - Border characters should use consistent styling
#[test]
fn test_parse_history_with_code_block() {
    let history = "YOU: Show me code\nAI: Here's some code:\n\n```rust\nfn main() {\n    println!(\"Hello\");\n}\n```\n\nDone!";
    let parsed = parse_history(history);
    
    // Should have multiple lines including code block frames
    assert!(parsed.lines.len() > 5);
    
    // Find the code block header line
    let header_line = parsed.lines.iter()
        .find(|line| line.spans.iter().any(|span| span.content.contains("┌── rust")))
        .expect("Should find code block header");
    
    assert!(header_line.spans[0].content.contains("rust"));
    assert_eq!(header_line.spans[0].style.fg, Some(Color::Yellow));
    
    // Find code lines (with │ prefix)
    let code_lines: Vec<_> = parsed.lines.iter()
        .filter(|line| line.spans.iter().any(|span| span.content == " │ "))
        .collect();
    
    assert!(!code_lines.is_empty());
    for code_line in code_lines {
        assert_eq!(code_line.spans[0].content, " │ ");
        assert_eq!(code_line.spans[0].style.fg, Some(Color::Yellow));
    }
    
    // Find footer line
    let footer_line = parsed.lines.iter()
        .find(|line| line.spans.iter().any(|span| span.content.contains("└──────────")))
        .expect("Should find code block footer");
    
    assert!(footer_line.spans[0].content.contains("└"));
    assert_eq!(footer_line.spans[0].style.fg, Some(Color::Yellow));
}

/// Tests conversation parsing with multiple code blocks in sequence.
/// 
/// This test validates the parser's ability to handle conversations containing
/// multiple code blocks with different programming languages. Tests that each
/// code block is parsed independently with correct language detection.
/// 
/// # Test Coverage
/// 
/// - Multiple code block detection in single conversation
/// - Language-specific highlighting for different languages
/// - Code block separation and isolation
/// - Header generation for each language type
/// 
/// # Test Data
/// 
/// Conversation with two code blocks:
/// - Python code block with print statement
/// - JavaScript code block with console.log
/// 
/// # Expected Behavior
/// 
/// - Each code block should have its own header with language name
/// - Language detection should work for both python and javascript
/// - Code blocks should be visually separated
/// - Each block should have appropriate syntax highlighting
#[test]
fn test_parse_history_multiple_code_blocks() {
    let history = r#"USER: Show examples
AI: First example:

```python
print("Hello")
```

And a second one:

```javascript  
console.log("Hi");
```

That's it!"#;
    
    let parsed = parse_history(history);
    
    // Should find both code block headers
    let python_header = parsed.lines.iter()
        .find(|line| line.spans.iter().any(|span| span.content.contains("python")));
    assert!(python_header.is_some());
    
    let js_header = parsed.lines.iter()
        .find(|line| line.spans.iter().any(|span| span.content.contains("javascript")));
    assert!(js_header.is_some());
}

#[test]
fn test_parse_history_code_without_language() {
    let history = "AI: Code without language:\n\n```\necho \"hello\"\n```";
    let parsed = parse_history(history);
    
    // Sollte "code" als Standard-Sprache verwenden
    let header_line = parsed.lines.iter()
        .find(|line| line.spans.iter().any(|span| span.content.contains("┌── code")))
        .expect("Should find code block header with 'code'");
    
    assert!(header_line.spans[0].content.contains("code"));
}

#[test]
fn test_process_styled_text_headers() {
    let text = "### Header test\nRegular text";
    let mut result = Text::default();
    
    process_styled_text(text, &mut result);
    
    assert!(result.lines.len() >= 2);
    
    // Header sollte als Bullet Point formatiert sein
    let header_line = &result.lines[0];
    assert!(header_line.spans[0].content.starts_with("● "));
    assert!(header_line.spans[0].content.contains("Header test"));
    assert_eq!(header_line.spans[0].style.fg, Some(Color::White));
    assert!(header_line.spans[0].style.add_modifier.contains(Modifier::BOLD));
    
    // Normale Zeile sollte unformatiert sein
    let normal_line = &result.lines[1];
    assert_eq!(normal_line.spans[0].content, "Regular text");
}

#[test]
fn test_process_styled_text_user_ai_labels() {
    let text = "YOU: User message\nAI: AI response\nRegular line";
    let mut result = Text::default();
    
    process_styled_text(text, &mut result);
    
    assert!(result.lines.len() >= 3);
    
    // YOU: Label
    let you_line = &result.lines[0];
    assert_eq!(you_line.spans[0].content, "YOU:");
    assert_eq!(you_line.spans[0].style.fg, Some(Color::Magenta));
    assert!(you_line.spans[0].style.add_modifier.contains(Modifier::BOLD));
    assert_eq!(you_line.spans[1].content, " User message");
    
    // AI: Label
    let ai_line = &result.lines[1];
    assert_eq!(ai_line.spans[0].content, "AI: ");
    assert_eq!(ai_line.spans[0].style.fg, Some(Color::Cyan));
    assert!(ai_line.spans[0].style.add_modifier.contains(Modifier::BOLD));
    assert_eq!(ai_line.spans[1].content, " AI response");
    
    // Regular line
    let regular_line = &result.lines[2];
    assert_eq!(regular_line.spans[0].content, "Regular line");
    assert_eq!(regular_line.spans[0].style, Style::default());
}

#[test]
fn test_process_styled_text_mixed_content() {
    let text = "### Important\nYOU: Question\nAI: Answer\n### Another header\nNormal text";
    let mut result = Text::default();
    
    process_styled_text(text, &mut result);
    
    assert!(result.lines.len() >= 5);
    
    // Erste Header
    assert!(result.lines[0].spans[0].content.starts_with("● Important"));
    assert_eq!(result.lines[0].spans[0].style.fg, Some(Color::White));
    
    // YOU/AI Labels
    assert_eq!(result.lines[1].spans[0].content, "YOU:");
    assert_eq!(result.lines[1].spans[0].style.fg, Some(Color::Magenta));
    assert_eq!(result.lines[2].spans[0].content, "AI: ");
    assert_eq!(result.lines[2].spans[0].style.fg, Some(Color::Cyan));
    
    // Zweite Header  
    assert!(result.lines[3].spans[0].content.starts_with("● Another header"));
    assert_eq!(result.lines[3].spans[0].style.fg, Some(Color::White));
    
    // Normal text
    assert_eq!(result.lines[4].spans[0].content, "Normal text");
}

#[test]
fn test_parse_history_empty_string() {
    let parsed = parse_history("");
    assert!(parsed.lines.is_empty() || parsed.lines.len() == 1);
}

#[test]
fn test_parse_history_whitespace_only() {
    let parsed = parse_history("   \n  \n   ");
    // Sollte Whitespace-Zeilen beibehalten oder korrekt verarbeiten
    assert!(parsed.lines.len() >= 3);
}

#[test]
fn test_code_block_edge_cases() {
    // Unvollständiger Code-Block
    let history1 = "```rust\nfn main() {";
    let parsed1 = parse_history(history1);
    // Sollte nicht crashen, aber möglicherweise nicht als Code-Block erkannt
    assert!(parsed1.lines.len() > 0);
    
    // Leerer Code-Block
    let history2 = "```\n```";
    let parsed2 = parse_history(history2);
    assert!(parsed2.lines.len() > 0);
    
    // Verschachtelte Backticks (sollten ignoriert werden)
    let history3 = "```\n`inner code`\n```";
    let parsed3 = parse_history(history3);
    assert!(parsed3.lines.len() > 2);
}

#[test]
fn test_special_characters_in_labels() {
    let text = "YOU: Message with üñíçødé\nAI: Response with 🦀 emoji";
    let mut result = Text::default();
    
    process_styled_text(text, &mut result);
    
    // Sollte Unicode korrekt verarbeiten
    assert_eq!(result.lines[0].spans[1].content, " Message with üñíçødé");
    assert_eq!(result.lines[1].spans[1].content, " Response with 🦀 emoji");
}

#[test]
fn test_banner_constant() {
    // Banner sollte nicht leer sein und ASCII-Art enthalten
    assert!(!BANNER.is_empty());
    assert!(BANNER.contains('|'));
    assert!(BANNER.contains('_'));
    assert!(BANNER.len() > 100); // Sollte eine angemessene Größe haben
}

#[test]
fn test_long_lines_in_history() {
    let long_line = "A".repeat(1000);
    let history = format!("YOU: {}\nAI: Response", long_line);
    let parsed = parse_history(&history);
    
    // Sollte lange Zeilen handhaben ohne zu crashen
    assert!(parsed.lines.len() >= 2);
    assert!(parsed.lines[0].spans[1].content.len() > 900);
}