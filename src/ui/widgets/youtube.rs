use crate::config::YoutubeConfig;
use crate::feeds::youtube::YoutubeFetcher;
use crate::feeds::{FeedData, FeedFetcher, YoutubeVideo};
use crate::ui::widgets::FeedWidget;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

pub struct YoutubeWidget {
    config: YoutubeConfig,
    videos: Vec<YoutubeVideo>,
    loading: bool,
    error: Option<String>,
    scroll_state: ListState,
    selected: bool,
}

impl YoutubeWidget {
    pub fn new(config: YoutubeConfig) -> Self {
        let mut scroll_state = ListState::default();
        scroll_state.select(Some(0));

        Self {
            config,
            videos: Vec::new(),
            loading: true,
            error: None,
            scroll_state,
            selected: false,
        }
    }
}

impl FeedWidget for YoutubeWidget {
    fn id(&self) -> String {
        format!(
            "youtube-{}-{}",
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

        if self.loading && self.videos.is_empty() {
            let loading_text =
                List::new(vec![ListItem::new("Loading YouTube videos...")]).block(block);
            frame.render_widget(loading_text, area);
            return;
        }

        if let Some(ref error) = self.error {
            let error_text =
                List::new(vec![ListItem::new(format!("Error: {}", error))]).block(block);
            frame.render_widget(error_text, area);
            return;
        }

        if self.videos.is_empty() {
            let empty_text = List::new(vec![ListItem::new("No videos found")]).block(block);
            frame.render_widget(empty_text, area);
            return;
        }

        let items: Vec<ListItem> = self
            .videos
            .iter()
            .enumerate()
            .map(|(i, video)| {
                // Title line with numbering
                let title_line = Line::from(vec![
                    Span::styled(format!("{}. ", i + 1), Style::default().fg(Color::DarkGray)),
                    Span::styled(&video.title, Style::default().fg(Color::White)),
                ]);

                // Metadata line: channel, date, views, duration
                let mut meta_parts: Vec<Span> = vec![
                    Span::styled("   ", Style::default()),
                    Span::styled(&video.channel, Style::default().fg(Color::Cyan)),
                ];

                if let Some(ref views) = video.view_count {
                    meta_parts.push(Span::styled(
                        format!(" | {}", views),
                        Style::default().fg(Color::Green),
                    ));
                }

                if let Some(ref duration) = video.duration {
                    meta_parts.push(Span::styled(
                        format!(" | {}", duration),
                        Style::default().fg(Color::Magenta),
                    ));
                }

                meta_parts.push(Span::styled(
                    format!(" | {}", video.published),
                    Style::default().fg(Color::DarkGray),
                ));

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
            FeedData::Youtube(videos) => {
                self.videos = videos;
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
        Box::new(YoutubeFetcher::new(
            self.config.api_key.clone(),
            self.config.channels.clone(),
            self.config.search_query.clone(),
            self.config.max_videos,
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
            if selected < self.videos.len().saturating_sub(1) {
                self.scroll_state.select(Some(selected + 1));
            }
        }
    }

    fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }

    fn get_selected_url(&self) -> Option<String> {
        self.scroll_state
            .selected()
            .and_then(|idx| self.videos.get(idx))
            .map(|video| format!("https://www.youtube.com/watch?v={}", video.id))
    }
}
