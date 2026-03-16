use crate::config::TwitterConfig;
use crate::feeds::{FeedData, FeedFetcher};
use crate::ui::widgets::FeedWidget;
use async_trait::async_trait;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};
use std::any::Any;
use std::process::Stdio;
use std::time::Instant;
use tokio::process::Command;

#[derive(Debug, Clone)]
pub struct TwitterWidget {
    id: String,
    title: String,
    position: (usize, usize),
    selected: bool,
    tweets: Vec<Tweet>,
    selected_index: usize,
    list_state: ListState,
    mode: TwitterMode,
    compose_text: String,
    search_query: String,
    detail_view: Option<TweetDetail>,
    status_message: Option<(String, Instant)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TwitterMode {
    Normal,
    Compose,
    Reply,
    Search,
}

// Re-export Tweet from twitter_message for backwards compatibility
pub use crate::twitter_message::Tweet;

#[derive(Debug, Clone)]
struct TweetDetail {
    content: String,
    scroll_offset: u16,
}

impl TwitterWidget {
    pub fn new(config: TwitterConfig) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            id: format!("twitter-{}-{}", config.position.row, config.position.col),
            title: config.title,
            position: (config.position.row, config.position.col),
            selected: false,
            tweets: Vec::new(),
            selected_index: 0,
            list_state,
            mode: TwitterMode::Normal,
            compose_text: String::new(),
            search_query: String::new(),
            detail_view: None,
            status_message: None,
        }
    }

    pub fn open_compose(&mut self) {
        self.mode = TwitterMode::Compose;
        self.compose_text.clear();
    }

    pub fn open_reply(&mut self) {
        if !self.tweets.is_empty() {
            self.mode = TwitterMode::Reply;
            self.compose_text.clear();
        }
    }

    pub fn open_search(&mut self) {
        self.mode = TwitterMode::Search;
        self.search_query.clear();
    }

    pub fn close_modal(&mut self) {
        self.mode = TwitterMode::Normal;
        self.compose_text.clear();
        self.search_query.clear();
    }

    pub fn add_char(&mut self, c: char) {
        match self.mode {
            TwitterMode::Compose | TwitterMode::Reply => {
                self.compose_text.push(c);
            }
            TwitterMode::Search => {
                self.search_query.push(c);
            }
            _ => {}
        }
    }

    pub fn delete_char(&mut self) {
        match self.mode {
            TwitterMode::Compose | TwitterMode::Reply => {
                self.compose_text.pop();
            }
            TwitterMode::Search => {
                self.search_query.pop();
            }
            _ => {}
        }
    }

    pub fn close_detail_view(&mut self) {
        self.detail_view = None;
    }

    pub fn clear_tweets(&mut self) {
        self.tweets.clear();
        self.selected_index = 0;
        self.list_state.select(Some(0));
        self.detail_view = None;
    }

    pub fn has_tweets(&self) -> bool {
        !self.tweets.is_empty()
    }

    pub fn has_detail_view(&self) -> bool {
        self.detail_view.is_some()
    }

    pub fn scroll_detail_down(&mut self) {
        if let Some(detail) = &mut self.detail_view {
            detail.scroll_offset = detail.scroll_offset.saturating_add(1);
        }
    }

    pub fn scroll_detail_up(&mut self) {
        if let Some(detail) = &mut self.detail_view {
            detail.scroll_offset = detail.scroll_offset.saturating_sub(1);
        }
    }

    pub async fn execute_bird_command_static(args: &[&str]) -> anyhow::Result<String> {
        // Check for environment variables
        let ct0 = std::env::var("CT0").map_err(|_| {
            anyhow::anyhow!("Missing CT0 environment variable. Export it before running feedtui.")
        })?;
        let auth_token = std::env::var("AUTH_TOKEN").map_err(|_| {
            anyhow::anyhow!(
                "Missing AUTH_TOKEN environment variable. Export it before running feedtui."
            )
        })?;

        // Build command with explicit auth flags
        let output = Command::new("bird")
            .arg("--auth-token")
            .arg(&auth_token)
            .arg("--ct0")
            .arg(&ct0)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    anyhow::anyhow!(
                        "Bird CLI not found. Install with: bun install -g @steipete/bird"
                    )
                } else {
                    anyhow::anyhow!("Failed to run bird: {}", e)
                }
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        // Filter out Node.js ExperimentalWarning lines from stderr
        let filtered_stderr: String = stderr
            .lines()
            .filter(|line| !line.contains("ExperimentalWarning"))
            .collect::<Vec<_>>()
            .join("\n");
        let filtered_stderr = filtered_stderr.trim();

        if output.status.success() {
            Ok(stdout)
        } else {
            let msg = if filtered_stderr.is_empty() {
                stdout
            } else {
                filtered_stderr.to_string()
            };
            Err(anyhow::anyhow!("Bird command failed: {}", msg.trim()))
        }
    }

    pub fn is_modal_open(&self) -> bool {
        self.mode != TwitterMode::Normal || self.detail_view.is_some()
    }

    pub fn get_mode(&self) -> TwitterMode {
        self.mode.clone()
    }

    pub fn get_compose_text(&self) -> &str {
        &self.compose_text
    }

    pub fn get_search_query(&self) -> &str {
        &self.search_query
    }

    pub fn get_selected_tweet_url(&self) -> Option<String> {
        self.tweets
            .get(self.selected_index)
            .and_then(|t| t.url.clone())
    }

    fn set_status(&mut self, msg: String) {
        self.status_message = Some((msg, Instant::now()));
    }

    /// Clear status message after 5 seconds
    pub fn clear_expired_status(&mut self) {
        if let Some((_, time)) = &self.status_message {
            if time.elapsed().as_secs() >= 5 {
                self.status_message = None;
            }
        }
    }

    pub fn handle_async_result(&mut self, data: crate::twitter_message::TwitterData) {
        use crate::twitter_message::TwitterData;

        match data {
            TwitterData::TweetPosted(msg) => {
                self.set_status(format!("Tweet posted: {}", msg.trim()));
                self.close_modal();
            }
            TwitterData::ReplyPosted(msg) => {
                self.set_status(format!("Reply posted: {}", msg.trim()));
                self.close_modal();
            }
            TwitterData::SearchResults(tweets) => {
                if tweets.is_empty() {
                    self.set_status("No results found".into());
                } else {
                    self.set_status(format!("Found {} tweets", tweets.len()));
                }
                self.tweets = tweets;
                self.selected_index = 0;
                if !self.tweets.is_empty() {
                    self.list_state.select(Some(0));
                }
                self.close_modal();
            }
            TwitterData::Mentions(tweets) => {
                if tweets.is_empty() {
                    self.set_status("No mentions found".into());
                } else {
                    self.set_status(format!("Loaded {} mentions", tweets.len()));
                }
                self.tweets = tweets;
                self.selected_index = 0;
                if !self.tweets.is_empty() {
                    self.list_state.select(Some(0));
                }
            }
            TwitterData::TweetDetail(content) => {
                self.detail_view = Some(TweetDetail {
                    content,
                    scroll_offset: 0,
                });
            }
            TwitterData::Error(e) => {
                self.set_status(format!("Error: {}", e));
                self.close_modal();
            }
        }
    }
}

