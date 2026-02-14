use crate::config::ClockConfig;
use crate::feeds::{FeedData, FeedFetcher};
use crate::ui::widgets::FeedWidget;
use async_trait::async_trait;
use jiff::Timestamp;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::any::Any;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct Clock {
    id: String,
    title: String,
    position: (usize, usize),
    timezones: Vec<String>,
    selected: bool,
    stopwatch_state: StopwatchState,
}

#[derive(Debug, Clone)]
struct StopwatchState {
    running: bool,
    elapsed: Duration,
    last_tick: Option<Instant>,
}

impl Default for StopwatchState {
    fn default() -> Self {
        Self {
            running: false,
            elapsed: Duration::ZERO,
            last_tick: None,
        }
    }
}

impl Clock {
    pub fn new(config: ClockConfig) -> Self {
        Self {
            id: format!("clock-{}-{}", config.position.row, config.position.col),
            title: config.title,
            position: (config.position.row, config.position.col),
            timezones: config.timezones,
            selected: false,
            stopwatch_state: StopwatchState::default(),
        }
    }

    pub fn toggle_stopwatch(&mut self) {
        if self.stopwatch_state.running {
            // Pause
            if let Some(last_tick) = self.stopwatch_state.last_tick {
                self.stopwatch_state.elapsed += last_tick.elapsed();
            }
            self.stopwatch_state.running = false;
            self.stopwatch_state.last_tick = None;
        } else {
            // Start/Resume
            self.stopwatch_state.running = true;
            self.stopwatch_state.last_tick = Some(Instant::now());
        }
    }

    pub fn reset_stopwatch(&mut self) {
        self.stopwatch_state = StopwatchState::default();
    }

    pub fn tick_stopwatch(&mut self) {
        if self.stopwatch_state.running {
            if let Some(last_tick) = self.stopwatch_state.last_tick {
                let delta = last_tick.elapsed();
                self.stopwatch_state.elapsed += delta;
                self.stopwatch_state.last_tick = Some(Instant::now());
            }
        }
    }

    fn get_current_elapsed(&self) -> Duration {
        if self.stopwatch_state.running {
            if let Some(last_tick) = self.stopwatch_state.last_tick {
                self.stopwatch_state.elapsed + last_tick.elapsed()
            } else {
                self.stopwatch_state.elapsed
            }
        } else {
            self.stopwatch_state.elapsed
        }
    }

    fn format_duration(duration: Duration) -> String {
        let total_secs = duration.as_secs();
        let hours = total_secs / 3600;
        let minutes = (total_secs % 3600) / 60;
        let seconds = total_secs % 60;
        let millis = duration.subsec_millis();
        format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, millis)
    }

    fn draw_analog_clock(hour: u32, minute: u32) -> Vec<String> {
        // Create a 9x9 pixelated clock face
        let mut clock = vec![
            "    ╭───╮    ".to_string(),
            "  ╭─────────╮".to_string(),
            " │           │".to_string(),
            "│             │".to_string(),
            "│      •      │".to_string(),
            "│             │".to_string(),
            " │           │".to_string(),
            "  ╰─────────╯".to_string(),
            "    ╰───╯    ".to_string(),
        ];

        // Clock face center (row 4, col 7 in the grid)
        let center_row = 4;
        let center_col = 7;

        // Convert hour to 12-hour format and calculate angles
        let hour_12 = hour % 12;
        // Hour hand: 30 degrees per hour, 0.5 degrees per minute
        let hour_angle =
            (hour_12 as f64 * 30.0 + minute as f64 * 0.5) * std::f64::consts::PI / 180.0;
        // Minute hand: 6 degrees per minute
        let minute_angle = minute as f64 * 6.0 * std::f64::consts::PI / 180.0;

        // Calculate hour hand position (short hand, length 2)
        let hour_length = 2.0;
        let hour_row =
            center_row as f64 - (hour_angle - std::f64::consts::PI / 2.0).sin() * hour_length;
        let hour_col =
            center_col as f64 + (hour_angle - std::f64::consts::PI / 2.0).cos() * hour_length;

        // Calculate minute hand position (long hand, length 3)
        let minute_length = 3.0;
        let minute_row =
            center_row as f64 - (minute_angle - std::f64::consts::PI / 2.0).sin() * minute_length;
        let minute_col =
            center_col as f64 + (minute_angle - std::f64::consts::PI / 2.0).cos() * minute_length;

        // Draw hands
        let hour_r = hour_row.round() as usize;
        let hour_c = hour_col.round() as usize;
        let minute_r = minute_row.round() as usize;
        let minute_c = minute_col.round() as usize;

        // Draw hour hand (thicker)
        if hour_r < clock.len() {
            let mut chars: Vec<char> = clock[hour_r].chars().collect();
            if hour_c < chars.len() {
                chars[hour_c] = '●';
                clock[hour_r] = chars.into_iter().collect();
            }
        }

        // Draw minute hand (thinner)
        if minute_r < clock.len() {
            let mut chars: Vec<char> = clock[minute_r].chars().collect();
            if minute_c < chars.len() {
                let ch = if minute_r == hour_r && minute_c == hour_c {
                    '●' // Both hands at same position
                } else {
                    '○'
                };
                chars[minute_c] = ch;
                clock[minute_r] = chars.into_iter().collect();
            }
        }

        clock
    }
}

