use crate::ui::widgets::SelectedItem;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
};

/// Article reader overlay for viewing feed content in the terminal
pub struct ArticleReader {
    pub visible: bool,
    item: Option<SelectedItem>,
    scroll_offset: u16,
    content_height: u16,
}

impl Default for ArticleReader {
    fn default() -> Self {
        Self {
            visible: false,
            item: None,
            scroll_offset: 0,
            content_height: 0,
        }
    }
}

impl ArticleReader {
    /// Show the article reader with the given item
    pub fn show(&mut self, item: SelectedItem) {
        self.item = Some(item);
        self.scroll_offset = 0;
        self.visible = true;
    }

    /// Hide the article reader
    pub fn hide(&mut self) {
        self.visible = false;
        self.item = None;
        self.scroll_offset = 0;
    }

    /// Toggle visibility
    pub fn toggle(&mut self) {
        if self.visible {
            self.hide();
        }
    }

    /// Scroll up
    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    /// Scroll down
    pub fn scroll_down(&mut self) {
        if self.scroll_offset < self.content_height.saturating_sub(1) {
            self.scroll_offset += 1;
        }
    }

    /// Page up
    pub fn page_up(&mut self, page_size: u16) {
        self.scroll_offset = self.scroll_offset.saturating_sub(page_size);
    }

    /// Page down
    pub fn page_down(&mut self, page_size: u16) {
        let max_scroll = self.content_height.saturating_sub(1);
        self.scroll_offset = (self.scroll_offset + page_size).min(max_scroll);
    }

    /// Get the current item's URL
    pub fn get_url(&self) -> Option<&str> {
        self.item.as_ref().and_then(|i| i.url.as_deref())
    }

    /// Render the article reader as an overlay
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        let Some(item) = &self.item else {
            return;
        };

        // Create a centered popup area (80% width, 80% height)
        let popup_area = centered_rect(80, 85, area);

        // Clear the background
        frame.render_widget(Clear, popup_area);

        // Create the main block
        let block = Block::default()
            .title(format!(" {} ", item.title))
            .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));

        let inner = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        // Build content lines
        let mut lines: Vec<Line> = Vec::new();

        // Source and metadata
        lines.push(Line::from(vec![
            Span::styled("Source: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&item.source, Style::default().fg(Color::Cyan)),
        ]));

        if let Some(ref metadata) = item.metadata {
            lines.push(Line::from(vec![
                Span::styled("Info: ", Style::default().fg(Color::DarkGray)),
                Span::styled(metadata, Style::default().fg(Color::Green)),
            ]));
        }

        if let Some(ref url) = item.url {
            lines.push(Line::from(vec![
                Span::styled("URL: ", Style::default().fg(Color::DarkGray)),
                Span::styled(url, Style::default().fg(Color::Blue).add_modifier(Modifier::UNDERLINED)),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(
                "─".repeat(inner.width.saturating_sub(2) as usize),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
        lines.push(Line::from(""));

        // Description/content
        if let Some(ref description) = item.description {
            // Strip HTML tags for cleaner display
            let clean_text = strip_html_tags(description);
            for line in clean_text.lines() {
                if !line.trim().is_empty() {
                    lines.push(Line::from(Span::styled(
                        line.to_string(),
                        Style::default().fg(Color::White),
                    )));
                }
            }
        } else {
            lines.push(Line::from(Span::styled(
                "No description available.",
                Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Press 'o' to open in browser for full content.",
                Style::default().fg(Color::Yellow),
            )));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(
                "─".repeat(inner.width.saturating_sub(2) as usize),
                Style::default().fg(Color::DarkGray),
            ),
        ]));

        // Help text
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("[Esc/q] ", Style::default().fg(Color::Yellow)),
            Span::styled("Close  ", Style::default().fg(Color::DarkGray)),
            Span::styled("[o] ", Style::default().fg(Color::Yellow)),
            Span::styled("Open in browser  ", Style::default().fg(Color::DarkGray)),
            Span::styled("[j/k or arrows] ", Style::default().fg(Color::Yellow)),
            Span::styled("Scroll", Style::default().fg(Color::DarkGray)),
        ]));

        // Update content height for scrolling
        self.content_height = lines.len() as u16;

        // Create scrollable paragraph
        let paragraph = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((self.scroll_offset, 0));

        // Split inner area for content and scrollbar
        let content_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(inner);

        frame.render_widget(paragraph, content_layout[0]);

        // Render scrollbar if content exceeds viewport
        if self.content_height > inner.height {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));

            let mut scrollbar_state = ScrollbarState::new(self.content_height as usize)
                .position(self.scroll_offset as usize);

            frame.render_stateful_widget(scrollbar, content_layout[1], &mut scrollbar_state);
        }
    }
}

/// Create a centered rectangle with given percentage of width and height
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Simple HTML tag stripping
fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    let mut in_entity = false;
    let mut entity = String::new();

    for ch in html.chars() {
        if ch == '<' {
            in_tag = true;
        } else if ch == '>' {
            in_tag = false;
        } else if ch == '&' && !in_tag {
            in_entity = true;
            entity.clear();
        } else if ch == ';' && in_entity {
            in_entity = false;
            // Convert common HTML entities
            match entity.as_str() {
                "amp" => result.push('&'),
                "lt" => result.push('<'),
                "gt" => result.push('>'),
                "quot" => result.push('"'),
                "apos" => result.push('\''),
                "nbsp" => result.push(' '),
                "#39" => result.push('\''),
                _ => {
                    // Try numeric entities
                    if entity.starts_with('#') {
                        if let Ok(code) = entity[1..].parse::<u32>() {
                            if let Some(c) = char::from_u32(code) {
                                result.push(c);
                            }
                        }
                    }
                }
            }
            entity.clear();
        } else if in_entity {
            entity.push(ch);
        } else if !in_tag {
            result.push(ch);
        }
    }

    // Clean up multiple whitespace
    let mut clean = String::new();
    let mut last_was_space = false;
    for ch in result.chars() {
        if ch.is_whitespace() {
            if !last_was_space {
                clean.push(if ch == '\n' { '\n' } else { ' ' });
                last_was_space = true;
            }
        } else {
            clean.push(ch);
            last_was_space = false;
        }
    }

    clean.trim().to_string()
}