struct TwitterFetcher;

#[async_trait]
impl FeedFetcher for TwitterFetcher {
    async fn fetch(&self) -> anyhow::Result<FeedData> {
        // Twitter widget doesn't auto-fetch
        Ok(FeedData::Loading)
    }
}

impl FeedWidget for TwitterWidget {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn position(&self) -> (usize, usize) {
        self.position
    }

    fn render(&self, frame: &mut Frame, area: Rect, selected: bool) {
        let border_style = if selected {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::Gray)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(self.title.as_str());

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Render tweet list
        if self.tweets.is_empty() {
            let help_text = vec![
                Line::from(""),
                Line::from(Span::styled(
                    "Twitter/X Feed",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from("Keybindings:"),
                Line::from("  t - Compose tweet"),
                Line::from("  r - Reply to selected"),
                Line::from("  / - Search"),
                Line::from("  m - Load mentions"),
                Line::from("  Enter - Read tweet/thread"),
                Line::from(""),
                Line::from(Span::styled(
                    "Note: Requires Bird CLI and auth",
                    Style::default().fg(Color::Yellow),
                )),
            ];
            let paragraph = Paragraph::new(help_text).alignment(Alignment::Center);
            frame.render_widget(paragraph, inner);
        } else {
            let max_text_width = inner.width.saturating_sub(2) as usize;
            let items: Vec<ListItem> = self
                .tweets
                .iter()
                .enumerate()
                .map(|(idx, tweet)| {
                    let style = if idx == self.selected_index {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    let prefix = format!("@{}: ", tweet.author);
                    let remaining = max_text_width.saturating_sub(prefix.len());
                    let text = if tweet.text.len() > remaining {
                        format!("{}...", &tweet.text[..remaining.saturating_sub(3)])
                    } else {
                        tweet.text.clone()
                    };
                    ListItem::new(Line::from(vec![
                        Span::styled(prefix, style.add_modifier(Modifier::BOLD)),
                        Span::styled(text, style),
                    ]))
                })
                .collect();

            let list = List::new(items).block(Block::default());
            frame.render_widget(list, inner);
        }

        // Render modals if open
        if self.mode == TwitterMode::Compose {
            self.render_compose_modal(frame, area);
        } else if self.mode == TwitterMode::Reply {
            self.render_reply_modal(frame, area);
        } else if self.mode == TwitterMode::Search {
            self.render_search_modal(frame, area);
        }

        // Render detail view if open
        if let Some(detail) = &self.detail_view {
            self.render_detail_view(frame, area, detail);
        }

        // Render status message if present and not expired
        if let Some((msg, time)) = &self.status_message {
            if time.elapsed().as_secs() < 5 {
                self.render_status(frame, area, msg);
            }
        }
    }

    fn update_data(&mut self, _data: FeedData) {
        // Twitter widget doesn't use standard feed data
    }

    fn create_fetcher(&self) -> Box<dyn FeedFetcher> {
        Box::new(TwitterFetcher)
    }

    fn scroll_up(&mut self) {
        if !self.tweets.is_empty() && self.selected_index > 0 {
            self.selected_index -= 1;
            self.list_state.select(Some(self.selected_index));
        }
    }

    fn scroll_down(&mut self) {
        if !self.tweets.is_empty() && self.selected_index < self.tweets.len() - 1 {
            self.selected_index += 1;
            self.list_state.select(Some(self.selected_index));
        }
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

    fn get_selected_discussion_url(&self) -> Option<String> {
        self.tweets
            .get(self.selected_index)
            .and_then(|t| t.url.clone())
    }
}

impl TwitterWidget {
    fn render_compose_modal(&self, frame: &mut Frame, area: Rect) {
        let modal_area = self.centered_modal(area, 50, 10);
        frame.render_widget(Clear, modal_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title("Compose Tweet");

        let inner = block.inner(modal_area);
        frame.render_widget(block, modal_area);

        let display_text = if self.compose_text.is_empty() {
            "Type your tweet...".to_string()
        } else {
            self.compose_text.clone()
        };

        let text_style = if self.compose_text.is_empty() {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::White)
        };

        let text = vec![
            Line::from(Span::styled(display_text, text_style)),
            Line::from(""),
            Line::from(Span::styled(
                "Enter to post | Esc to cancel",
                Style::default().fg(Color::DarkGray),
            )),
        ];

        let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });
        frame.render_widget(paragraph, inner);
    }

    fn render_reply_modal(&self, frame: &mut Frame, area: Rect) {
        let modal_area = self.centered_modal(area, 50, 12);
        frame.render_widget(Clear, modal_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title("Reply to Tweet");

        let inner = block.inner(modal_area);
        frame.render_widget(block, modal_area);

        let mut lines = Vec::new();

        // Show which tweet is being replied to
        if let Some(tweet) = self.tweets.get(self.selected_index) {
            lines.push(Line::from(vec![
                Span::styled("To @", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    &tweet.author,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(": ", Style::default().fg(Color::DarkGray)),
            ]));
            let preview_width = inner.width.saturating_sub(2) as usize;
            let preview = if tweet.text.len() > preview_width {
                format!("{}...", &tweet.text[..preview_width.saturating_sub(3)])
            } else {
                tweet.text.clone()
            };
            lines.push(Line::from(Span::styled(
                preview,
                Style::default().fg(Color::DarkGray),
            )));
            lines.push(Line::from(""));
        }

        let display_text = if self.compose_text.is_empty() {
            "Type your reply...".to_string()
        } else {
            self.compose_text.clone()
        };

        let text_style = if self.compose_text.is_empty() {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::White)
        };

        lines.push(Line::from(Span::styled(display_text, text_style)));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Enter to post | Esc to cancel",
            Style::default().fg(Color::DarkGray),
        )));

        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
        frame.render_widget(paragraph, inner);
    }

