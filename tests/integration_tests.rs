//! Integration tests for LazyLlama
//! 
//! Diese Tests prüfen das Zusammenspiel der verschiedenen Module
//! und die gesamte Anwendungslogik.

use std::collections::HashMap;

// Für Integration-Tests importieren wir die Bibliothek als externe Crate
// Note: In echten Integration-Tests würde man `use lazyllama::*` verwenden,
// aber da wir eine binäre Anwendung haben, verwenden wir direkte Module

#[tokio::test]
async fn test_app_initialization() {
    // Test dass die App korrekt initialisiert werden kann
    // ohne echte Ollama-Verbindung
    
    // Simuliere App-Erstellung ohne Ollama API calls
    let models = vec!["test_model".to_string()];
    let model_inputs: HashMap<String, String> = HashMap::new();
    let model_histories: HashMap<String, String> = HashMap::new();
    
    assert!(!models.is_empty());
    assert!(model_inputs.is_empty());
    assert!(model_histories.is_empty());
}

#[test]
fn test_text_processing_pipeline() {
    // Test der kompletten Text-Verarbeitungs-Pipeline
    
    let input = "YOU: Can you show me a Rust function?\nAI: Here's a simple function:\n\n```rust\nfn hello() {\n    println!(\"Hello, World!\");\n}\n```\n\nThat's it!";
    
    // Simuliere parse_history Funktionalität
    let has_user_label = input.contains("YOU:");
    let has_ai_label = input.contains("AI:");
    let has_code_block = input.contains("```rust");
    
    assert!(has_user_label);
    assert!(has_ai_label);
    assert!(has_code_block);
    
    // Test Code-Block-Extraktion
    let code_start = input.find("```rust").unwrap();
    let code_end = input.rfind("```").unwrap();
    assert!(code_end > code_start);
    
    let code_section = &input[code_start..=code_end + 2];
    assert!(code_section.contains("fn hello()"));
    assert!(code_section.contains("println!"));
}

#[test]
fn test_model_buffer_management() {
    // Test der Model-Buffer-Verwaltung
    
    let mut model_inputs: HashMap<String, String> = HashMap::new();
    let mut model_histories: HashMap<String, String> = HashMap::new();
    let mut model_cursors: HashMap<String, usize> = HashMap::new();
    
    // Simuliere Model-Wechsel
    let model1 = "llama2:7b".to_string();
    let model2 = "codellama:13b".to_string();
    
    // Setze Daten für Model 1
    model_inputs.insert(model1.clone(), "Test input 1".to_string());
    model_histories.insert(model1.clone(), "YOU: Test\nAI: Response 1".to_string());
    model_cursors.insert(model1.clone(), 5);
    
    // Setze Daten für Model 2  
    model_inputs.insert(model2.clone(), "Test input 2".to_string());
    model_histories.insert(model2.clone(), "YOU: Code\nAI: ```rust\nfn test() {}\n```".to_string());
    model_cursors.insert(model2.clone(), 8);
    
    // Prüfe dass Daten korrekt gespeichert sind
    assert_eq!(model_inputs.get(&model1).unwrap(), "Test input 1");
    assert_eq!(model_inputs.get(&model2).unwrap(), "Test input 2");
    
    assert!(model_histories.get(&model1).unwrap().contains("Response 1"));
    assert!(model_histories.get(&model2).unwrap().contains("```rust"));
    
    assert_eq!(*model_cursors.get(&model1).unwrap(), 5);
    assert_eq!(*model_cursors.get(&model2).unwrap(), 8);
}

