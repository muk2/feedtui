use crate::config::RssConfig;
use crate::feeds::rss::RssFetcher;
use crate::feeds::{FeedData, FeedFetcher, RssItem};
use crate::ui::widgets::{FeedWidget, SelectedItem};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

pub struct RssWidget {
    config: RssConfig,
    items: Vec<RssItem>,
    loading: bool,
    error: Option<String>,
    scroll_state: ListState,
    selected: bool,
}

impl RssWidget {
    pub fn new(config: RssConfig) -> Self {
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

impl FeedWidget for RssWidget {
    fn id(&self) -> String {
        format!(
            "rss-{}-{}",
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
            let loading_text = List::new(vec![ListItem::new("Loading...")]).block(block);
            frame.render_widget(loading_text, area);
            return;
        }

        if let Some(ref error) = self.error {
            let error_text =
                List::new(vec![ListItem::new(format!("Error: {}", error))]).block(block);
            frame.render_widget(error_text, area);
            return;
        }

        let items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let title_line = Line::from(vec![
                    Span::styled(format!("{}. ", i + 1), Style::default().fg(Color::DarkGray)),
                    Span::styled(&item.title, Style::default().fg(Color::White)),
                ]);

                let meta_parts: Vec<Span> = vec![
                    Span::styled("   ", Style::default()),
                    Span::styled(&item.source, Style::default().fg(Color::Cyan)),
                    Span::styled(
                        item.published
                            .as_ref()
                            .map(|d| format!(" | {}", d))
                            .unwrap_or_default(),
                        Style::default().fg(Color::DarkGray),
                    ),
                ];

                let meta_line = Line::from(meta_parts);

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
            FeedData::Rss(items) => {
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
        Box::new(RssFetcher::new(
            self.config.feeds.clone(),
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
            title: item.title.clone(),
            url: item.link.clone(),
            description: item.description.clone(),
            source: item.source.clone(),
            metadata: item.published.clone(),
        })
    }

    fn get_selected_discussion_url(&self) -> Option<String> {
        None
    }
}
