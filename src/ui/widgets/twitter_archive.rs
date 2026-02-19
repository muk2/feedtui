use crate::config::TwitterArchiveConfig;
use crate::feeds::twitter_archive::TwitterArchiveFetcher;
use crate::feeds::{FeedData, FeedFetcher, TwitterArchiveItem};
use crate::ui::widgets::{FeedWidget, SelectedItem};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

pub struct TwitterArchiveWidget {
    config: TwitterArchiveConfig,
    items: Vec<TwitterArchiveItem>,
    loading: bool,
    error: Option<String>,
    scroll_state: ListState,
    selected: bool,
}

impl TwitterArchiveWidget {
    pub fn new(config: TwitterArchiveConfig) -> Self {
        let mut scroll_state = ListState::default();
        scroll_state.select(Some(0));

        Self {
            config,
            items: Vec::new(),
            loading: true,
            error: None,
            scroll_state,
            selected: false,
        }
    }
}

impl FeedWidget for TwitterArchiveWidget {
    fn id(&self) -> String {
        format!(
            "twitter_archive-{}-{}",
            self.config.position.row, self.config.position.col
        )
    }

    fn title(&self) -> &str {
        &self.config.title
    }

    fn position(&self) -> (usize, usize) {
        (self.config.position.row, self.config.position.col)
    }

    fn render(&self, frame: &mut Frame, area: Rect, selected: bool) {
        let border_style = if selected {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };

        let block = Block::default()
            .title(format!(" {} ", self.config.title))
            .borders(Borders::ALL)
            .border_style(border_style);

        if self.loading && self.items.is_empty() {
            let loading_text =
                List::new(vec![ListItem::new("Loading archived tweets...")]).block(block);
            frame.render_widget(loading_text, area);
            return;
        }

        if let Some(ref error) = self.error {
            let error_text =
                List::new(vec![ListItem::new(format!("Error: {}", error))]).block(block);
            frame.render_widget(error_text, area);
            return;
        }

        if self.items.is_empty() {
            let empty_text =
                List::new(vec![ListItem::new("No archived tweets found.")]).block(block);
            frame.render_widget(empty_text, area);
            return;
        }

        let items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                // First line: index + tweet preview or URL
                let preview = item.tweet_text.as_deref().unwrap_or(&item.original_url);
                let title_line = Line::from(vec![
                    Span::styled(format!("{}. ", i + 1), Style::default().fg(Color::DarkGray)),
                    Span::styled(preview, Style::default().fg(Color::White)),
                ]);

                // Second line: author + date
                let author_str = item.author.as_deref().unwrap_or("unknown");
                let meta_line = Line::from(vec![
                    Span::styled("   ", Style::default()),
                    Span::styled(author_str, Style::default().fg(Color::Cyan)),
                    Span::styled(
                        format!(" | {}", item.date_display),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]);

                ListItem::new(vec![title_line, meta_line])
            })
            .collect();

        let list = List::new(items).block(block).highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

