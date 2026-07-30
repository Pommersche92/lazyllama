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

//! Terminal User Interface rendering and layout functions.
//!
//! This module handles all aspects of the visual presentation including:
//! - Widget rendering and layout management
//! - Markdown parsing and syntax highlighting
//! - Scroll position calculation and management
//! - Real-time UI updates during AI response streaming
//!
//! The UI is built with Ratatui and features:
//! - Responsive layout that adapts to terminal size
//! - Code block highlighting with language detection
//! - Smart scrolling with autoscroll and manual modes
//! - Model status indicators and selection highlighting
//! - Animated loading indicators

use crate::app::App;
use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
use regex::Regex;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

/// ASCII art banner displayed at the top of the application.
/// 
/// This constant contains the stylized "LazyLlama" text that appears
/// in the header section of the terminal interface.
pub const BANNER: &str = r#"
| |    __ _  ______  __| |    | | __ _ _ __ ___   __ _
| |   / _` ||_  /\ \/ /| |    | |/ _` | '_ ` _ \ / _` |
| |__| (_| | / /  \  / | |___ | | (_| | | | | | | (_| |
|_____\__,_|/___| /_/  |_____||_|\__,_|_| |_| |_|\__,_|
"#;

/// Converts a syntect Color to a ratatui Color.
///
/// Syntect uses RGBA colors while ratatui uses RGB. This function
/// extracts the RGB components and creates a ratatui RGB color.
fn syntect_color_to_ratatui(color: syntect::highlighting::Color) -> Color {
    Color::Rgb(color.r, color.g, color.b)
}

/// Maps common markdown language tags to syntect language names.
///
/// Markdown code fences often use different identifiers than syntect's
/// internal language names. This function provides a mapping to ensure
/// proper syntax highlighting for commonly used language tags.
///
/// # Arguments
///
/// * `lang` - The language tag from markdown (e.g., "js", "py", "csharp")
///
/// # Returns
///
/// The syntect-compatible language name (e.g., "JavaScript", "Python", "C#")
fn map_language_tag(lang: &str) -> &str {
    match lang.to_lowercase().as_str() {
        // JavaScript variants
        "javascript" | "js" => "JavaScript",
        "typescript" | "ts" => "TypeScript",
        "jsx" => "JavaScript (Babel)",
        "tsx" => "TypeScript",
        
        // Python
        "python" | "py" => "Python",
        
        // Rust
        "rust" | "rs" => "Rust",
        
        // C family
        "c" => "C",
        "cpp" | "c++" | "cxx" => "C++",
        "csharp" | "cs" | "c#" => "C#",
        
        // JVM languages
        "java" => "Java",
        "kotlin" | "kt" => "Kotlin",
        "scala" => "Scala",
        "groovy" => "Groovy",
        
        // Other compiled languages
        "go" | "golang" => "Go",
        "swift" => "Swift",
        "objective-c" | "objc" => "Objective-C",
        "objective-c++" | "objc++" => "Objective-C++",
        
        // Scripting languages
        "ruby" | "rb" => "Ruby",
        "php" => "PHP",
        "perl" | "pl" => "Perl",
        "lua" => "Lua",
        "r" => "R",
        
        // Shell scripting
        "bash" | "sh" | "shell" => "Bourne Again Shell (bash)",
        "zsh" => "Bourne Again Shell (bash)", // Close enough
        "fish" => "Bourne Again Shell (bash)",
        "powershell" | "ps1" => "PowerShell",
        
        // Web technologies
        "html" | "htm" => "HTML",
        "css" => "CSS",
        "scss" | "sass" => "SCSS",
        "less" => "LESS",
        
        // Data formats
        "json" => "JSON",
        "yaml" | "yml" => "YAML",
        "toml" => "TOML",
        "xml" => "XML",
        "csv" => "CSV",
        
        // Markup
        "markdown" | "md" => "Markdown",
        "latex" | "tex" => "LaTeX",
        "rst" | "restructuredtext" => "reStructuredText",
        
        // SQL
        "sql" | "mysql" | "postgresql" | "postgres" => "SQL",
        
        // Functional languages
        "haskell" | "hs" => "Haskell",
        "ocaml" | "ml" => "OCaml",
        "erlang" | "erl" => "Erlang",
        "elixir" | "ex" | "exs" => "Elixir",
        "clojure" | "clj" => "Clojure",
        "fsharp" | "fs" | "f#" => "F#",
        
        // Lisp family
        "lisp" => "Lisp",
        "scheme" => "Scheme",
        
        // Other languages
        "dart" => "Dart",
        "julia" | "jl" => "Julia",
        "zig" => "Zig",
        "nim" => "Nim",
        "crystal" => "Crystal",
        "d" => "D",
        "v" | "vlang" => "V",
        
        // Config files
        "ini" | "cfg" => "INI",
        "env" | "dotenv" => "Shell Script (Bash)", // Approximation
        "dockerfile" | "docker" => "Dockerfile",
        "makefile" | "make" => "Makefile",
        
        // Default: return as-is
        _ => lang,
    }
}

/// Applies syntax highlighting to a code block and returns styled lines.
///
/// Uses syntect to parse and highlight code based on the specified language.
/// Falls back to plain text if the language is not recognized.
///
/// # Arguments
///
/// * `code` - The code content to highlight
/// * `language` - The programming language identifier (e.g., "rust", "python")
/// * `theme_name` - The name of the syntect theme to use for highlighting
///
/// # Returns
///
/// A vector of Lines with syntax-highlighted spans, each prefixed with a
/// yellow border character (│).
fn highlight_code_block(code: &str, language: &str, theme_name: &str) -> Vec<Line<'static>> {
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    
    // Map the language tag to syntect's expected name
    let mapped_lang = map_language_tag(language);
    
    // Try to find the syntax by mapped name, then extension, then original name
    let syntax = ps.find_syntax_by_name(mapped_lang)
        .or_else(|| ps.find_syntax_by_extension(mapped_lang))
        .or_else(|| ps.find_syntax_by_extension(language))
        .unwrap_or_else(|| ps.find_syntax_plain_text());
    
    // Use the specified theme, fallback to base16-ocean.dark if not found
    let theme = ts.themes.get(theme_name)
        .unwrap_or_else(|| &ts.themes["base16-ocean.dark"]);
    
    let mut highlighter = HighlightLines::new(syntax, theme);
    let mut lines = Vec::new();
    
    for line in LinesWithEndings::from(code) {
        let ranges = highlighter.highlight_line(line, &ps).unwrap_or_default();
        let mut spans = vec![
            Span::styled(" │ ".to_string(), Style::default().fg(Color::Yellow))
        ];
        
        for (style, text) in ranges {
            let fg_color = syntect_color_to_ratatui(style.foreground);
            let mut ratatui_style = Style::default().fg(fg_color);
            
            // Apply text modifiers based on syntect font style
            if style.font_style.contains(syntect::highlighting::FontStyle::BOLD) {
                ratatui_style = ratatui_style.add_modifier(Modifier::BOLD);
            }
            if style.font_style.contains(syntect::highlighting::FontStyle::ITALIC) {
                ratatui_style = ratatui_style.add_modifier(Modifier::ITALIC);
            }
            if style.font_style.contains(syntect::highlighting::FontStyle::UNDERLINE) {
                ratatui_style = ratatui_style.add_modifier(Modifier::UNDERLINED);
            }
            
            spans.push(Span::styled(text.to_string(), ratatui_style));
        }
        
        lines.push(Line::from(spans));
    }
    
    lines
}

/// Main rendering function for the Ratatui terminal interface.
///
/// This function orchestrates the complete UI layout and rendering process,
/// creating a responsive three-panel interface with header, main content,
/// and status bar. The layout dynamically adjusts to terminal size and
/// provides real-time updates during AI interactions.
///
/// # Arguments
///
/// * `f` - Mutable reference to the Ratatui frame for widget rendering
/// * `app` - Mutable reference to application state for data access and updates
///
/// # Layout Structure
///
/// ```text
/// ┌─────────────────────────────────────────┐
/// │              ASCII Banner               │ 7 lines
/// ├─────────────┬───────────────────────────┤
/// │   Models    │    Conversation History   │ Flexible
/// │   (25%)     │         (75%)             │ height
/// │             ├───────────────────────────┤
/// │             │      Input Field          │ 3-7 lines
/// ├─────────────┴───────────────────────────┤  (dynamic)
/// │            Status Bar                   │ 1 line
/// └─────────────────────────────────────────┘
/// ```
///
/// # Features
///
/// - **Model List**: Shows available AI models with status indicators
/// - **Chat History**: Displays conversation with markdown and code highlighting
/// - **Input Field**: Multiline text entry with dynamic height (1-5 content lines)
///   - Automatically expands based on newline characters
///   - Selection highlighting with blue background
///   - Blinking cursor with reversed colors
/// - **Status Bar**: Keyboard shortcuts and current model information
///   - Keybindings are dynamically pulled from the configured settings
/// - **Responsive Design**: Adapts to terminal size changes
/// - **Smart Scrolling**: Auto-scroll with manual override capability
///
/// # Visual Elements
///
/// - Models with conversation history show file icons (📝/📄)
/// - Selected model highlighted with different colors
/// - Loading state shows animated spinner in input field
/// - Scroll status indicator in conversation header
/// - Color-coded borders for different UI states
///
/// # Performance
///
/// This function is called frequently (up to 20fps during streaming)
/// and is optimized for minimal computational overhead while providing
/// smooth visual feedback.
pub fn ui(f: &mut Frame, app: &mut App) {
    if app.debug_keys {
        app.render_count = app.render_count.wrapping_add(1);
    }
    let root_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());

    f.render_widget(
        Paragraph::new(BANNER)
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center),
        root_layout[0],
    );

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(root_layout[1]);

    // Modellliste rendern mit erweiterten Informationen
    let selected_model = app.list_state.selected()
        .and_then(|i| app.models.get(i))
        .cloned()
        .unwrap_or_else(|| "None".to_string());
    
    let items: Vec<ListItem> = app
        .models
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let is_selected = app.list_state.selected() == Some(i);
            let history_len = app.model_histories.get(m).map(|h| h.len()).unwrap_or(0);
            let display = if history_len > 0 {
                format!("{} [{}]", m, if history_len > 1000 { "📝" } else { "📄" })
            } else {
                m.clone()
            };
            ListItem::new(display)
                .style(if is_selected {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                })
        })
        .collect();
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL)
            .title(format!(" Models ({}) ", app.models.len())))
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, main_chunks[0], &mut app.list_state);

    // Calculate dynamic input height based on newlines (1-5 content lines + 2 for borders)
    let input_line_count = app.input.lines().count().max(1).min(5);
    let input_height = (input_line_count + 2) as u16; // +2 for top and bottom border

    let chat_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(input_height)])
        .split(main_chunks[1]);

    // Verlauf parsen und Scrollen berechnen
    let history_text = parse_history(&app.history, app.settings.syntax_theme.as_str());
    let visible_height = chat_chunks[0].height.saturating_sub(2);
    // The inner width of the chat panel (minus the two border columns).
    let visible_width = chat_chunks[0].width.saturating_sub(2) as usize;

    // `Text::height()` only counts logical lines.  When `Wrap` is active,
    // every line whose display width exceeds `visible_width` occupies
    // *multiple* visual rows.  Using the raw logical count makes `max_scroll`
    // too small so the bottom of the history is unreachable.
    // We therefore sum up the wrapped row count for every logical line.
    let total_lines: u16 = if visible_width == 0 {
        history_text.height() as u16
    } else {
        history_text
            .lines
            .iter()
            .map(|line| {
                let line_width: usize = line
                    .spans
                    .iter()
                    .map(|s| s.content.chars().count())
                    .sum();
                if line_width == 0 {
                    1u16
                } else {
                    ((line_width + visible_width - 1) / visible_width) as u16
                }
            })
            .sum()
    };

    if app.autoscroll {
        app.scroll = total_lines.saturating_sub(visible_height);
    } else {
        let max_scroll = total_lines.saturating_sub(visible_height);
        if app.scroll > max_scroll {
            app.scroll = max_scroll;
        }
    }

    let scroll_status = if app.autoscroll {
        " [AUTOSCROLL] "
    } else {
        " [MANUAL SCROLL 🔒] "
    };
    f.render_widget(Clear, chat_chunks[0]);
    f.render_widget(
        Paragraph::new(history_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" Conversation History{} ", scroll_status))
                    .border_style(if !app.autoscroll {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    }),
            )
            .wrap(Wrap { trim: true })
            .scroll((app.scroll, 0)),
        chat_chunks[0],
    );

    // Spinner-Animation berechnen
    let spinner_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let frame_idx = (app.start_time.elapsed().as_millis() / 100) as usize % spinner_frames.len();
    let input_title = if app.is_loading {
        format!(" {} AI is thinking... ", spinner_frames[frame_idx])
    } else {
        " > Input ".into()
    };

    // Build multiline input text with selection and cursor support
    let input_chars: Vec<char> = app.input.chars().collect();
    let cursor_pos = app.cursor_pos.min(input_chars.len());
    let selection_range = app.get_selection_range();
    
    let cursor_style = Style::default().add_modifier(Modifier::REVERSED);
    let selection_style = Style::default()
        .bg(Color::Blue)
        .fg(Color::White);

    // Pre-calculate which line the cursor is on by counting newlines before cursor
    let cursor_line = input_chars.iter()
        .take(cursor_pos)
        .filter(|&&c| c == '\n')
        .count();
    
    // Build lines with proper selection and cursor handling
    let mut lines = Vec::new();
    let mut current_line_spans = Vec::new();
    let mut char_index = 0;

    for ch in input_chars.iter().chain(std::iter::once(&'\0')) {
        let is_newline = *ch == '\n';
        let is_end = *ch == '\0';
        
        if !is_newline && !is_end {
            // Determine style for this character
            let mut style = Style::default();
            let mut is_cursor = false;
            
            if char_index == cursor_pos && app.cursor_visible {
                style = cursor_style;
                is_cursor = true;
            }
            
            if let Some((sel_start, sel_end)) = selection_range {
                if char_index >= sel_start && char_index < sel_end {
                    style = selection_style;
                    if char_index == cursor_pos && app.cursor_visible {
                        // Cursor within selection - combine styles
                        style = selection_style.add_modifier(Modifier::REVERSED);
                    }
                }
            }
            
            if is_cursor || style.bg.is_some() || style.add_modifier != Modifier::empty() {
                current_line_spans.push(Span::styled(ch.to_string(), style));
            } else {
                current_line_spans.push(Span::raw(ch.to_string()));
            }
            
            char_index += 1;
        } else if is_newline {
            // End current line and start new one
            if current_line_spans.is_empty() {
                current_line_spans.push(Span::raw(" "));
            }
            lines.push(Line::from(current_line_spans.clone()));
            current_line_spans.clear();
            char_index += 1;
        } else {
            // End of input (\0 marker)
            // If cursor is at the end, ensure line is visible even when cursor blinks
            if char_index == cursor_pos {
                if app.cursor_visible {
                    current_line_spans.push(Span::styled(" ", cursor_style));
                } else {
                    // Placeholder space to keep empty line visible when cursor is invisible
                    current_line_spans.push(Span::raw(" "));
                }
            }
            
            // Add final line if it has content or if it's the first line
            if !current_line_spans.is_empty() {
                lines.push(Line::from(current_line_spans));
            } else if lines.is_empty() {
                // Ensure at least one line exists
                lines.push(Line::from(vec![Span::raw(" ")]));
            }
            break;
        }
    }

    // Calculate input scrolling to keep cursor visible  
    // Do NOT modify app state during rendering - only calculate local scroll offset
    let visible_input_height = input_height.saturating_sub(2) as usize; // Subtract borders
    let total_input_lines = lines.len();
    
    // Calculate the optimal scroll position based on cursor location
    let mut scroll_offset = app.input_scroll;
    
    if cursor_line < scroll_offset as usize {
        // Cursor is above visible area, scroll up
        scroll_offset = cursor_line as u16;
    } else if cursor_line >= (scroll_offset as usize + visible_input_height) {
        // Cursor is below visible area, scroll down
        scroll_offset = (cursor_line + 1).saturating_sub(visible_input_height) as u16;
    }
    
    // Clamp scroll to valid range
    let max_input_scroll = total_input_lines.saturating_sub(visible_input_height);
    scroll_offset = scroll_offset.min(max_input_scroll as u16);

    let input_text = Text::from(lines);

    f.render_widget(
        Paragraph::new(input_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(input_title)
                    .border_style(if app.is_loading {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    }),
            )
            .wrap(Wrap { trim: false })
            .scroll((scroll_offset, 0)),
        chat_chunks[1],
    );
    
    // Build status bar with dynamic keybindings from settings
    let kb = &app.settings.keybindings;
    let status = build_status_bar(app, kb, &selected_model, total_lines, visible_height);
    
    f.render_widget(
        Paragraph::new(status).style(Style::default().bg(Color::White).fg(Color::Black)),
        root_layout[2],
    );
    
    // Render version dialog if open
    if app.show_version_dialog {
        render_version_dialog(f, app);
    }
    
    // Render settings dialog if open
    if app.show_settings_dialog {
        render_settings_dialog(f, app);
    }
}

