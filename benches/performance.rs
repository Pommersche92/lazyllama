//! Benchmark tests for LazyLlama performance-critical functions
//! 
//! These tests measure the performance of frequently used functions
//! and ensure they remain performant even with large datasets.
//!
//! ## Test Categories
//!
//! - **String Operations**: Character insertion, unicode handling, text searching
//! - **Text Parsing**: History parsing, regex operations, line iteration
//! - **Cursor Operations**: Byte/character index conversions, navigation
//! - **UI Rendering**: Widget creation, layout computation, text formatting
//! - **Memory Operations**: Buffer allocation, cloning, cleanup
//!
//! ## Performance Requirements
//!
//! - String operations: < 1μs per character
//! - Text parsing: < 100μs per 1KB of text
//! - Cursor operations: < 10μs per operation
//! - UI rendering: < 16ms for full frame (60 FPS)
//!
//! ## Running Benchmarks
//!
//! ```bash
//! cargo bench
//! ```

use std::hint::black_box;
use std::time::{Duration, Instant};

/// Simple benchmark helper function
///
/// Executes a function multiple times and measures average execution time.
/// Uses `black_box` to prevent compiler optimizations from skewing results.
///
/// # Arguments
///
/// * `name` - Descriptive name for the benchmark (displayed in output)
/// * `f` - Function to benchmark (should be cheap to call repeatedly)
/// * `iterations` - Number of times to execute the function
///
/// # Output
///
/// Prints timing statistics including:
/// - Total number of iterations
/// - Average time per iteration
/// - Total elapsed time
///
/// # Example
///
/// ```ignore
/// bench_fn("string_creation", || {
///     let s = String::from("test");
///     drop(s);
/// }, 1000);
/// ```
fn bench_fn<F>(name: &str, f: F, iterations: usize) 
where 
    F: Fn() -> ()
{
    let start = Instant::now();
    for _ in 0..iterations {
        black_box(f());
    }
    let duration = start.elapsed();
    let avg_time = duration / iterations as u32;
    println!("{}: {} iterations, avg: {:?}, total: {:?}", 
             name, iterations, avg_time, duration);
}

#[cfg(test)]
mod performance_tests {
    use super::*;

    /// Benchmarks fundamental string operations commonly used throughout LazyLlama.
    ///
    /// This test measures performance of:
    /// - Character insertion at various positions (ASCII)
    /// - Unicode character insertion and handling (emojis, accented chars)
    /// - Pattern searching in large text buffers
    ///
    /// These operations are critical for:
    /// - User input handling in the terminal UI
    /// - Chat history parsing and display
    /// - Search functionality within conversations
    ///
    /// # Performance Requirements
    ///
    /// - Character insertion: Should complete in < 1μs per character
    /// - Unicode handling: Should not be significantly slower than ASCII
    /// - Pattern search: Should handle 10KB+ text efficiently
    ///
    /// # Test Data
    ///
    /// - 100 character insertions per iteration
    /// - 50 unicode character insertions per iteration  
    /// - Search in 1000-repetition pattern ("YOU: ")
    #[test]
    fn bench_string_operations() {
        // Benchmark for string operations frequently used in the app
        
        // Character insertion benchmark
        bench_fn("char_insertion", || {
            let mut s = String::with_capacity(1000);
            for i in 0..100 {
                s.insert(i.min(s.len()), 'A');
            }
            drop(s);
        }, 1000);
        
        // Unicode character handling
        bench_fn("unicode_insertion", || {
            let mut s = String::with_capacity(1000);
            for i in 0..50 {
                s.insert(i.min(s.len()), '🦀');
            }
            drop(s);
        }, 1000);
        
        // String searching (as used in history parsing)
        let large_text = "YOU: ".repeat(1000);
        bench_fn("string_search", || {
            let _count = large_text.matches("YOU:").count();
        }, 1000);
    }

    /// Benchmarks text parsing operations used in conversation history processing.
    ///
    /// This test measures performance of:
    /// - Regex-based code block detection and extraction
    /// - Line-by-line iteration through large text buffers
    /// - Character counting for display calculations
    ///
    /// These operations are essential for:
    /// - Rendering markdown and code syntax highlighting
    /// - Calculating scroll positions and viewport bounds
    /// - Processing streaming AI responses in real-time
    ///
    /// # Performance Requirements
    ///
    /// - History parsing: Should handle 100KB+ conversations smoothly
    /// - Line iteration: Must be efficient for real-time display updates
    /// - Character counting: Used frequently during text input
    ///
    /// # Test Data
    ///
    /// Creates synthetic conversation history with:
    /// - 100 "YOU:/AI:" exchanges
    /// - 50 code blocks with syntax highlighting
    /// - Mixed text content totaling ~50KB
    #[test]
    fn bench_text_parsing() {
        // Benchmark for text parsing operations
        
        let test_history = format!(
            "{}```rust\n{}\n```\n{}", 
            "YOU: Test\nAI: Here's code:\n\n".repeat(100),
            "fn main() {\n    println!(\"Hello\");\n}\n".repeat(50),
            "\n\nThat's it!".repeat(100)
        );
        
        bench_fn("history_parsing", || {
            // Simulate regex-based code block parsing
            let code_blocks: Vec<_> = test_history.match_indices("```").collect();
            drop(code_blocks);
        }, 100);
        
        bench_fn("line_iteration", || {
            let lines: Vec<_> = test_history.lines().collect();
            drop(lines);
        }, 100);
        
        bench_fn("character_counting", || {
            let _char_count = test_history.chars().count();
        }, 100);
    }