    fn render_search_modal(&self, frame: &mut Frame, area: Rect) {
        let modal_area = self.centered_modal(area, 50, 7);
        frame.render_widget(Clear, modal_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title("Search Twitter");

        let inner = block.inner(modal_area);
        frame.render_widget(block, modal_area);

        let text = vec![
            Line::from(format!("Query: {}", self.search_query)),
            Line::from(""),
            Line::from(Span::styled(
                "Enter to search | Esc to cancel",
                Style::default().fg(Color::DarkGray),
            )),
        ];

        let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });
        frame.render_widget(paragraph, inner);
    }

    fn render_detail_view(&self, frame: &mut Frame, area: Rect, detail: &TweetDetail) {
        let modal_area = self.center_rect(80, 80, area);
        frame.render_widget(Clear, modal_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title("Tweet Detail (j/k scroll, Esc close)");

        let inner = block.inner(modal_area);
        frame.render_widget(block, modal_area);

        let paragraph = Paragraph::new(detail.content.as_str())
            .wrap(Wrap { trim: false })
            .scroll((detail.scroll_offset, 0));
        frame.render_widget(paragraph, inner);
    }

    fn render_status(&self, frame: &mut Frame, area: Rect, message: &str) {
        let status_area = Rect::new(
            area.x + 2,
            area.y + area.height.saturating_sub(3),
            area.width.saturating_sub(4),
            3,
        );

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));

        let paragraph = Paragraph::new(message)
            .block(block)
            .wrap(Wrap { trim: true });
        frame.render_widget(Clear, status_area);
        frame.render_widget(paragraph, status_area);
    }

    /// Create a centered modal rect with absolute minimum sizes
    fn centered_modal(&self, area: Rect, min_width: u16, min_height: u16) -> Rect {
        let width = area.width.saturating_sub(4).max(min_width).min(area.width);
        let height = min_height.min(area.height);
        let x = area.x + area.width.saturating_sub(width) / 2;
        let y = area.y + area.height.saturating_sub(height) / 2;
        Rect::new(x, y, width, height)
    }

    fn center_rect(&self, percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
}
