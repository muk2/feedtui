use crate::config::GithubConfig;
use crate::feeds::github::GithubFetcher;
use crate::feeds::{
    FeedData, FeedFetcher, GithubCommit, GithubDashboard, GithubNotification, GithubPullRequest,
};
use crate::ui::widgets::FeedWidget;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Tabs},
    Frame,
};

#[derive(Debug, Clone, Copy, PartialEq)]
enum DashboardTab {
    Notifications,
    PullRequests,
    Commits,
}

pub struct GithubWidget {
    config: GithubConfig,
    dashboard: GithubDashboard,
    current_tab: DashboardTab,
    loading: bool,
    error: Option<String>,
    scroll_state: ListState,
    selected: bool,
}

impl GithubWidget {
    pub fn new(config: GithubConfig) -> Self {
        let mut scroll_state = ListState::default();
        scroll_state.select(Some(0));

        // Determine initial tab based on config
        let current_tab = if config.show_notifications {
            DashboardTab::Notifications
        } else if config.show_pull_requests {
            DashboardTab::PullRequests
        } else if config.show_commits {
            DashboardTab::Commits
        } else {
            DashboardTab::Notifications
        };

        Self {
            config,
            dashboard: GithubDashboard::default(),
            current_tab,
            loading: true,
            error: None,
            scroll_state,
            selected: false,
        }
    }

    pub fn next_tab(&mut self) {
        let available_tabs = self.get_available_tabs();
        if available_tabs.is_empty() {
            return;
        }

        let current_idx = available_tabs
            .iter()
            .position(|&t| t == self.current_tab)
            .unwrap_or(0);
        let next_idx = (current_idx + 1) % available_tabs.len();
        self.current_tab = available_tabs[next_idx];

        // Reset scroll when changing tabs
        self.scroll_state.select(Some(0));
    }

    pub fn prev_tab(&mut self) {
        let available_tabs = self.get_available_tabs();
        if available_tabs.is_empty() {
            return;
        }

        let current_idx = available_tabs
            .iter()
            .position(|&t| t == self.current_tab)
            .unwrap_or(0);
        let prev_idx = if current_idx == 0 {
            available_tabs.len() - 1
        } else {
            current_idx - 1
        };
        self.current_tab = available_tabs[prev_idx];

        // Reset scroll when changing tabs
        self.scroll_state.select(Some(0));
    }

    fn get_available_tabs(&self) -> Vec<DashboardTab> {
        let mut tabs = Vec::new();
        if self.config.show_notifications {
            tabs.push(DashboardTab::Notifications);
        }
        if self.config.show_pull_requests {
            tabs.push(DashboardTab::PullRequests);
        }
        if self.config.show_commits {
            tabs.push(DashboardTab::Commits);
        }
        tabs
    }