struct ClockFetcher;

#[async_trait]
impl FeedFetcher for ClockFetcher {
    async fn fetch(&self) -> anyhow::Result<FeedData> {
        // Clock doesn't need to fetch data
        Ok(FeedData::Loading)
    }
}

impl FeedWidget for Clock {
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

        // Split the area for clocks and stopwatch
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(self.timezones.len() as u16 * 3),
                Constraint::Min(4),
            ])
            .split(inner);

        // Render timezone clocks
        self.render_clocks(frame, chunks[0]);

        // Render stopwatch
        self.render_stopwatch(frame, chunks[1]);
    }

    fn update_data(&mut self, _data: FeedData) {
        // Clock doesn't need external data updates
        // It uses system time
    }

    fn create_fetcher(&self) -> Box<dyn FeedFetcher> {
        Box::new(ClockFetcher)
    }

    fn scroll_up(&mut self) {
        // Not applicable for clock widget
    }

    fn scroll_down(&mut self) {
        // Not applicable for clock widget
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
        None
    }
}

impl Clock {
    fn render_clocks(&self, frame: &mut Frame, area: Rect) {
        let now = Timestamp::now();

        // Try to detect local timezone (fallback to UTC if detection fails)
        let local_tz_name = jiff::tz::TimeZone::system()
            .iana_name()
            .unwrap_or("UTC")
            .to_string();

        // Split area for analog clock and timezone list
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(17), Constraint::Min(0)])
            .split(area);

        // Render analog clock for first timezone
        if let Some(first_tz) = self.timezones.first() {
            if let Ok(tz) = jiff::tz::TimeZone::get(first_tz) {
                let time_in_tz = now.to_zoned(tz);
                let hour = time_in_tz.hour() as u32;
                let minute = time_in_tz.minute() as u32;

                let clock_art = Self::draw_analog_clock(hour, minute);
                let clock_lines: Vec<Line> = clock_art
                    .iter()
                    .map(|line| {
                        Line::from(Span::styled(line.clone(), Style::default().fg(Color::Cyan)))
                    })
                    .collect();

                let clock_widget = Paragraph::new(clock_lines).alignment(Alignment::Left);
                frame.render_widget(clock_widget, chunks[0]);
            }
        }

        // Render timezone text on the right
        let mut text_lines = Vec::new();

        for timezone_str in &self.timezones {
            if let Ok(tz) = jiff::tz::TimeZone::get(timezone_str) {
                let time_in_tz = now.to_zoned(tz);
                let is_local = timezone_str == &local_tz_name;

                // Format time as HH:MM:SS
                let time_str = time_in_tz.strftime("%H:%M:%S").to_string();
                // Format date as MMM DD
                let date_str = time_in_tz.strftime("%b %d").to_string();

                let tz_name = timezone_str
                    .split('/')
                    .next_back()
                    .unwrap_or(timezone_str)
                    .replace('_', " ");

                let style = if is_local {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                text_lines.push(Line::from(vec![
                    Span::styled(format!("{:<15}", tz_name), style),
                    Span::styled(format!("{:<10}", time_str), style),
                    Span::styled(date_str, style),
                ]));
                text_lines.push(Line::from(""));
            }
        }

        let paragraph = Paragraph::new(text_lines).alignment(Alignment::Left);
        frame.render_widget(paragraph, chunks[1]);
    }

    fn render_stopwatch(&self, frame: &mut Frame, area: Rect) {
        let elapsed = self.get_current_elapsed();
        let time_str = Self::format_duration(elapsed);

        let status = if self.stopwatch_state.running {
            "[Running]"
        } else if elapsed.as_secs() > 0 {
            "[Paused]"
        } else {
            "[Stopped]"
        };

        let status_color = if self.stopwatch_state.running {
            Color::Green
        } else if elapsed.as_secs() > 0 {
            Color::Yellow
        } else {
            Color::Gray
        };

        let text = vec![
            Line::from(Span::styled(
                "Stopwatch",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                time_str,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(status, Style::default().fg(status_color))),
            Line::from(""),
            Line::from(Span::styled(
                "s: Start/Pause | r: Reset",
                Style::default().fg(Color::DarkGray),
            )),
        ];

        let paragraph = Paragraph::new(text).alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
    }
}
