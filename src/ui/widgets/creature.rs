use crate::config::CreatureConfig;
use crate::creature::Creature;
use crate::creature::art::{get_creature_art, get_greeting, get_idle_message};
use crate::feeds::{FeedData, FeedFetcher};
use crate::ui::widgets::FeedWidget;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
};
use std::time::Instant;

pub struct CreatureWidget {
    config: CreatureConfig,
    creature: Creature,
    selected: bool,
    animation_frame: usize,
    last_frame_time: Instant,
    show_greeting: bool,
    greeting_timer: Option<Instant>,
}

impl CreatureWidget {
    pub fn new(config: CreatureConfig, creature: Creature) -> Self {
        Self {
            config,
            creature,
            selected: false,
            animation_frame: 0,
            last_frame_time: Instant::now(),
            show_greeting: true,
            greeting_timer: Some(Instant::now()),
        }
    }

    pub fn creature(&self) -> &Creature {
        &self.creature
    }

    pub fn creature_mut(&mut self) -> &mut Creature {
        &mut self.creature
    }

    /// Update animation frame
    pub fn tick(&mut self) {
        // Animate every 500ms
        if self.last_frame_time.elapsed().as_millis() > 500 {
            self.animation_frame = self.animation_frame.wrapping_add(1);
            self.last_frame_time = Instant::now();
        }

        // Hide greeting after 5 seconds
        if let Some(timer) = self.greeting_timer {
            if timer.elapsed().as_secs() > 5 {
                self.show_greeting = false;
                self.greeting_timer = None;
            }
        }
    }
}

impl FeedWidget for CreatureWidget {
    fn id(&self) -> String {
        format!(
            "creature-{}-{}",
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
            .title(format!(
                " {} - {} (Lv.{}) ",
                self.config.title, self.creature.name, self.creature.level
            ))
            .borders(Borders::ALL)
            .border_style(border_style);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Split inner area into sections
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8), // Creature art
                Constraint::Length(2), // XP bar
                Constraint::Length(2), // Stats
                Constraint::Min(1),    // Message/greeting
            ])
            .split(inner);

        // Render creature ASCII art
        self.render_creature_art(frame, chunks[0]);

        // Render XP bar
        self.render_xp_bar(frame, chunks[1]);

        // Render stats
        self.render_stats(frame, chunks[2]);

        // Render message
        self.render_message(frame, chunks[3]);
    }

    fn update_data(&mut self, _data: FeedData) {
        // Creature widget doesn't receive feed data
        // It's updated through its own mechanism
    }

    fn create_fetcher(&self) -> Box<dyn FeedFetcher> {
        // Return a dummy fetcher since creature doesn't fetch external data
        Box::new(CreatureFetcher {})
    }

    fn scroll_up(&mut self) {
        // Could be used to cycle through emotes or view stats
    }

    fn scroll_down(&mut self) {
        // Could be used to cycle through emotes or view stats
    }

    fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }
}

impl CreatureWidget {
    fn render_creature_art(&self, frame: &mut Frame, area: Rect) {
        let outfit = self.creature.equipped_outfit.as_deref();
        let art_lines = get_creature_art(
            &self.creature.species,
            &self.creature.mood,
            outfit,
            self.animation_frame,
        );

        let color = self.creature.appearance.primary_color.to_ratatui_color();

        let lines: Vec<Line> = art_lines
            .iter()
            .map(|line| Line::from(Span::styled(line.as_str(), Style::default().fg(color))))
            .collect();

        let art = Paragraph::new(lines).alignment(Alignment::Center);
        frame.render_widget(art, area);
    }

    fn render_xp_bar(&self, frame: &mut Frame, area: Rect) {
        let progress = self.creature.level_progress();
        let xp_to_next = self.creature.xp_to_next_level();

        let label = format!(
            "XP: {} / {} to Lv.{}",
            self.creature.experience,
            self.creature.experience + xp_to_next,
            self.creature.level + 1
        );

        let gauge = Gauge::default()
            .block(Block::default())
            .gauge_style(
                Style::default()
                    .fg(Color::Cyan)
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .percent((progress * 100.0) as u16)
            .label(label);

        frame.render_widget(gauge, area);
    }

    fn render_stats(&self, frame: &mut Frame, area: Rect) {
        let stats_line = Line::from(vec![
            Span::styled("Points: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{}", self.creature.points),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  |  "),
            Span::styled("Sessions: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{}", self.creature.total_sessions),
                Style::default().fg(Color::Green),
            ),
            Span::raw("  |  "),
            Span::styled("Mood: ", Style::default().fg(Color::White)),
            Span::styled(
                self.creature.mood.emoji(),
                Style::default().fg(Color::Magenta),
            ),
        ]);

        let stats = Paragraph::new(stats_line).alignment(Alignment::Center);
        frame.render_widget(stats, area);
    }

    fn render_message(&self, frame: &mut Frame, area: Rect) {
        let message = if self.show_greeting {
            get_greeting(&self.creature.mood, &self.creature.name)
        } else {
            let idle = get_idle_message(self.animation_frame);
            format!("{}: {}", self.creature.name, idle)
        };

        let msg = Paragraph::new(message)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        frame.render_widget(msg, area);
    }
}

/// Dummy fetcher for creature (doesn't actually fetch anything)
struct CreatureFetcher;

#[async_trait::async_trait]
impl FeedFetcher for CreatureFetcher {
    async fn fetch(&self) -> anyhow::Result<FeedData> {
        // Return loading to indicate this widget manages its own state
        Ok(FeedData::Loading)
    }
}