/// Builds the status bar text with dynamic keybinding display.
fn build_status_bar(app: &App, kb: &crate::app::KeyBindings, selected_model: &str, total_lines: u16, visible_height: u16) -> String {
    let mut status = format!(
        " {}: Quit | {}: Settings | {}: Copy | {}: Paste | {}: Newline | {}/{}: Scroll | {}: Model [{}] ",
        kb.quit.short_display(),
        kb.open_settings.short_display(),
        kb.copy.short_display(),
        kb.paste.short_display(),
        kb.insert_newline.short_display(),
        kb.page_up.short_display(),
        kb.page_down.short_display(),
        kb.select_next_model.short_display(),
        selected_model
    );
    if app.debug_keys {
        let max_scroll = total_lines.saturating_sub(visible_height);
        let last_key = app.debug_last_key.as_deref().unwrap_or("-");
        status.push_str(&format!(
            "| Scroll: {}/{} | Render: {} | Key: {} ",
            app.scroll, max_scroll, app.render_count, last_key
        ));
    }
    status
}

/// Returns the list of all selectable (theme and keybinding) indices for the settings dialog.
///
/// "selectable" means the user can highlight and interact with this item.
/// Headers, spacers, and footer text are non-selectable and skipped during navigation.
///
/// The returned vector contains the actual list indices of selectable items.
pub fn get_settings_selectable_indices(theme_count: usize, binding_count: usize) -> Vec<usize> {
    // Actual item layout:
    //   0                     -> "Syntax Highlighting Theme:" header (non-selectable)
    //   1 .. 1+theme_count-1  -> themes (selectable)
    //   1+theme_count         -> spacer (non-selectable)
    //   1+theme_count+1       -> "Keybindings (...)" header (non-selectable)
    //   1+theme_count+2 .. 1+theme_count+2+binding_count-1 -> keybindings (selectable)
    //   1+theme_count+2+binding_count     -> spacer (non-selectable)
    //   1+theme_count+2+binding_count+1   -> footer (non-selectable)
    
    let mut indices = Vec::new();
    // Theme selectable indices: 1 .. theme_count (inclusive)
    for i in 1..=theme_count {
        indices.push(i);
    }
    // Keybinding selectable indices: theme_count+3 .. theme_count+3+binding_count-1 (inclusive)
    let bind_start = theme_count + 3;
    for i in bind_start..bind_start + binding_count {
        indices.push(i);
    }
    indices
}

