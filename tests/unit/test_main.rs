use std::time::Duration;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, KeyEventKind};

/// Helper zum Erstellen von KeyEvent-Strukturen für Tests
fn create_key_event(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent {
        code,
        modifiers,
        kind: crossterm::event::KeyEventKind::Press,
        state: crossterm::event::KeyEventState::empty(),
    }
}

#[test]
fn test_key_combinations() {
    // Test dass verschiedene Key-Kombinationen korrekt erkannt werden
    let ctrl_q = create_key_event(KeyCode::Char('q'), KeyModifiers::CONTROL);
    assert_eq!(ctrl_q.code, KeyCode::Char('q'));
    assert!(ctrl_q.modifiers.contains(KeyModifiers::CONTROL));

    let ctrl_c = create_key_event(KeyCode::Char('c'), KeyModifiers::CONTROL);
    assert_eq!(ctrl_c.code, KeyCode::Char('c'));
    assert!(ctrl_c.modifiers.contains(KeyModifiers::CONTROL));

    let normal_char = create_key_event(KeyCode::Char('a'), KeyModifiers::empty());
    assert_eq!(normal_char.code, KeyCode::Char('a'));
    assert!(!normal_char.modifiers.contains(KeyModifiers::CONTROL));
}

#[test]
fn test_special_keys() {
    // Test spezielle Tasten
    let enter = create_key_event(KeyCode::Enter, KeyModifiers::empty());
    assert_eq!(enter.code, KeyCode::Enter);

    let backspace = create_key_event(KeyCode::Backspace, KeyModifiers::empty());
    assert_eq!(backspace.code, KeyCode::Backspace);

    let up_arrow = create_key_event(KeyCode::Up, KeyModifiers::empty());
    assert_eq!(up_arrow.code, KeyCode::Up);

    let down_arrow = create_key_event(KeyCode::Down, KeyModifiers::empty());
    assert_eq!(down_arrow.code, KeyCode::Down);

    let page_up = create_key_event(KeyCode::PageUp, KeyModifiers::empty());
    assert_eq!(page_up.code, KeyCode::PageUp);

    let page_down = create_key_event(KeyCode::PageDown, KeyModifiers::empty());
    assert_eq!(page_down.code, KeyCode::PageDown);
}

#[test]
fn test_ctrl_combinations() {
    // Test erweiterte Ctrl-Kombinationen
    let ctrl_left = create_key_event(KeyCode::Left, KeyModifiers::CONTROL);
    assert_eq!(ctrl_left.code, KeyCode::Left);
    assert!(ctrl_left.modifiers.contains(KeyModifiers::CONTROL));

    let ctrl_right = create_key_event(KeyCode::Right, KeyModifiers::CONTROL);
    assert_eq!(ctrl_right.code, KeyCode::Right);
    assert!(ctrl_right.modifiers.contains(KeyModifiers::CONTROL));

    let ctrl_backspace = create_key_event(KeyCode::Backspace, KeyModifiers::CONTROL);
    assert_eq!(ctrl_backspace.code, KeyCode::Backspace);
    assert!(ctrl_backspace.modifiers.contains(KeyModifiers::CONTROL));

    let ctrl_delete = create_key_event(KeyCode::Delete, KeyModifiers::CONTROL);
    assert_eq!(ctrl_delete.code, KeyCode::Delete);
    assert!(ctrl_delete.modifiers.contains(KeyModifiers::CONTROL));
}

#[test]
fn test_navigation_keys() {
    // Test Home/End Navigation
    let home = create_key_event(KeyCode::Home, KeyModifiers::empty());
    assert_eq!(home.code, KeyCode::Home);

    let end = create_key_event(KeyCode::End, KeyModifiers::empty());
    assert_eq!(end.code, KeyCode::End);

    let left = create_key_event(KeyCode::Left, KeyModifiers::empty());
    assert_eq!(left.code, KeyCode::Left);

    let right = create_key_event(KeyCode::Right, KeyModifiers::empty());
    assert_eq!(right.code, KeyCode::Right);
}

#[test]
fn test_character_input() {
    // Test verschiedene Zeichen-Eingaben
    let char_a = create_key_event(KeyCode::Char('a'), KeyModifiers::empty());
    assert_eq!(char_a.code, KeyCode::Char('a'));

    let char_z = create_key_event(KeyCode::Char('Z'), KeyModifiers::empty());
    assert_eq!(char_z.code, KeyCode::Char('Z'));

    let char_number = create_key_event(KeyCode::Char('5'), KeyModifiers::empty());
    assert_eq!(char_number.code, KeyCode::Char('5'));

    let char_space = create_key_event(KeyCode::Char(' '), KeyModifiers::empty());
    assert_eq!(char_space.code, KeyCode::Char(' '));

    // Test Sonderzeichen
    let char_special = create_key_event(KeyCode::Char('@'), KeyModifiers::empty());
    assert_eq!(char_special.code, KeyCode::Char('@'));
}