        let mut state = self.scroll_state.clone();
        frame.render_stateful_widget(list, area, &mut state);
    }

    fn update_data(&mut self, data: FeedData) {
        self.loading = false;
        match data {
            FeedData::TwitterArchive(items) => {
                self.items = items;
                self.error = None;
            }
            FeedData::Error(e) => {
                self.error = Some(e);
            }
            FeedData::Loading => {
                self.loading = true;
            }
            _ => {}
        }
    }

    fn create_fetcher(&self) -> Box<dyn FeedFetcher> {
        Box::new(TwitterArchiveFetcher::new(
            self.config.archive_query.clone(),
            self.config.max_items,
        ))
    }

    fn scroll_up(&mut self) {
        if let Some(selected) = self.scroll_state.selected() {
            if selected > 0 {
                self.scroll_state.select(Some(selected - 1));
            }
        }
    }

    fn scroll_down(&mut self) {
        if let Some(selected) = self.scroll_state.selected() {
            if selected < self.items.len().saturating_sub(1) {
                self.scroll_state.select(Some(selected + 1));
            }
        }
    }

    fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }

    fn get_selected_item(&self) -> Option<SelectedItem> {
        let idx = self.scroll_state.selected()?;
        let item = self.items.get(idx)?;

        Some(SelectedItem {
            title: item
                .tweet_text
                .clone()
                .unwrap_or_else(|| item.original_url.clone()),
            url: Some(item.archive_url.clone()),
            description: item.tweet_text.clone(),
            source: item
                .author
                .clone()
                .unwrap_or_else(|| "Wayback Machine".to_string()),
            metadata: Some(item.date_display.clone()),
        })
    }

    fn get_selected_discussion_url(&self) -> Option<String> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Position, TwitterArchiveConfig};

    fn make_config() -> TwitterArchiveConfig {
        TwitterArchiveConfig {
            title: "Test Archive".to_string(),
            archive_query: "twitter.com/testuser*".to_string(),
            max_items: 10,
            position: Position { row: 0, col: 0 },
        }
    }

    fn make_item(idx: usize) -> TwitterArchiveItem {
        TwitterArchiveItem {
            timestamp: format!("2023061514302{}", idx),
            original_url: format!("https://twitter.com/testuser/status/{}", idx),
            archive_url: format!(
                "https://web.archive.org/web/2023061514302{}/https://twitter.com/testuser/status/{}",
                idx, idx
            ),
            tweet_text: Some(format!("Tweet number {}", idx)),
            author: Some("@testuser".to_string()),
            date_display: format!("2023-06-15 14:3{}", idx),
        }
    }

    #[test]
    fn test_widget_id() {
        let widget = TwitterArchiveWidget::new(make_config());
        assert_eq!(widget.id(), "twitter_archive-0-0");
    }

    #[test]
    fn test_widget_title() {
        let widget = TwitterArchiveWidget::new(make_config());
        assert_eq!(widget.title(), "Test Archive");
    }

    #[test]
    fn test_widget_position() {
        let widget = TwitterArchiveWidget::new(make_config());
        assert_eq!(widget.position(), (0, 0));
    }

    #[test]
    fn test_widget_initial_state() {
        let widget = TwitterArchiveWidget::new(make_config());
        assert!(widget.loading);
        assert!(widget.items.is_empty());
        assert!(widget.error.is_none());
    }

    #[test]
    fn test_update_data_with_items() {
        let mut widget = TwitterArchiveWidget::new(make_config());
        let items = vec![make_item(0), make_item(1), make_item(2)];

        widget.update_data(FeedData::TwitterArchive(items));

        assert!(!widget.loading);
        assert_eq!(widget.items.len(), 3);
        assert!(widget.error.is_none());
    }

    #[test]
    fn test_update_data_with_error() {
        let mut widget = TwitterArchiveWidget::new(make_config());
        widget.update_data(FeedData::Error("Network error".to_string()));

        assert!(!widget.loading);
        assert_eq!(widget.error, Some("Network error".to_string()));
    }

    #[test]
    fn test_update_data_loading() {
        let mut widget = TwitterArchiveWidget::new(make_config());
        widget.loading = false;
        widget.update_data(FeedData::Loading);
        assert!(widget.loading);
    }

    #[test]
    fn test_scroll_down() {
        let mut widget = TwitterArchiveWidget::new(make_config());
        let items = vec![make_item(0), make_item(1), make_item(2)];
        widget.update_data(FeedData::TwitterArchive(items));

        assert_eq!(widget.scroll_state.selected(), Some(0));
        widget.scroll_down();
        assert_eq!(widget.scroll_state.selected(), Some(1));
        widget.scroll_down();
        assert_eq!(widget.scroll_state.selected(), Some(2));
        // Should not go past the end
        widget.scroll_down();
        assert_eq!(widget.scroll_state.selected(), Some(2));
    }

    #[test]
    fn test_scroll_up() {
        let mut widget = TwitterArchiveWidget::new(make_config());
        let items = vec![make_item(0), make_item(1), make_item(2)];
        widget.update_data(FeedData::TwitterArchive(items));

        widget.scroll_down();
        widget.scroll_down();
        assert_eq!(widget.scroll_state.selected(), Some(2));

        widget.scroll_up();
        assert_eq!(widget.scroll_state.selected(), Some(1));
        widget.scroll_up();
        assert_eq!(widget.scroll_state.selected(), Some(0));
        // Should not go below 0
        widget.scroll_up();
        assert_eq!(widget.scroll_state.selected(), Some(0));
    }

    #[test]
    fn test_get_selected_item() {
        let mut widget = TwitterArchiveWidget::new(make_config());
        let items = vec![make_item(0), make_item(1)];
        widget.update_data(FeedData::TwitterArchive(items));

        let selected = widget.get_selected_item().unwrap();
        assert_eq!(selected.title, "Tweet number 0");
        assert!(selected.url.unwrap().contains("web.archive.org"));
        assert_eq!(selected.source, "@testuser");
    }

    #[test]
    fn test_get_selected_item_empty() {
        let widget = TwitterArchiveWidget::new(make_config());
        // Items are empty but scroll_state is at 0, so get(0) returns None
        assert!(widget.get_selected_item().is_none());
    }

    #[test]
    fn test_set_selected() {
        let mut widget = TwitterArchiveWidget::new(make_config());
        assert!(!widget.selected);
        widget.set_selected(true);
        assert!(widget.selected);
        widget.set_selected(false);
        assert!(!widget.selected);
    }

    #[test]
    fn test_get_selected_discussion_url() {
        let widget = TwitterArchiveWidget::new(make_config());
        assert!(widget.get_selected_discussion_url().is_none());
    }
}