    fn render_notifications(&self) -> Vec<ListItem> {
        self.dashboard
            .notifications
            .iter()
            .enumerate()
            .map(|(i, notif)| {
                let unread_indicator = if notif.unread { "● " } else { "○ " };
                let title_line = Line::from(vec![
                    Span::styled(
                        format!("{}{} ", unread_indicator, i + 1),
                        if notif.unread {
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::DarkGray)
                        },
                    ),
                    Span::styled(&notif.title, Style::default().fg(Color::White)),
                ]);

                let meta_line = Line::from(vec![
                    Span::styled(
                        format!("   {} | ", notif.repository),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(
                        format!("{} | ", notif.notification_type),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::styled(&notif.reason, Style::default().fg(Color::DarkGray)),
                ]);

                ListItem::new(vec![title_line, meta_line])
            })
            .collect()
    }

    fn render_pull_requests(&self) -> Vec<ListItem> {
        self.dashboard
            .pull_requests
            .iter()
            .enumerate()
            .map(|(i, pr)| {
                let status_icon = if pr.draft {
                    "📝 "
                } else if pr.state == "open" {
                    "🟢 "
                } else {
                    "🔴 "
                };

                let title_line = Line::from(vec![
                    Span::styled(
                        format!("{}#{} ", status_icon, pr.number),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(&pr.title, Style::default().fg(Color::White)),
                ]);

                let meta_line = Line::from(vec![
                    Span::styled(
                        format!("   {} | ", pr.repository),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(
                        format!("by {} | ", pr.author),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::styled(
                        format!("{} comments", pr.comments),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]);

                ListItem::new(vec![title_line, meta_line])
            })
            .collect()
    }

    fn render_commits(&self) -> Vec<ListItem> {
        self.dashboard
            .commits
            .iter()
            .enumerate()
            .map(|(i, commit)| {
                let title_line = Line::from(vec![
                    Span::styled(
                        format!("🔹 {} ", &commit.sha),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(&commit.message, Style::default().fg(Color::White)),
                ]);

                let meta_line = Line::from(vec![
                    Span::styled(
                        format!("   {} | ", commit.repository),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(
                        format!("by {} | ", commit.author),
                        Style::default().fg(Color::Green),
                    ),
                    Span::styled(&commit.branch, Style::default().fg(Color::DarkGray)),
                ]);

                ListItem::new(vec![title_line, meta_line])
            })
            .collect()
    }
}

impl FeedWidget for GithubWidget {
    fn id(&self) -> String {
        format!(
            "github-{}-{}",
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

        // Build tab titles
        let mut tab_titles = Vec::new();
        if self.config.show_notifications {
            let unread_count = self
                .dashboard
                .notifications
                .iter()
                .filter(|n| n.unread)
                .count();
            let notif_title = if unread_count > 0 {
                format!(" Notifications ({}) ", unread_count)
            } else {
                " Notifications ".to_string()
            };
            tab_titles.push(notif_title);
        }
        if self.config.show_pull_requests {
            tab_titles.push(format!(
                " Pull Requests ({}) ",
                self.dashboard.pull_requests.len()
            ));
        }
        if self.config.show_commits {
            tab_titles.push(format!(" Commits ({}) ", self.dashboard.commits.len()));
        }

        // Determine selected tab index
        let available_tabs = self.get_available_tabs();
        let selected_tab_idx = available_tabs
            .iter()
            .position(|&t| t == self.current_tab)
            .unwrap_or(0);

        let title = format!(" {} ", self.config.title);
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style);

        if self.loading
            && self.dashboard.notifications.is_empty()
            && self.dashboard.pull_requests.is_empty()
            && self.dashboard.commits.is_empty()
        {
            let loading_text = List::new(vec![ListItem::new("Loading dashboard...")]).block(block);
            frame.render_widget(loading_text, area);
            return;
        }

        if let Some(ref error) = self.error {
            let error_text =
                List::new(vec![ListItem::new(format!("Error: {}", error))]).block(block);
            frame.render_widget(error_text, area);
            return;
        }

        // Render tabs
        let tabs = Tabs::new(tab_titles)
            .block(block.clone())
            .select(selected_tab_idx)
            .highlight_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_widget(tabs, area);

        // Render content based on current tab
        let items = match self.current_tab {
            DashboardTab::Notifications => {
                if self.dashboard.notifications.is_empty() {
                    vec![ListItem::new("No notifications")]
                } else {
                    self.render_notifications()
                }
            }
            DashboardTab::PullRequests => {
                if self.dashboard.pull_requests.is_empty() {
                    vec![ListItem::new("No pull requests")]
                } else {
                    self.render_pull_requests()
                }
            }
            DashboardTab::Commits => {
                if self.dashboard.commits.is_empty() {
                    vec![ListItem::new("No recent commits")]
                } else {
                    self.render_commits()
                }
            }
        };

        // Create inner area for list (below tabs)
        let inner_area = Rect {
            x: area.x + 1,
            y: area.y + 2,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(3),
        };

        let list = List::new(items).highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

        let mut state = self.scroll_state.clone();
        frame.render_stateful_widget(list, inner_area, &mut state);
    }

    fn update_data(&mut self, data: FeedData) {
        self.loading = false;
        match data {
            FeedData::Github(dashboard) => {
                self.dashboard = dashboard;
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
        Box::new(GithubFetcher::new(
            self.config.token.clone(),
            self.config.username.clone(),
            self.config.show_notifications,
            self.config.show_pull_requests,
            self.config.show_commits,
            self.config.max_notifications,
            self.config.max_pull_requests,
            self.config.max_commits,
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
        let max_items = match self.current_tab {
            DashboardTab::Notifications => self.dashboard.notifications.len(),
            DashboardTab::PullRequests => self.dashboard.pull_requests.len(),
            DashboardTab::Commits => self.dashboard.commits.len(),
        };

        if let Some(selected) = self.scroll_state.selected() {
            if selected < max_items.saturating_sub(1) {
                self.scroll_state.select(Some(selected + 1));
            }
        }
    }

    fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }

    fn get_selected_url(&self) -> Option<String> {
        let idx = self.scroll_state.selected()?;
        match self.current_tab {
            DashboardTab::Notifications => {
                self.dashboard.notifications.get(idx).map(|n| n.url.clone())
            }
            DashboardTab::PullRequests => self
                .dashboard
                .pull_requests
                .get(idx)
                .map(|pr| format!("https://github.com/{}/pull/{}", pr.repository, pr.number)),
            DashboardTab::Commits => self.dashboard.commits.get(idx).map(|c| c.url.clone()),
        }
    }
}