/// Renders the settings dialog popup.
///
/// This function displays a centered popup dialog that allows users to configure
/// application settings, including syntax highlighting theme and keybindings.
/// The dialog shows two sections: theme selection and keybinding configuration.
///
/// The dialog uses stateful rendering so the List widget handles scrolling
/// automatically as the user navigates.
///
/// # Arguments
///
/// * `f` - The frame to render into
/// * `app` - The application state containing current settings and selection
///
/// # Navigation
///
/// - Up/Down arrows: Navigate through themes and keybindings (skips headers)
/// - Enter: Apply selected theme or start keybinding recording
/// - Esc/q: Close dialog without changes
fn render_settings_dialog(f: &mut Frame, app: &App) {
    let area = f.area();
    
    // Create a centered popup area (70% width, 80% height)
    let popup_width = (area.width * 70) / 100;
    let popup_height = (area.height * 80) / 100;
    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;
    
    let popup_area = ratatui::layout::Rect {
        x: popup_x,
        y: popup_y,
        width: popup_width,
        height: popup_height,
    };
    
    // Clear the popup area first
    f.render_widget(Clear, popup_area);
    
    // Build the settings content
    let themes = crate::app::SyntaxTheme::all();
    let bindings = crate::app::KeyBindings::all();
    
    let mut items: Vec<ListItem> = Vec::new();
    
    // Add theme section header (index 0, non-selectable)
    items.push(ListItem::new(Line::from(vec![
        Span::styled("Syntax Highlighting Theme:", Style::default().fg(Color::Yellow)),
    ])));
    
    // Add themes (indices 1..1+theme_count, selectable)
    for (_idx, theme) in themes.iter().enumerate() {
        let is_current = theme == &app.settings.syntax_theme;
        
        let mut label = format!("  {}", theme.display_name());
        if is_current {
            label.push_str(" ✓");
        }
        
        let style = if is_current {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        };
        
        items.push(ListItem::new(label).style(style));
    }
    
    // Add spacer (non-selectable)
    items.push(ListItem::new(""));
    
    // Add keybinding section header (non-selectable)
    items.push(ListItem::new(Line::from(vec![
        Span::styled("Keybindings (Enter to rebind):", Style::default().fg(Color::Yellow)),
    ])));
    
    // Add keybindings (selectable)
    for (name, display_name) in &bindings {
        let is_recording = app.settings_recording_binding.as_deref() == Some(name);
        
        let current_kc = app.settings.keybindings.get(name)
            .map(|kc| kc.display())
            .unwrap_or_else(|| "?".to_string());
        
        let label = if is_recording {
            format!("  {}: [press key...]", display_name)
        } else {
            format!("  {}: {}", display_name, current_kc)
        };
        
        let style = if is_recording {
            Style::default().fg(Color::Yellow).bg(Color::Blue).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        
        items.push(ListItem::new(label).style(style));
    }
    
    // Add footer
    items.push(ListItem::new(""));
    items.push(ListItem::new(Line::from(vec![
        Span::styled("↑↓: Navigate  Enter: Apply/Record  Esc: Close", 
            Style::default().fg(Color::DarkGray)),
    ])));
    
    // Create a ListState with the current selection for stateful scrolling
    let mut list_state = ratatui::widgets::ListState::default();
    list_state.select(Some(app.settings_selection));
    
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Settings ")
        )
        .highlight_style(
            Style::default().fg(Color::Black).bg(Color::Cyan)
        );
    
    f.render_stateful_widget(list, popup_area, &mut list_state);
}