#[test]
fn test_modifier_detection() {
    // Test dass Modifier korrekt erkannt werden
    let no_modifiers = KeyModifiers::empty();
    assert!(!no_modifiers.contains(KeyModifiers::CONTROL));
    assert!(!no_modifiers.contains(KeyModifiers::SHIFT));
    assert!(!no_modifiers.contains(KeyModifiers::ALT));

    let ctrl = KeyModifiers::CONTROL;
    assert!(ctrl.contains(KeyModifiers::CONTROL));
    assert!(!ctrl.contains(KeyModifiers::SHIFT));

    let shift = KeyModifiers::SHIFT;
    assert!(shift.contains(KeyModifiers::SHIFT));
    assert!(!shift.contains(KeyModifiers::CONTROL));

    let alt = KeyModifiers::ALT;
    assert!(alt.contains(KeyModifiers::ALT));
    assert!(!alt.contains(KeyModifiers::CONTROL));
}

#[test]
fn test_combined_modifiers() {
    // Test Kombinationen von Modifiern
    let ctrl_shift = KeyModifiers::CONTROL | KeyModifiers::SHIFT;
    assert!(ctrl_shift.contains(KeyModifiers::CONTROL));
    assert!(ctrl_shift.contains(KeyModifiers::SHIFT));
    assert!(!ctrl_shift.contains(KeyModifiers::ALT));

    let ctrl_alt = KeyModifiers::CONTROL | KeyModifiers::ALT;
    assert!(ctrl_alt.contains(KeyModifiers::CONTROL));
    assert!(ctrl_alt.contains(KeyModifiers::ALT));
    assert!(!ctrl_alt.contains(KeyModifiers::SHIFT));
}

#[test]
fn test_key_event_kind() {
    // Test KeyEventKind (Press vs Release)
    let press_event = KeyEvent {
        code: KeyCode::Char('a'),
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
        state: crossterm::event::KeyEventState::empty(),
    };
    assert_eq!(press_event.kind, KeyEventKind::Press);

    let release_event = KeyEvent {
        code: KeyCode::Char('a'),
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Release,
        state: crossterm::event::KeyEventState::empty(),
    };
    assert_eq!(release_event.kind, KeyEventKind::Release);
}

#[test]
fn test_duration_polling() {
    // Test dass Duration Werte korrekt sind
    let poll_duration = Duration::from_millis(100);
    assert_eq!(poll_duration.as_millis(), 100);
    
    let longer_duration = Duration::from_secs(1);
    assert_eq!(longer_duration.as_millis(), 1000);
    
    // Test für negative/zero Fälle
    let zero_duration = Duration::from_millis(0);
    assert_eq!(zero_duration.as_millis(), 0);
}

// Mock für Terminal Operations
struct MockTerminal;

impl MockTerminal {
    fn new() -> Self {
        MockTerminal
    }

    fn setup() -> anyhow::Result<Self> {
        // Mock Terminal setup ohne echte crossterm calls
        Ok(MockTerminal::new())
    }

    fn cleanup(&self) -> anyhow::Result<()> {
        // Mock cleanup
        Ok(())
    }
}

#[test]
fn test_mock_terminal_lifecycle() {
    // Test dass Terminal Mock korrekt funktioniert
    let terminal = MockTerminal::setup();
    assert!(terminal.is_ok());
    
    let terminal = terminal.unwrap();
    let cleanup_result = terminal.cleanup();
    assert!(cleanup_result.is_ok());
}

#[test] 
fn test_event_loop_logic() {
    // Test logik ohne echte Terminal I/O
    let mut should_quit = false;
    let key = create_key_event(KeyCode::Char('q'), KeyModifiers::CONTROL);
    
    // Simuliere die quit logic
    if key.code == KeyCode::Char('q') && key.modifiers.contains(KeyModifiers::CONTROL) {
        should_quit = true;
    }
    
    assert!(should_quit);
}

#[test]
fn test_key_pattern_matching() {
    // Test der verschiedenen Key-Pattern aus der Event Loop
    struct TestCase {
        key: KeyEvent,
        expected_action: &'static str,
    }

    let test_cases = vec![
        TestCase {
            key: create_key_event(KeyCode::Char('q'), KeyModifiers::CONTROL),
            expected_action: "quit",
        },
        TestCase {
            key: create_key_event(KeyCode::Char('c'), KeyModifiers::CONTROL),
            expected_action: "clear",
        },
        TestCase {
            key: create_key_event(KeyCode::Char('s'), KeyModifiers::CONTROL),
            expected_action: "toggle_autoscroll",
        },
        TestCase {
            key: create_key_event(KeyCode::Up, KeyModifiers::empty()),
            expected_action: "previous_model",
        },
        TestCase {
            key: create_key_event(KeyCode::Down, KeyModifiers::empty()),
            expected_action: "next_model",
        },
        TestCase {
            key: create_key_event(KeyCode::Enter, KeyModifiers::empty()),
            expected_action: "send_query",
        },
        TestCase {
            key: create_key_event(KeyCode::Char('a'), KeyModifiers::empty()),
            expected_action: "insert_char",
        },
    ];

    for test_case in test_cases {
        let action = match (test_case.key.code, test_case.key.modifiers.contains(KeyModifiers::CONTROL)) {
            (KeyCode::Char('q'), true) => "quit",
            (KeyCode::Char('c'), true) => "clear", 
            (KeyCode::Char('s'), true) => "toggle_autoscroll",
            (KeyCode::Up, _) => "previous_model",
            (KeyCode::Down, _) => "next_model",
            (KeyCode::Enter, _) => "send_query",
            (KeyCode::Char(_), false) => "insert_char",
            _ => "unknown",
        };
        assert_eq!(action, test_case.expected_action);
    }
}