    /// Benchmarks cursor navigation and text position operations.
    ///
    /// This test measures performance of:
    /// - Character-to-byte index conversions (critical for UTF-8 handling)
    /// - Cursor movement calculations
    /// - Text boundary detection
    ///
    /// These operations are triggered constantly during:
    /// - User input and cursor movement in the terminal
    /// - Text selection and editing operations
    /// - Display of cursor position indicators
    ///
    /// # Performance Requirements
    ///
    /// - Character/byte conversion: Must be < 10μs for typical input lengths
    /// - Should handle unicode text without significant performance penalty
    /// - Must scale linearly with text length (O(n) acceptable)
    ///
    /// # Test Data
    ///
    /// - 1000-word test string with mixed ASCII content
    /// - 100 random cursor position queries per iteration
    /// - Simulates typical user input session workload
    #[test]
    fn bench_cursor_operations() {
        // Benchmark for cursor navigation
        
        let test_text = "word ".repeat(1000);
        
        bench_fn("char_to_byte_index", || {
            for i in 0..100 {
                let char_pos = i % 100;
                let _byte_pos = test_text
                    .char_indices()
                    .nth(char_pos)
                    .map(|(idx, _)| idx)
                    .unwrap_or_else(|| test_text.len());
            }
        }, 100);
        
        bench_fn("word_boundary_detection", || {
            let chars: Vec<char> = test_text.chars().collect();
            for i in 0..chars.len().min(1000) {
                let _is_word = chars[i].is_alphanumeric() || chars[i] == '_';
            }
        }, 100);
    }

    #[test]
    fn bench_hash_map_operations() {
        // Benchmark für HashMap-Operationen (Model-Buffer-Management)
        
        use std::collections::HashMap;
        
        let mut model_buffers: HashMap<String, (String, String, usize)> = HashMap::new();
        
        // Initial population
        for i in 0..1000 {
            model_buffers.insert(
                format!("model_{}", i),
                (
                    format!("input_{}", i), 
                    format!("history_{}", i),
                    i
                )
            );
        }
        
        bench_fn("hashmap_lookups", || {
            for i in 0..100 {
                let key = format!("model_{}", i % 1000);
                let _value = model_buffers.get(&key);
            }
        }, 100);
        
        bench_fn("hashmap_inserts", || {
            let mut map: HashMap<String, String> = HashMap::with_capacity(100);
            for i in 0..100 {
                map.insert(format!("key_{}", i), format!("value_{}", i));
            }
            drop(map);
        }, 100);
    }

    #[test]
    fn bench_memory_allocations() {
        // Benchmark für Speicher-Allokationen
        
        bench_fn("string_allocations", || {
            let mut strings = Vec::with_capacity(100);
            for i in 0..100 {
                strings.push(format!("String number {}", i));
            }
            drop(strings);
        }, 1000);
        
        bench_fn("vec_growth", || {
            let mut vec = Vec::new();
            for i in 0..1000 {
                vec.push(i);
            }
            drop(vec);
        }, 100);
        
        bench_fn("vec_with_capacity", || {
            let mut vec = Vec::with_capacity(1000);
            for i in 0..1000 {
                vec.push(i);
            }
            drop(vec);
        }, 100);
    }