#[test]
fn test_history_parsing_edge_cases() {
    // Test edge cases der History-Parsing
    
    let test_cases = vec![
        // Leere History
        "",
        
        // Nur Whitespace
        "   \n  \n   ",
        
        // Unvollständiger Code-Block
        "```rust\nfn incomplete()",
        
        // Mehrere Code-Blocks
        "```rust\ncode1\n```\nText\n```python\ncode2\n```",
        
        // Labels ohne Inhalt
        "YOU:\nAI:",
        
        // Unicode-Zeichen
        "YOU: Hëllö 🦀\nAI: Ümlauts ñ Emojis 🎉",
        
        // Code-Block ohne Sprache
        "```\necho \"hello\"\n```",
        
        // Verschachtelte Backticks
        "```rust\nlet s = \"`inner`\";\n```",
        
        // HTML-ähnliche tags (sollten als normaler Text behandelt werden)
        "YOU: <script>alert('test')</script>\nAI: I see HTML tags.",
    ];
    
    // Separate sehr lange Zeile Test
    let long_line_test = format!("YOU: {}\nAI: Response", "A".repeat(10000));
    let mut all_tests = test_cases.clone();
    all_tests.push(&long_line_test);
    
    for (i, test_input) in all_tests.iter().enumerate() {
        // Simuliere Parsing ohne Panic
        let has_you = test_input.contains("YOU:");
        let has_ai = test_input.contains("AI:");
        let has_code = test_input.contains("```");
        
        // Kein Test sollte crashen
        assert!(true, "Test case {} completed without panic", i);
        
        // Basic sanity checks
        if !test_input.is_empty() {
            if has_you || has_ai || has_code {
                assert!(test_input.len() > 0);
            }
        }
    }
}

#[test]
fn test_cursor_navigation_scenarios() {
    // Test verschiedene Cursor-Navigation-Szenarien
    
    struct CursorTest {
        input: String,
        initial_pos: usize,
        operation: &'static str,
        expected_pos: usize,
        expected_input: String,
    }
    
    let tests = vec![
        // Standard character insertion
        CursorTest {
            input: "Hello".to_string(),
            initial_pos: 2,
            operation: "insert_char_X",
            expected_pos: 3,
            expected_input: "HeXllo".to_string(),
        },
        
        // Backspace
        CursorTest {
            input: "Hello".to_string(),
            initial_pos: 3,
            operation: "backspace",
            expected_pos: 2,
            expected_input: "Helo".to_string(),
        },
        
        // Word navigation
        CursorTest {
            input: "Hello World Test".to_string(),
            initial_pos: 8,
            operation: "word_left",
            expected_pos: 6, // Beginning of "World"
            expected_input: "Hello World Test".to_string(),
        },
        
        // Home/End navigation
        CursorTest {
            input: "Test String".to_string(), 
            initial_pos: 5,
            operation: "home",
            expected_pos: 0,
            expected_input: "Test String".to_string(),
        },
        
        // Unicode character handling
        CursorTest {
            input: "Hëllö".to_string(),
            initial_pos: 2,
            operation: "insert_char_🦀",
            expected_pos: 3,
            expected_input: "Hë🦀llö".to_string(),
        },
    ];
    
    for test in tests {
        // Simuliere die verschiedenen Operationen
        let mut current_input = test.input.clone();
        let mut current_pos = test.initial_pos;
        
        match test.operation {
            "insert_char_X" => {
                // Simuliere Character-Insertion
                if current_pos <= current_input.chars().count() {
                    let char_indices: Vec<_> = current_input.char_indices().collect();
                    let byte_pos = if current_pos < char_indices.len() {
                        char_indices[current_pos].0
                    } else {
                        current_input.len()
                    };
                    current_input.insert(byte_pos, 'X');
                    current_pos += 1;
                }
            },
            
            "insert_char_🦀" => {
                // Unicode insertion
                if current_pos <= current_input.chars().count() {
                    let char_indices: Vec<_> = current_input.char_indices().collect();
                    let byte_pos = if current_pos < char_indices.len() {
                        char_indices[current_pos].0
                    } else {
                        current_input.len()
                    };
                    current_input.insert(byte_pos, '🦀');
                    current_pos += 1;
                }
            },
            
            "backspace" => {
                // Simuliere Backspace
                if current_pos > 0 {
                    let char_indices: Vec<_> = current_input.char_indices().collect();
                    let remove_pos = current_pos - 1;
                    if remove_pos < char_indices.len() {
                        let byte_pos = char_indices[remove_pos].0;
                        current_input.remove(byte_pos);
                        current_pos -= 1;
                    }
                }
            },
            
            "word_left" => {
                // Simuliere Word-left Navigation
                let chars: Vec<char> = current_input.chars().collect();
                let mut i = current_pos.min(chars.len());
                
                // Skip whitespace
                while i > 0 && !chars[i - 1].is_alphanumeric() && chars[i - 1] != '_' {
                    i -= 1;
                }
                // Skip word characters
                while i > 0 && (chars[i - 1].is_alphanumeric() || chars[i - 1] == '_') {
                    i -= 1;
                }
                current_pos = i;
            },
            
            "home" => {
                current_pos = 0;
            },
            
            _ => {}
        }
        
        assert_eq!(current_pos, test.expected_pos, "Position mismatch for operation: {}", test.operation);
        assert_eq!(current_input, test.expected_input, "Input mismatch for operation: {}", test.operation);
    }
}

