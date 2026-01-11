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

//! Funktionen zur Gestaltung der Terminal-Benutzeroberfläche.
//!
//! Dieses Modul übernimmt das Rendering der Widgets, das Parsing von Markdown
//! und die Berechnung der Scroll-Position.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
use regex::Regex;
use crate::app::App;

/// Das ASCII-Banner, das oben in der Anwendung angezeigt wird.
pub const BANNER: &str = r#"
| |    __ _  ______  __| |    | | __ _ _ __ ___   __ _
| |   / _` ||_  /\ \/ /| |    | |/ _` | '_ ` _ \ / _` |
| |__| (_| | / /  \  / | |___ | | (_| | | | | | | (_| |
|_____\__,_|/___| /_/  |_____||_|\__,_|_| |_| |_|\__,_|
"#;

/// Haupt-Rendering-Funktion für Ratatui.
///
/// Zeichnet das Banner, die Modellliste, den Chatverlauf (mit Markdown-Support)
/// sowie das Eingabefeld inklusive Spinner.
pub fn ui(f: &mut Frame, app: &mut App) {
    let root_layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Length(7), Constraint::Min(0), Constraint::Length(1)])
    .split(f.size());

    f.render_widget(Paragraph::new(BANNER).style(Style::default().fg(Color::Cyan)).alignment(Alignment::Center), root_layout[0]);

    let main_chunks = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
    .split(root_layout[1]);

    // Modellliste rendern
    let items: Vec<ListItem> = app.models.iter().map(|m| ListItem::new(m.as_str())).collect();
    let list = List::new(items)
    .block(Block::default().borders(Borders::ALL).title(" Models "))
    .highlight_style(Style::default().bg(Color::Blue).add_modifier(Modifier::BOLD))
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
        if app.scroll > max_scroll { app.scroll = max_scroll; }
    }

    let scroll_status = if app.autoscroll { " [AUTOSCROLL] " } else { " [MANUAL SCROLL 🔒] " };
    f.render_widget(
        Paragraph::new(history_text)
        .block(Block::default().borders(Borders::ALL).title(format!(" Conversation History{} ", scroll_status))
        .border_style(if !app.autoscroll { Style::default().fg(Color::Yellow) } else { Style::default() }))
        .wrap(Wrap { trim: true })
        .scroll((app.scroll, 0)),
                    chat_chunks[0]
    );

    // Spinner-Animation berechnen
    let spinner_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let frame_idx = (app.start_time.elapsed().as_millis() / 100) as usize % spinner_frames.len();
    let input_title = if app.is_loading { format!(" {} AI is thinking... ", spinner_frames[frame_idx]) } else { " > Input ".into() };

    f.render_widget(Paragraph::new(app.input.as_str()).block(Block::default().borders(Borders::ALL).title(input_title).border_style(if app.is_loading { Style::default().fg(Color::Yellow) } else { Style::default() })), chat_chunks[1]);
    f.render_widget(Paragraph::new(" C-q: Quit | C-c: Clear | C-s: AutoScroll | PgUp/Dn: Scroll ").style(Style::default().bg(Color::White).fg(Color::Black)), root_layout[2]);
}

/// Parst den History-String und wandelt ihn in ein formatiertes Ratatui-[Text]-Objekt um.
/// Erkennt Code-Blöcke und delegiert einfachen Text an [`process_styled_text`].
fn parse_history<'a>(history: &'a str) -> Text<'a> {
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

        text.push_line(Line::from(Span::styled(format!(" ┌── {} ──", lang), Style::default().fg(Color::Yellow))));
        for line in code_content.lines() {
            text.push_line(Line::from(vec![Span::styled(" │ ", Style::default().fg(Color::Yellow)), Span::raw(line)]));
        }
        text.push_line(Line::from(Span::styled(" └──────────", Style::default().fg(Color::Yellow))));
        last_match_end = full_match.end();
    }
    if last_match_end < history.len() {
        process_styled_text(&history[last_match_end..], &mut text);
    }
    text
}

/// Verarbeitet normalen Text zeilenweise und wendet einfaches Styling für Labels und Markdown-Header an.
fn process_styled_text<'a>(text: &'a str, target: &mut Text<'a>) {
    for line in text.lines() {
        let trimmed = line.trim();
        let mut spans = Vec::new();
        if trimmed.starts_with("###") {
            spans.push(Span::styled(format!("● {}", trimmed.trim_start_matches('#').trim()), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)));
        } else if line.starts_with("YOU:") {
            spans.push(Span::styled("YOU:", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)));
            spans.push(Span::raw(&line[4..]));
        } else if line.starts_with("AI:") {
            spans.push(Span::styled("AI: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
            spans.push(Span::raw(&line[3..]));
        } else {
            spans.push(Span::raw(line));
        }
        target.push_line(Line::from(spans));
    }
}