/// Renders the version update notification dialog.
///
/// This function displays a centered popup dialog when a newer version of
/// LazyLlama is available on GitHub. The dialog shows the current installed
/// version and the latest available version, along with a message prompting
/// the user to visit the releases page.
///
/// # Arguments
///
/// * `f` - The frame to render into
/// * `app` - The application state containing version information
///
/// # Navigation
///
/// - Esc: Close the dialog
fn render_version_dialog(f: &mut Frame, app: &App) {
    let area = f.area();
    
    // Create a centered popup area (50% width, ~40% height)
    let popup_width = (area.width * 50) / 100;
    let popup_height = 12; // Fixed height for the version dialog
    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;
    
    let popup_area = ratatui::layout::Rect {
        x: popup_x,
        y: popup_y,
        width: popup_width,
        height: popup_height,
    };
    
    // Clear the popup area first
    f.render_widget(Clear, popup_area);
    
    // Build the dialog content
    let latest_version = app.latest_version.as_deref().unwrap_or("?.?.?");
    let current_version = crate::app::VERSION;
    
    let mut lines: Vec<Line> = Vec::new();
    
    // Title line
    lines.push(Line::from(vec![
        Span::styled(
            "  🦙 Update Available!",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(""));
    
    // Version info
    lines.push(Line::from(vec![
        Span::raw("  Current version: "),
        Span::styled(
            current_version.to_string(),
            Style::default().fg(Color::Yellow),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::raw("  Latest version:  "),
        Span::styled(
            latest_version.to_string(),
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(""));
    
    // Message
    lines.push(Line::from(vec![
        Span::styled(
            "  A new version of LazyLlama is available!",
            Style::default().fg(Color::White),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled(
            "  Visit the GitHub releases page to download it.",
            Style::default().fg(Color::White),
        ),
    ]));
    lines.push(Line::from(""));
    
    // Close instruction
    lines.push(Line::from(vec![
        Span::styled(
            "  Press Esc to close this dialog",
            Style::default().fg(Color::DarkGray),
        ),
    ]));
    
    let text = Text::from(lines);
    
    f.render_widget(
        Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(" Update Checker "),
            ),
        popup_area,
    );
}

/// Parses conversation history and converts it into a formatted Ratatui Text object.
///
/// This function processes the raw conversation history string and applies syntax
/// highlighting for markdown elements, particularly code blocks. It uses regex
/// pattern matching to identify code blocks and delegates regular text processing
/// to [`process_styled_text`].
///
/// # Arguments
///
/// * `history` - The raw conversation history string containing user and AI messages
///
/// # Returns
///
/// A formatted [`Text`] object ready for rendering with Ratatui, containing:
/// - Syntax-highlighted code blocks with language-specific borders
/// - Styled user/AI message labels with appropriate colors
/// - Markdown formatting for headers and emphasis
///
/// # Code Block Processing
///
/// Code blocks are detected using the regex pattern:
/// ```regex
/// (?s)```(?P<lang>\w+)?\n(?P<code>.*?)```
/// ```
///
/// Each code block is rendered with:
/// - Language-specific header: `┌── rust ──`
/// - Yellow-colored borders and prefixes
/// - Preserved indentation and formatting
/// - Consistent visual separation from regular text
///
/// # Performance
///
/// Uses single-pass regex processing with efficient string slicing to minimize
/// allocations. The function handles large conversation histories gracefully
/// without significant performance degradation.
///
/// # Example Input/Output
///
/// ```text
/// Input: "YOU: Hello\n\nAI: Here's some code:\n\n```rust\nfn main() {}\n```"
/// Output: Formatted Text with colored labels and bordered code block
/// ```
pub fn parse_history<'a>(history: &'a str, theme_name: &str) -> Text<'a> {
    // Updated regex to allow optional whitespace after language name
    let code_block_re = Regex::new(r"(?s)```(?P<lang>\w+)?\s*\n(?P<code>.*?)```").unwrap();
    let mut text = Text::default();
    let mut last_match_end = 0;

    for caps in code_block_re.captures_iter(history) {
        let full_match = caps.get(0).unwrap();
        if full_match.start() > last_match_end {
            process_styled_text(&history[last_match_end..full_match.start()], &mut text);
        }
        let lang = caps.name("lang").map_or("code", |m| m.as_str());
        let code_content = caps.name("code").map_or("", |m| m.as_str());

        // Header with language name
        text.push_line(Line::from(Span::styled(
            format!(" ┌── {} ──", lang),
            Style::default().fg(Color::Yellow),
        )));
        
        // Syntax-highlighted code lines
        let highlighted_lines = highlight_code_block(code_content, lang, theme_name);
        for line in highlighted_lines {
            text.push_line(line);
        }
        
        // Footer
        text.push_line(Line::from(Span::styled(
            " └──────────",
            Style::default().fg(Color::Yellow),
        )));
        last_match_end = full_match.end();
    }
    if last_match_end < history.len() {
        process_styled_text(&history[last_match_end..], &mut text);
    }
    text
}

/// Processes regular text line-by-line and applies GitHub-flavored markdown styling.
///
/// This function handles comprehensive markdown formatting using pulldown-cmark parser,
/// supporting all major GitHub markdown features including inline formatting, lists,
/// blockquotes, headers, and special text markers like task lists.
///
/// # Arguments
///
/// * `text` - The raw text string to be processed and styled
/// * `target` - Mutable reference to the Text object where styled content is appended
///
/// # Supported Markdown Features
///
/// - **Inline Formatting**:
///   - Bold: `**text**` or `__text__`
///   - Italic: `*text*` or `_text_`
///   - Strikethrough: `~~text~~`
///   - Inline code: `` `code` ``
///   - Combined formatting: `***bold italic***`
///
/// - **Headers**: All levels `#` to `######`
///
/// - **Lists**:
///   - Unordered lists: `-`, `*`, `+`
///   - Ordered lists: `1.`, `2.`, etc.
///   - Task lists: `- [ ]` and `- [x]`
///
/// - **Blockquotes**: `> quote text`
///
/// - **Horizontal Rules**: `---`, `***`, `___`
///
/// - **Links**: `[text](url)` - displays the link text
///
/// # Special Labels
///
/// - `YOU:` prefix styled in bold magenta
/// - `AI:` prefix styled in bold cyan
/// 
/// # Color Scheme
///
/// - Headers: White with bold modifier (levels indicated by bullet style)
/// - Bold text: Bold modifier
/// - Italic text: Italic modifier  
/// - Strikethrough: Crossed-out modifier
/// - Inline code: Yellow foreground with dim modifier
/// - User labels: Magenta with bold modifier
/// - AI labels: Cyan with bold modifier
/// - Blockquotes: Green with italic modifier
/// - Links: Blue with underline modifier
/// - List items: Bullet points in appropriate colors
///
/// # Side Effects
///
/// Appends styled content directly to the provided `target` Text object,
/// allowing for incremental building of complex formatted documents.
pub fn process_styled_text<'a>(text: &'a str, target: &mut Text<'a>) {
    for line in text.lines() {
        // Check for special YOU:/AI: labels first
        if line.starts_with("YOU:") {
            let rest = if line.len() > 4 { &line[4..] } else { "" };
            let mut spans: Vec<Span<'static>> = vec![
                Span::styled(
                    "YOU:".to_string(),
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
            ];
            // Add the rest as-is (including leading space if present)
            if !rest.is_empty() {
                spans.push(Span::raw(rest.to_string()));
            }
            target.push_line(Line::from(spans));
            continue;
        } else if line.starts_with("AI:") {
            let rest = if line.len() > 3 { &line[3..] } else { "" };
            let mut spans: Vec<Span<'static>> = vec![
                Span::styled(
                    "AI: ".to_string(),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ];
            // Add the rest as-is (including leading space if present)
            if !rest.is_empty() {
                spans.push(Span::raw(rest.to_string()));
            }
            target.push_line(Line::from(spans));
            continue;
        }

        // Parse the line as markdown
        let mut spans: Vec<Span<'static>> = Vec::new();
        parse_line_markdown(line, &mut spans);
        target.push_line(Line::from(spans));
    }
}

/// Parses a single line of markdown and converts it to styled spans.
///
/// Handles block-level markdown elements like headers, lists, blockquotes,
/// and horizontal rules, as well as inline formatting.
fn parse_line_markdown(line: &str, spans: &mut Vec<Span<'static>>) {
    let trimmed = line.trim();
    
    // Check for horizontal rule
    if trimmed == "---" || trimmed == "***" || trimmed == "___" {
        spans.push(Span::styled(
            "─".repeat(40),
            Style::default().fg(Color::DarkGray),
        ));
        return;
    }

    // Check for headers (# to ######)
    if let Some(header_text) = trimmed.strip_prefix("######").map(|s| (6, s))
        .or_else(|| trimmed.strip_prefix("#####").map(|s| (5, s)))
        .or_else(|| trimmed.strip_prefix("####").map(|s| (4, s)))
        .or_else(|| trimmed.strip_prefix("###").map(|s| (3, s)))
        .or_else(|| trimmed.strip_prefix("##").map(|s| (2, s)))
        .or_else(|| trimmed.strip_prefix("#").map(|s| (1, s)))
    {
        let (_level, text) = header_text;
        // Use consistent bullet style for all headers to maintain compatibility
        let bullet = "● ";
        spans.push(Span::styled(
            format!("{}{}", bullet, text.trim()),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ));
        return;
    }

    // Check for blockquote
    if let Some(quote_text) = trimmed.strip_prefix(">") {
        spans.push(Span::styled(
            "│ ".to_string(),
            Style::default().fg(Color::Green),
        ));
        parse_inline_markdown(quote_text.trim(), spans, Style::default().fg(Color::Green).add_modifier(Modifier::ITALIC));
        return;
    }

    // Check for unordered list
    if let Some(list_text) = trimmed.strip_prefix("- ")
        .or_else(|| trimmed.strip_prefix("* "))
        .or_else(|| trimmed.strip_prefix("+ "))
    {
        // Check for task list
        if list_text.trim().starts_with("[ ]") {
            spans.push(Span::styled("☐ ".to_string(), Style::default().fg(Color::Yellow)));
            parse_inline_markdown(&list_text.trim()[3..].trim(), spans, Style::default());
        } else if list_text.trim().starts_with("[x]") || list_text.trim().starts_with("[X]") {
            spans.push(Span::styled("☑ ".to_string(), Style::default().fg(Color::Green)));
            parse_inline_markdown(&list_text.trim()[3..].trim(), spans, Style::default().add_modifier(Modifier::DIM));
        } else {
            spans.push(Span::styled("• ".to_string(), Style::default().fg(Color::Cyan)));
            parse_inline_markdown(list_text, spans, Style::default());
        }
        return;
    }

    // Check for ordered list
    if let Some(caps) = Regex::new(r"^(\d+)\.\s+(.*)$").unwrap().captures(trimmed) {
        let prefix = format!("{}. ", &caps[1]);
        let rest = caps[2].to_string();
        spans.push(Span::styled(
            prefix,
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ));
        parse_inline_markdown(&rest, spans, Style::default());
        return;
    }

    // Regular text with inline markdown
    parse_inline_markdown(line, spans, Style::default());
}

/// Parses inline markdown formatting within a text string.
///
/// Supports bold, italic, strikethrough, inline code, and links.
/// Uses pulldown-cmark for accurate CommonMark/GFM parsing.
/// Returns owned Span objects to avoid lifetime issues.
fn parse_inline_markdown(text: &str, spans: &mut Vec<Span<'static>>, base_style: Style) {
    let parser = Parser::new(text);
    let mut current_style = base_style;
    let mut style_stack = Vec::new();

    for event in parser {
        match event {
            Event::Text(t) => {
                spans.push(Span::styled(t.to_string(), current_style));
            }
            Event::Code(code) => {
                spans.push(Span::styled(
                    format!("`{}`", code),
                    base_style.fg(Color::Yellow).add_modifier(Modifier::DIM),
                ));
            }
            Event::Start(tag) => {
                style_stack.push(current_style);
                current_style = match tag {
                    Tag::Strong => current_style.add_modifier(Modifier::BOLD),
                    Tag::Emphasis => current_style.add_modifier(Modifier::ITALIC),
                    Tag::Strikethrough => current_style.add_modifier(Modifier::CROSSED_OUT),
                    Tag::Link { .. } => current_style.fg(Color::Blue).add_modifier(Modifier::UNDERLINED),
                    _ => current_style,
                };
            }
            Event::End(tag_end) => {
                if let Some(previous_style) = style_stack.pop() {
                    match tag_end {
                        TagEnd::Link => {
                            // For links, we've already shown the link text
                            // Skip the URL part as it's not useful in TUI
                        }
                        _ => {}
                    }
                    current_style = previous_style;
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                // Breaks within inline text are handled by line processing
            }
            _ => {
                // Handle other events if needed
            }
        }
    }
}