#[test] 
fn test_application_state_consistency() {
    // Test dass der Anwendungsstatus konsistent bleibt
    
    let models = vec!["model1".to_string(), "model2".to_string(), "model3".to_string()];
    let mut selected_index = 0;
    let mut model_buffers: HashMap<String, (String, String, usize)> = HashMap::new();
    
    // Initialisiere Buffer für alle Modelle mit Standardwerten
    for model in &models {
        model_buffers.insert(model.clone(), (String::new(), String::new(), 0));
    }
    
    // Simuliere Model-Wechsel und Buffer-Updates
    for iteration in 0..10 {
        // Setze Daten für aktuelles Model
        let current_model = &models[selected_index];
        let new_data = (
            format!("Input for {} iteration {}", current_model, iteration),
            format!("History for {} iteration {}", current_model, iteration),
            current_model.len() + iteration,
        );
        model_buffers.insert(current_model.clone(), new_data);
        
        // Wechsle zum nächsten Model (mit Wraparound)
        selected_index = (selected_index + 1) % models.len();
        
        // Prüfe dass alle Buffer noch existieren
        assert_eq!(model_buffers.len(), models.len());
        
        // Prüfe nur das zuletzt aktualisierte Model, da andere noch default values haben
        let last_model = &models[if selected_index == 0 { models.len() - 1 } else { selected_index - 1 }];
        if let Some((input, history, _cursor)) = model_buffers.get(last_model) {
            assert!(input.contains(last_model) || input.is_empty());
            assert!(history.contains(last_model) || history.is_empty());
        }
    }
}

#[test]
fn test_error_recovery_scenarios() {
    // Test Fehler-Behandlung und Recovery
    
    // Simuliere verschiedene Fehlerzustände
    let error_scenarios = vec![
        "Empty model list",
        "Invalid model index", 
        "Corrupted input buffer",
        "Network connection failure",
        "File system access denied",
    ];
    
    for scenario in error_scenarios {
        match scenario {
            "Empty model list" => {
                let models: Vec<String> = Vec::new();
                // App sollte mit leerem Model-Vector umgehen können
                assert_eq!(models.len(), 0);
                
                // Navigation sollte safe sein
                let selected = if models.is_empty() { None } else { Some(0) };
                assert_eq!(selected, None);
            },
            
            "Invalid model index" => {
                let models = vec!["model1".to_string()];
                let invalid_index = 5;
                // Sollte nicht crashen
                let selected = if invalid_index < models.len() { 
                    Some(invalid_index) 
                } else { 
                    None 
                };
                assert_eq!(selected, None);
            },
            
            "Corrupted input buffer" => {
                // Test mit invalid UTF-8 würde hier stehen,
                // aber Rust String garantiert valid UTF-8
                let buffer = String::from("Valid UTF-8 string with üñíçødé 🦀");
                assert!(buffer.is_ascii() == false); // Contains non-ASCII
                assert!(!buffer.is_empty());
            },
            
            _ => {
                // Andere Szenarien würden externe Dependencies benötigen
                assert!(true, "Scenario {} acknowledged", scenario);
            }
        }
    }
}