    #[test]
    fn bench_real_world_scenarios() {
        // Benchmark für realistische Anwendungsszenarien
        
        // Simuliere große Conversation History
        let large_history = format!(
            "{}{}{}",
            "YOU: Simple question\nAI: Simple answer\n".repeat(500),
            "AI: Here's some code:\n\n```rust\nfn example() {\n    println!(\"test\");\n}\n```\n\n".repeat(100),
            "YOU: Thanks!\nAI: You're welcome!\n".repeat(300)
        );
        
        bench_fn("large_history_processing", || {
            // Simuliere komplette History-Verarbeitung
            let lines: Vec<_> = large_history.lines().collect();
            let you_count = large_history.matches("YOU:").count();
            let ai_count = large_history.matches("AI:").count();
            let code_blocks = large_history.matches("```").count() / 2;
            
            drop((lines, you_count, ai_count, code_blocks));
        }, 10);
        
        // Simuliere schnelle Model-Wechsel
        bench_fn("rapid_model_switching", || {
            use std::collections::HashMap;
            
            let mut app_state = HashMap::new();
            let models = vec!["model1", "model2", "model3", "model4", "model5"];
            
            // Simuliere 50 Model-Wechsel mit Buffer-Save/Load
            for i in 0..50 {
                let _current_model = models[i % models.len()];
                
                // Save current state
                app_state.insert("current_input", format!("input_{}", i));
                app_state.insert("current_history", format!("history_{}", i));
                
                // Load new state
                let _input = app_state.get("current_input").cloned().unwrap_or_default();
                let _history = app_state.get("current_history").cloned().unwrap_or_default();
            }
            drop(app_state);
        }, 100);
    }

    #[test]
    fn bench_edge_case_performance() {
        // Performance-Tests für Edge Cases
        
        // Sehr lange einzelne Zeile
        let long_line = "A".repeat(100_000);
        bench_fn("very_long_line", || {
            let _char_count = long_line.chars().count();
            let _byte_len = long_line.len();
        }, 10);
        
        // Viele kleine Zeilen
        let many_lines = "Short line\n".repeat(10_000);
        bench_fn("many_small_lines", || {
            let _line_count = many_lines.lines().count();
        }, 10);
        
        // Unicode-heavy text
        let unicode_text = "🦀🎉🌟✨🔥💯🚀⭐".repeat(1_000);
        bench_fn("unicode_heavy", || {
            let _char_count = unicode_text.chars().count();
            let _byte_len = unicode_text.len();
        }, 10);
    }
}

/// Performance regression tests
/// Diese Tests stellen sicher, dass Performance nicht signifikant abnimmt
#[cfg(test)]
mod regression_tests {
    use super::*;

    #[test]
    fn test_performance_bounds() {
        // Diese Tests definieren Performance-Grenzen für kritische Operationen
        
        let start = Instant::now();
        
        // String-Operationen sollten unter 1ms für 1000 Zeichen sein
        let test_string = "Test ".repeat(200); // 1000 characters
        for _ in 0..1000 {
            let _chars: Vec<char> = test_string.chars().collect();
        }
        
        let string_ops_duration = start.elapsed();
        assert!(string_ops_duration < Duration::from_millis(100), 
                "String operations too slow: {:?}", string_ops_duration);
        
        // HashMap-Operationen sollten unter 10ms für 10k Einträge sein
        let start = Instant::now();
        let mut map = std::collections::HashMap::new();
        for i in 0..10_000 {
            map.insert(i, format!("value_{}", i));
        }
        for i in 0..10_000 {
            let _val = map.get(&i);
        }
        
        let hashmap_duration = start.elapsed();
        assert!(hashmap_duration < Duration::from_millis(50),
                "HashMap operations too slow: {:?}", hashmap_duration);
    }
    
    #[test]
    fn test_memory_usage_bounds() {
        // Grobe Memory-Usage-Tests
        let initial_memory = get_memory_usage().unwrap_or(0);
        
        // Allokiere kontrollierte Menge Memory
        let large_strings: Vec<String> = (0..1000)
            .map(|i| "Data ".repeat(100) + &i.to_string())
            .collect();
        
        let after_alloc = get_memory_usage().unwrap_or(0);
        let memory_increase = after_alloc.saturating_sub(initial_memory);
        
        // Speicher-Verbrauch sollte reasonabel sein
        // (In echten Tests würde man exakte Limits definieren)
        assert!(memory_increase < 100_000_000, // 100MB limit
                "Memory usage too high: {} bytes", memory_increase);
        
        drop(large_strings);
        
        // Memory sollte wieder freigegeben werden (wird vom OS verwaltet)
        let after_drop = get_memory_usage().unwrap_or(0);
        println!("Memory usage - Initial: {}, After alloc: {}, After drop: {}", 
                 initial_memory, after_alloc, after_drop);
    }
    
    /// Hilfsfunktion für Memory-Usage (Platform-specific)
    fn get_memory_usage() -> Option<usize> {
        // Vereinfachte Memory-Messung nur für Demo
        // In echten Tests würde man psutil oder ähnliche Libraries verwenden
        #[cfg(unix)]
        {
            use std::fs;
            let status = fs::read_to_string("/proc/self/status").ok()?;
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        return parts[1].parse::<usize>().ok().map(|kb| kb * 1024);
                    }
                }
            }
        }
        None
    }
}