use crate::config::HackernewsConfig;
use crate::feeds::hackernews::HnFetcher;
use crate::feeds::{FeedData, FeedFetcher, HnStory};
use crate::ui::widgets::FeedWidget;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

pub struct HackernewsWidget {
    config: HackernewsConfig,
    stories: Vec<HnStory>,
    loading: bool,
    error: Option<String>,
    scroll_state: ListState,
    selected: bool,
}

impl HackernewsWidget {
    pub fn new(config: HackernewsConfig) -> Self {
        let mut scroll_state = ListState::default();
        scroll_state.select(Some(0));

        Self {
            config,
            stories: Vec::new(),
            loading: true,
            error: None,
            scroll_state,
            selected: false,
        }
    }
}

impl FeedWidget for HackernewsWidget {
    fn id(&self) -> String {
        format!(
            "hackernews-{}-{}",
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

        if self.loading && self.stories.is_empty() {
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
            .stories
            .iter()
            .enumerate()
            .map(|(i, story)| {
                let title_line = Line::from(vec![
                    Span::styled(format!("{}. ", i + 1), Style::default().fg(Color::DarkGray)),
                    Span::styled(&story.title, Style::default().fg(Color::White)),
                ]);

                let meta_line = Line::from(vec![
                    Span::styled(
                        format!("   {} pts | ", story.score),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::styled(
                        format!("{} comments | ", story.descendants),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(
                        format!("by {}", story.by),
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
            FeedData::HackerNews(stories) => {
                self.stories = stories;
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
        Box::new(HnFetcher::new(
            self.config.story_type.clone(),
            self.config.story_count,
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
            if selected < self.stories.len().saturating_sub(1) {
                self.scroll_state.select(Some(selected + 1));
            }
        }
    }

    fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }
}
