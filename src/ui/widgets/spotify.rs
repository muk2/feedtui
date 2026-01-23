use crate::config::SpotifyConfig;
use crate::feeds::spotify::SpotifyFetcher;
use crate::feeds::{FeedData, FeedFetcher, SpotifyPlayback};
use crate::ui::widgets::FeedWidget;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use std::any::Any;

pub struct SpotifyWidget {
    config: SpotifyConfig,
    playback: SpotifyPlayback,
    loading: bool,
    error: Option<String>,
    selected: bool,
}

impl SpotifyWidget {
    pub fn new(config: SpotifyConfig) -> Self {
        Self {
            config,
            playback: SpotifyPlayback::default(),
            loading: true,
            error: None,
            selected: false,
        }
    }

    pub fn get_fetcher(&self) -> SpotifyFetcher {
        SpotifyFetcher::new(
            self.config.client_id.clone(),
            self.config.client_secret.clone(),
            self.config.refresh_token.clone(),
        )
    }

    fn format_time(ms: u32) -> String {
        let seconds = ms / 1000;
        let minutes = seconds / 60;
        let seconds = seconds % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }
}

impl FeedWidget for SpotifyWidget {
    fn id(&self) -> String {
        format!("spotify_{}", self.config.title)
    }

    fn title(&self) -> &str {
        &self.config.title
    }

    fn position(&self) -> (usize, usize) {
        (self.config.position.row, self.config.position.col)
    }

    fn render(&self, frame: &mut Frame, area: Rect, selected: bool) {
        let block = Block::default()
            .title(self.config.title.clone())
            .borders(Borders::ALL)
            .border_style(if selected {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            });

        if let Some(error) = &self.error {
            let error_text = Paragraph::new(error.clone())
                .block(block)
                .style(Style::default().fg(Color::Red))
                .alignment(Alignment::Center);
            frame.render_widget(error_text, area);
            return;
        }

        if self.loading {
            let loading_text = Paragraph::new("Loading Spotify...")
                .block(block)
                .alignment(Alignment::Center);
            frame.render_widget(loading_text, area);
            return;
        }

        let mut lines = Vec::new();

        // Playback status icon
        let status_icon = if self.playback.is_playing {
            "▶ Playing"
        } else {
            "⏸ Paused"
        };
        lines.push(Line::from(vec![Span::styled(
            status_icon,
            Style::default()
                .fg(if self.playback.is_playing {
                    Color::Green
                } else {
                    Color::Yellow
                })
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(""));

        // Track information
        if let Some(track) = &self.playback.track_name {
            lines.push(Line::from(vec![
                Span::styled("Track: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    track,
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
        }

        if let Some(artist) = &self.playback.artist_name {
            lines.push(Line::from(vec![
                Span::styled("Artist: ", Style::default().fg(Color::DarkGray)),
                Span::styled(artist, Style::default().fg(Color::Cyan)),
            ]));
        }

        if let Some(album) = &self.playback.album_name {
            lines.push(Line::from(vec![
                Span::styled("Album: ", Style::default().fg(Color::DarkGray)),
                Span::styled(album, Style::default().fg(Color::Magenta)),
            ]));
        }

        // Progress bar
        if let (Some(progress), Some(duration)) =
            (self.playback.progress_ms, self.playback.duration_ms)
        {
            lines.push(Line::from(""));
            let progress_str = Self::format_time(progress);
            let duration_str = Self::format_time(duration);

            // Create a simple text-based progress indicator
            let bar_width = 30;
            let progress_ratio = progress as f64 / duration as f64;
            let filled = (bar_width as f64 * progress_ratio) as usize;
            let empty = bar_width - filled;

            let bar = format!("[{}{}]", "━".repeat(filled), "─".repeat(empty));

            lines.push(Line::from(vec![
                Span::styled(&progress_str, Style::default().fg(Color::DarkGray)),
                Span::styled(" ", Style::default()),
                Span::styled(&bar, Style::default().fg(Color::Green)),
                Span::styled(" ", Style::default()),
                Span::styled(&duration_str, Style::default().fg(Color::DarkGray)),
            ]));
        }

        // Controls hint
        if selected {
            lines.push(Line::from(""));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "Controls: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]));
            lines.push(Line::from(vec![
                Span::styled(
                    "Space",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" = Play/Pause  ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    "n",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" = Next  ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    "p",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" = Previous", Style::default().fg(Color::DarkGray)),
            ]));
        }

        let paragraph = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    fn update_data(&mut self, data: FeedData) {
        match data {
            FeedData::Spotify(playback) => {
                self.playback = playback;
                self.loading = false;
                self.error = None;
            }
            FeedData::Error(e) => {
                self.error = Some(e);
                self.loading = false;
            }
            FeedData::Loading => {
                self.loading = true;
                self.error = None;
            }
            _ => {}
        }
    }

    fn create_fetcher(&self) -> Box<dyn FeedFetcher> {
        Box::new(self.get_fetcher())
    }

    fn scroll_up(&mut self) {
        // No scrolling for Spotify widget
    }

    fn scroll_down(&mut self) {
        // No scrolling for Spotify widget
    }

    fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        Some(self)
    }
}
