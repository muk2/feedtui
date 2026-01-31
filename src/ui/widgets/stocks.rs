use crate::config::StocksConfig;
use crate::feeds::stocks::StocksFetcher;
use crate::feeds::{FeedData, FeedFetcher, StockQuote};
use crate::ui::widgets::FeedWidget;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

pub struct StocksWidget {
    config: StocksConfig,
    quotes: Vec<StockQuote>,
    loading: bool,
    error: Option<String>,
    scroll_state: ListState,
    selected: bool,
}

impl StocksWidget {
    pub fn new(config: StocksConfig) -> Self {
        let mut scroll_state = ListState::default();
        scroll_state.select(Some(0));

        Self {
            config,
            quotes: Vec::new(),
            loading: true,
            error: None,
            scroll_state,
            selected: false,
        }
    }
}

impl FeedWidget for StocksWidget {
    fn id(&self) -> String {
        format!(
            "stocks-{}-{}",
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

        if self.loading && self.quotes.is_empty() {
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
            .quotes
            .iter()
            .map(|quote| {
                let change_color = if quote.change >= 0.0 {
                    Color::Green
                } else {
                    Color::Red
                };

                let change_symbol = if quote.change >= 0.0 { "+" } else { "" };

                let symbol_line = Line::from(vec![
                    Span::styled(
                        format!("{:<6}", quote.symbol),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(" ${:.2}", quote.price),
                        Style::default().fg(Color::White),
                    ),
                ]);

                let change_line = Line::from(vec![Span::styled(
                    format!(
                        "      {}{:.2} ({}{:.2}%)",
                        change_symbol, quote.change, change_symbol, quote.change_percent
                    ),
                    Style::default().fg(change_color),
                )]);

                ListItem::new(vec![symbol_line, change_line])
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
            FeedData::Stocks(quotes) => {
                self.quotes = quotes;
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
        Box::new(StocksFetcher::new(self.config.symbols.clone()))
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
            if selected < self.quotes.len().saturating_sub(1) {
                self.scroll_state.select(Some(selected + 1));
            }
        }
    }

    fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }

    fn get_selected_discussion_url(&self) -> Option<String> {
        None
    }
}
