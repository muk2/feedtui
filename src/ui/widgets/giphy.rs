use crate::config::GiphyConfig;
use crate::feeds::giphy::{GiphyFetcher, GiphyMode};
use crate::feeds::{FeedData, FeedFetcher, GiphyGif};
use crate::ui::widgets::FeedWidget;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::time::Instant;

pub struct GiphyWidget {
    config: GiphyConfig,
    gif: Option<GiphyGif>,
    current_frame: usize,
    last_frame_time: Instant,
    loading: bool,
    paused: bool,
    error: Option<String>,
    selected: bool,
}

impl GiphyWidget {
    pub fn new(config: GiphyConfig) -> Self {
        Self {
            config,
            gif: None,
            current_frame: 0,
            last_frame_time: Instant::now(),
            loading: true,
            paused: false,
            error: None,
            selected: false,
        }
    }

    /// Advance the animation frame based on elapsed time
    pub fn tick(&mut self) {
        if self.paused {
            return;
        }

        let frame_delay_ms = self.config.frame_delay_ms.unwrap_or(150);

        if self.last_frame_time.elapsed().as_millis() >= frame_delay_ms as u128 {
            if let Some(ref gif) = self.gif {
                if !gif.frames.is_empty() {
                    self.current_frame = (self.current_frame + 1) % gif.frames.len();
                    self.last_frame_time = Instant::now();
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }
}

impl FeedWidget for GiphyWidget {
    fn id(&self) -> String {
        format!(
            "giphy-{}-{}",
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
            Style::default().fg(Color::Magenta)
        };

        let title = if let Some(ref gif) = self.gif {
            let status = if self.paused { " [PAUSED]" } else { "" };
            let frame_info = format!(" [{}/{}]", self.current_frame + 1, gif.frames.len());
            format!(
                " {} - {}{}{} ",
                self.config.title, gif.title, frame_info, status
            )
        } else {
            format!(" {} ", self.config.title)
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style);

        if self.loading && self.gif.is_none() {
            let loading_text = Paragraph::new("Loading GIF...").block(block);
            frame.render_widget(loading_text, area);
            return;
        }

        if let Some(ref error) = self.error {
            let error_text = Paragraph::new(format!("Error: {}", error)).block(block);
            frame.render_widget(error_text, area);
            return;
        }

        if let Some(ref gif) = self.gif {
            if gif.frames.is_empty() {
                let empty = Paragraph::new("No frames").block(block);
                frame.render_widget(empty, area);
                return;
            }

            let current = &gif.frames[self.current_frame];
            let inner_height = area.height.saturating_sub(2) as usize;

            let lines: Vec<Line> = current
                .iter()
                .take(inner_height)
                .map(|line| {
                    Line::from(Span::styled(
                        line.clone(),
                        Style::default().fg(Color::Green),
                    ))
                })
                .collect();

            let paragraph = Paragraph::new(lines).block(block);
            frame.render_widget(paragraph, area);
        } else {
            let empty = Paragraph::new("No GIF loaded").block(block);
            frame.render_widget(empty, area);
        }
    }

    fn update_data(&mut self, data: FeedData) {
        self.loading = false;
        match data {
            FeedData::Giphy(gif) => {
                self.gif = Some(gif);
                self.current_frame = 0;
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
        let mode = match self.config.mode.as_deref() {
            Some("trending") => GiphyMode::Trending,
            Some("random") => GiphyMode::Random,
            _ => GiphyMode::Search,
        };

        Box::new(GiphyFetcher::new(
            self.config.api_key.clone(),
            self.config.query.clone(),
            mode,
        ))
    }

    fn scroll_up(&mut self) {
        // Scroll backward through frames manually
        if let Some(ref gif) = self.gif {
            if !gif.frames.is_empty() {
                self.current_frame = if self.current_frame == 0 {
                    gif.frames.len() - 1
                } else {
                    self.current_frame - 1
                };
            }
        }
    }

    fn scroll_down(&mut self) {
        // Scroll forward through frames manually
        if let Some(ref gif) = self.gif {
            if !gif.frames.is_empty() {
                self.current_frame = (self.current_frame + 1) % gif.frames.len();
            }
        }
    }

    fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }

    fn get_selected_discussion_url(&self) -> Option<String> {
        None
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }
}
