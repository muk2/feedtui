use crate::config::SportsConfig;
use crate::feeds::sports::SportsFetcher;
use crate::feeds::{FeedData, FeedFetcher, SportsEvent};
use crate::ui::widgets::FeedWidget;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
};

pub struct SportsWidget {
    config: SportsConfig,
    events: Vec<SportsEvent>,
    loading: bool,
    error: Option<String>,
    scroll_state: ListState,
    selected: bool,
}

impl SportsWidget {
    pub fn new(config: SportsConfig) -> Self {
        let mut scroll_state = ListState::default();
        scroll_state.select(Some(0));

        Self {
            config,
            events: Vec::new(),
            loading: true,
            error: None,
            scroll_state,
            selected: false,
        }
    }
}

impl FeedWidget for SportsWidget {
    fn id(&self) -> String {
        format!(
            "sports-{}-{}",
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

        if self.loading && self.events.is_empty() {
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

        if self.events.is_empty() {
            let no_games = List::new(vec![ListItem::new("No games scheduled")]).block(block);
            frame.render_widget(no_games, area);
            return;
        }

        let items: Vec<ListItem> = self
            .events
            .iter()
            .map(|event| {
                let score_text = match (event.home_score, event.away_score) {
                    (Some(h), Some(a)) => format!("{} - {}", h, a),
                    _ => "vs".to_string(),
                };

                let status_color = match event.status.to_lowercase().as_str() {
                    s if s.contains("final") => Color::Gray,
                    s if s.contains("progress") || s.contains("half") || s.contains("quarter") => {
                        Color::Green
                    }
                    _ => Color::Yellow,
                };

                let game_line = Line::from(vec![
                    Span::styled(
                        format!("[{}] ", event.league),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(&event.away_team, Style::default().fg(Color::White)),
                    Span::styled(
                        format!(" {} ", score_text),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(&event.home_team, Style::default().fg(Color::White)),
                ]);

                let status_line = Line::from(vec![
                    Span::styled("      ", Style::default()),
                    Span::styled(&event.status, Style::default().fg(status_color)),
                ]);

                ListItem::new(vec![game_line, status_line])
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
            FeedData::Sports(events) => {
                self.events = events;
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
        Box::new(SportsFetcher::new(self.config.leagues.clone()))
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
            if selected < self.events.len().saturating_sub(1) {
                self.scroll_state.select(Some(selected + 1));
            }
        }
    }

    fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }
}
