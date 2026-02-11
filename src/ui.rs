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
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
use regex::Regex;

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
/// │             │       Input Field         │ 3 lines
/// ├─────────────┴───────────────────────────┤
/// │            Status Bar                   │ 1 line
/// └─────────────────────────────────────────┘
/// ```
///
/// # Features
///
/// - **Model List**: Shows available AI models with status indicators
/// - **Chat History**: Displays conversation with markdown and code highlighting
/// - **Input Field**: Text entry with loading animation and status
/// - **Status Bar**: Keyboard shortcuts and current model information
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

    let chat_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(main_chunks[1]);

    // Verlauf parsen und Scrollen berechnen
    let history_text = parse_history(&app.history);
    let visible_height = chat_chunks[0].height.saturating_sub(2);
    let total_lines = history_text.height() as u16;

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

    let input_chars: Vec<char> = app.input.chars().collect();
    let cursor_pos = app.cursor_pos.min(input_chars.len());
    let mut input_spans = Vec::new();

    if cursor_pos > 0 {
        let before: String = input_chars[..cursor_pos].iter().collect();
        input_spans.push(Span::raw(before));
    }

    let cursor_style = Style::default().add_modifier(Modifier::REVERSED);
    if cursor_pos < input_chars.len() {
        let ch = input_chars[cursor_pos].to_string();
        if app.cursor_visible {
            input_spans.push(Span::styled(ch, cursor_style));
        } else {
            input_spans.push(Span::raw(ch));
        }

        if cursor_pos + 1 < input_chars.len() {
            let after: String = input_chars[cursor_pos + 1..].iter().collect();
            input_spans.push(Span::raw(after));
        }
    } else if app.cursor_visible {
        input_spans.push(Span::styled(" ", cursor_style));
    }

    let input_text = Text::from(Line::from(input_spans));

    f.render_widget(
        Paragraph::new(input_text).block(
            Block::default()
                .borders(Borders::ALL)
                .title(input_title)
                .border_style(if app.is_loading {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                }),
        ),
        chat_chunks[1],
    );
    let mut status = format!(
        " C-q: Quit | C-c: Clear | C-s: AutoScroll | PgUp/Dn: Scroll | ↑↓: Switch Model [{}] ",
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
    f.render_widget(
        Paragraph::new(status).style(Style::default().bg(Color::White).fg(Color::Black)),
        root_layout[2],
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
pub fn parse_history<'a>(history: &'a str) -> Text<'a> {
    let code_block_re = Regex::new(r"(?s)```(?P<lang>\w+)?\n(?P<code>.*?)```").unwrap();
    let mut text = Text::default();
    let mut last_match_end = 0;

    for caps in code_block_re.captures_iter(history) {
        let full_match = caps.get(0).unwrap();
        if full_match.start() > last_match_end {
            process_styled_text(&history[last_match_end..full_match.start()], &mut text);
        }
        let lang = caps.name("lang").map_or("code", |m| m.as_str());
        let code_content = caps.name("code").map_or("", |m| m.as_str());

        text.push_line(Line::from(Span::styled(
            format!(" ┌── {} ──", lang),
            Style::default().fg(Color::Yellow),
        )));
        for line in code_content.lines() {
            text.push_line(Line::from(vec![
                Span::styled(" │ ", Style::default().fg(Color::Yellow)),
                Span::raw(line),
            ]));
        }
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

/// Processes regular text line-by-line and applies styling for labels and markdown headers.
///
/// This function handles non-code text formatting, applying appropriate colors and
/// styles to different types of content including conversation labels, markdown
/// headers, and regular text. It preserves the original text structure while
/// adding visual styling.
///
/// # Arguments
///
/// * `text` - The raw text string to be processed and styled
/// * `target` - Mutable reference to the Text object where styled content is appended
///
/// # Styling Rules
///
/// - **Headers**: Lines starting with `###` are converted to bullet points (`•`) in bold white
/// - **User Messages**: "YOU:" prefix is styled in bold magenta, rest in default color
/// - **AI Messages**: "AI:" prefix is styled in bold cyan, rest in default color
/// - **Regular Text**: Rendered without special styling in default terminal colors
///
/// # Text Processing
///
/// The function processes each line individually and:
/// 1. Trims whitespace to detect line types
/// 2. Creates appropriate styled spans based on content
/// 3. Preserves original text after removing formatting markers
/// 4. Combines spans into cohesive line objects
/// 
/// # Color Scheme
///
/// - Headers: White with bold modifier
/// - User labels: Magenta with bold modifier
/// - AI labels: Cyan with bold modifier
/// - Regular text: Default terminal colors
///
/// # Side Effects
///
/// Appends styled content directly to the provided `target` Text object,
/// allowing for incremental building of complex formatted documents.
pub fn process_styled_text<'a>(text: &'a str, target: &mut Text<'a>) {
    for line in text.lines() {
        let trimmed = line.trim();
        let mut spans = Vec::new();
        if trimmed.starts_with("###") {
            spans.push(Span::styled(
                format!("● {}", trimmed.trim_start_matches('#').trim()),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ));
        } else if line.starts_with("YOU:") {
            spans.push(Span::styled(
                "YOU:",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::raw(&line[4..]));
        } else if line.starts_with("AI:") {
            spans.push(Span::styled(
                "AI: ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::raw(&line[3..]));
        } else {
            spans.push(Span::raw(line));
        }
        target.push_line(Line::from(spans));
    }
}


