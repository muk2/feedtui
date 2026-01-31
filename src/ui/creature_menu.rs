use crate::creature::{
    art::get_creature_art, get_all_outfits, get_skill_tree, Creature, CreatureColor,
    CreatureSpecies,
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs},
    Frame,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuTab {
    Stats,
    Skills,
    Outfits,
    Customize,
}

impl MenuTab {
    fn all() -> Vec<MenuTab> {
        vec![
            MenuTab::Stats,
            MenuTab::Skills,
            MenuTab::Outfits,
            MenuTab::Customize,
        ]
    }

    fn name(&self) -> &'static str {
        match self {
            MenuTab::Stats => "Stats",
            MenuTab::Skills => "Skills",
            MenuTab::Outfits => "Outfits",
            MenuTab::Customize => "Customize",
        }
    }
}

pub struct CreatureMenu {
    pub visible: bool,
    current_tab: MenuTab,
    skill_list_state: ListState,
    outfit_list_state: ListState,
    species_list_state: ListState,
    color_list_state: ListState,
}

impl Default for CreatureMenu {
    fn default() -> Self {
        let mut skill_list_state = ListState::default();
        skill_list_state.select(Some(0));
        let mut outfit_list_state = ListState::default();
        outfit_list_state.select(Some(0));
        let mut species_list_state = ListState::default();
        species_list_state.select(Some(0));
        let mut color_list_state = ListState::default();
        color_list_state.select(Some(0));

        Self {
            visible: false,
            current_tab: MenuTab::Stats,
            skill_list_state,
            outfit_list_state,
            species_list_state,
            color_list_state,
        }
    }
}

impl CreatureMenu {
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn next_tab(&mut self) {
        let tabs = MenuTab::all();
        let current_idx = tabs
            .iter()
            .position(|t| *t == self.current_tab)
            .unwrap_or(0);
        let next_idx = (current_idx + 1) % tabs.len();
        self.current_tab = tabs[next_idx];
    }

    pub fn prev_tab(&mut self) {
        let tabs = MenuTab::all();
        let current_idx = tabs
            .iter()
            .position(|t| *t == self.current_tab)
            .unwrap_or(0);
        let prev_idx = if current_idx == 0 {
            tabs.len() - 1
        } else {
            current_idx - 1
        };
        self.current_tab = tabs[prev_idx];
    }

    pub fn scroll_up(&mut self) {
        match self.current_tab {
            MenuTab::Skills => {
                if let Some(selected) = self.skill_list_state.selected() {
                    if selected > 0 {
                        self.skill_list_state.select(Some(selected - 1));
                    }
                }
            }
            MenuTab::Outfits => {
                if let Some(selected) = self.outfit_list_state.selected() {
                    if selected > 0 {
                        self.outfit_list_state.select(Some(selected - 1));
                    }
                }
            }
            MenuTab::Customize => {
                if let Some(selected) = self.species_list_state.selected() {
                    if selected > 0 {
                        self.species_list_state.select(Some(selected - 1));
                    }
                }
            }
            _ => {}
        }
    }

    pub fn scroll_down(&mut self, creature: &Creature) {
        match self.current_tab {
            MenuTab::Skills => {
                let skill_count = get_skill_tree().len();
                if let Some(selected) = self.skill_list_state.selected() {
                    if selected < skill_count.saturating_sub(1) {
                        self.skill_list_state.select(Some(selected + 1));
                    }
                }
            }
            MenuTab::Outfits => {
                let outfit_count = creature.unlocked_outfits.len();
                if let Some(selected) = self.outfit_list_state.selected() {
                    if selected < outfit_count.saturating_sub(1) {
                        self.outfit_list_state.select(Some(selected + 1));
                    }
                }
            }
            MenuTab::Customize => {
                let species_count = CreatureSpecies::all().len();
                if let Some(selected) = self.species_list_state.selected() {
                    if selected < species_count.saturating_sub(1) {
                        self.species_list_state.select(Some(selected + 1));
                    }
                }
            }
            _ => {}
        }
    }

    pub fn select(&mut self, creature: &mut Creature) -> bool {
        match self.current_tab {
            MenuTab::Skills => {
                let skills: Vec<_> = get_skill_tree().into_iter().collect();
                if let Some(selected) = self.skill_list_state.selected() {
                    if let Some((id, skill)) = skills.get(selected) {
                        if creature.can_purchase_skill(skill) {
                            creature.purchase_skill(skill);
                            return true;
                        } else if creature.unlocked_skills.contains(id) {
                            creature.toggle_skill(id);
                            return true;
                        }
                    }
                }
            }
            MenuTab::Outfits => {
                if let Some(selected) = self.outfit_list_state.selected() {
                    if let Some(outfit_id) = creature.unlocked_outfits.get(selected).cloned() {
                        creature.equip_outfit(&outfit_id);
                        return true;
                    }
                }
            }
            MenuTab::Customize => {
                let species = CreatureSpecies::all();
                if let Some(selected) = self.species_list_state.selected() {
                    if let Some(new_species) = species.get(selected) {
                        creature.species = new_species.clone();
                        return true;
                    }
                }
            }
            _ => {}
        }
        false
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, creature: &Creature) {
        // Create a centered popup
        let popup_area = centered_rect(80, 80, area);

        // Clear the background
        frame.render_widget(Clear, popup_area);

        // Main block
        let block = Block::default()
            .title(format!(" {} - Level {} ", creature.name, creature.level))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        // Layout: tabs at top, content below
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(inner);

        // Render tabs
        let tab_titles: Vec<Line> = MenuTab::all()
            .iter()
            .map(|t| {
                let style = if *t == self.current_tab {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                Line::from(Span::styled(t.name(), style))
            })
            .collect();

        let tabs = Tabs::new(tab_titles)
            .block(Block::default().borders(Borders::BOTTOM))
            .highlight_style(Style::default().fg(Color::Yellow))
            .select(
                MenuTab::all()
                    .iter()
                    .position(|t| *t == self.current_tab)
                    .unwrap_or(0),
            );
        frame.render_widget(tabs, chunks[0]);

        // Render content based on selected tab
        match self.current_tab {
            MenuTab::Stats => self.render_stats(frame, chunks[1], creature),
            MenuTab::Skills => self.render_skills(frame, chunks[1], creature),
            MenuTab::Outfits => self.render_outfits(frame, chunks[1], creature),
            MenuTab::Customize => self.render_customize(frame, chunks[1], creature),
        }

        // Help text at bottom
        let help =
            Paragraph::new("Tab/Shift+Tab: Switch tabs | j/k: Navigate | Enter: Select | t: Close")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);

        let help_area = Rect {
            x: popup_area.x,
            y: popup_area.y + popup_area.height - 1,
            width: popup_area.width,
            height: 1,
        };
        frame.render_widget(help, help_area);
    }

    fn render_stats(&self, frame: &mut Frame, area: Rect, creature: &Creature) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(area);

        // Creature preview
        let art_lines = get_creature_art(
            &creature.species,
            &creature.mood,
            creature.equipped_outfit.as_deref(),
            0,
        );
        let color = creature.appearance.primary_color.to_ratatui_color();
        let lines: Vec<Line> = art_lines
            .iter()
            .map(|line| Line::from(Span::styled(line.as_str(), Style::default().fg(color))))
            .collect();
        let art = Paragraph::new(lines)
            .alignment(Alignment::Center)
            .block(Block::default().title(" Preview ").borders(Borders::ALL));
        frame.render_widget(art, chunks[0]);

        // Stats
        let stats_text = vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(Color::Gray)),
                Span::styled(&creature.name, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Species: ", Style::default().fg(Color::Gray)),
                Span::styled(creature.species.name(), Style::default().fg(Color::Cyan)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Level: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}", creature.level),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Experience: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}", creature.experience),
                    Style::default().fg(Color::Green),
                ),
            ]),
            Line::from(vec![
                Span::styled("Points: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}", creature.points),
                    Style::default().fg(Color::Magenta),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Total Sessions: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}", creature.total_sessions),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::styled("Total Time: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format_duration(creature.total_time_seconds),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Skills Unlocked: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}", creature.unlocked_skills.len()),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::styled("Outfits Unlocked: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}", creature.unlocked_outfits.len()),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
        ];

        let stats = Paragraph::new(stats_text)
            .block(Block::default().title(" Stats ").borders(Borders::ALL));
        frame.render_widget(stats, chunks[1]);
    }

    fn render_skills(&mut self, frame: &mut Frame, area: Rect, creature: &Creature) {
        let skills = get_skill_tree();
        let mut skill_list: Vec<_> = skills.into_iter().collect();
        // Sort by cost, then by ID for stable ordering (prevents flickering)
        skill_list.sort_by(|a, b| a.1.cost.cmp(&b.1.cost).then_with(|| a.0.cmp(&b.0)));

        let items: Vec<ListItem> = skill_list
            .iter()
            .map(|(id, skill)| {
                let unlocked = creature.unlocked_skills.contains(id);
                let active = creature.active_skills.contains(id);
                let can_buy = creature.can_purchase_skill(skill);

                let status = if unlocked && active {
                    "[ACTIVE]"
                } else if unlocked {
                    "[OWNED]"
                } else if can_buy {
                    "[BUY]"
                } else {
                    "[LOCKED]"
                };

                let status_color = if unlocked && active {
                    Color::Green
                } else if unlocked {
                    Color::Cyan
                } else if can_buy {
                    Color::Yellow
                } else {
                    Color::DarkGray
                };

                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(status, Style::default().fg(status_color)),
                        Span::raw(" "),
                        Span::styled(&skill.name, Style::default().fg(Color::White)),
                        Span::raw(" - "),
                        Span::styled(
                            format!("{} pts", skill.cost),
                            Style::default().fg(Color::Magenta),
                        ),
                    ]),
                    Line::from(Span::styled(
                        format!("  {}", skill.description),
                        Style::default().fg(Color::Gray),
                    )),
                ])
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(format!(" Skill Tree (Points: {}) ", creature.points))
                    .borders(Borders::ALL),
            )
            .highlight_style(Style::default().bg(Color::DarkGray));

        frame.render_stateful_widget(list, area, &mut self.skill_list_state);
    }

    fn render_outfits(&mut self, frame: &mut Frame, area: Rect, creature: &Creature) {
        let all_outfits = get_all_outfits();

        let items: Vec<ListItem> = creature
            .unlocked_outfits
            .iter()
            .filter_map(|id| all_outfits.get(id))
            .map(|outfit| {
                let equipped = creature.equipped_outfit.as_ref() == Some(&outfit.id);
                let marker = if equipped { "[*]" } else { "[ ]" };

                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(
                            marker,
                            Style::default().fg(if equipped {
                                Color::Green
                            } else {
                                Color::DarkGray
                            }),
                        ),
                        Span::raw(" "),
                        Span::styled(&outfit.name, Style::default().fg(Color::White)),
                    ]),
                    Line::from(Span::styled(
                        format!("  {}", outfit.description),
                        Style::default().fg(Color::Gray),
                    )),
                ])
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().title(" Outfits ").borders(Borders::ALL))
            .highlight_style(Style::default().bg(Color::DarkGray));

        frame.render_stateful_widget(list, area, &mut self.outfit_list_state);
    }

    fn render_customize(&mut self, frame: &mut Frame, area: Rect, creature: &Creature) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Species selection
        let species = CreatureSpecies::all();
        let items: Vec<ListItem> = species
            .iter()
            .map(|s| {
                let selected = creature.species == *s;
                let marker = if selected { "[*]" } else { "[ ]" };

                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(
                            marker,
                            Style::default().fg(if selected {
                                Color::Green
                            } else {
                                Color::DarkGray
                            }),
                        ),
                        Span::raw(" "),
                        Span::styled(s.name(), Style::default().fg(Color::White)),
                    ]),
                    Line::from(Span::styled(
                        format!("  {}", s.description()),
                        Style::default().fg(Color::Gray),
                    )),
                ])
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().title(" Species ").borders(Borders::ALL))
            .highlight_style(Style::default().bg(Color::DarkGray));

        frame.render_stateful_widget(list, chunks[0], &mut self.species_list_state);

        // Color preview
        let colors = CreatureColor::all();
        let color_text: Vec<Line> = colors
            .iter()
            .map(|c| {
                let selected = creature.appearance.primary_color == *c;
                let marker = if selected { "[*]" } else { "[ ]" };
                Line::from(vec![
                    Span::styled(marker, Style::default().fg(Color::White)),
                    Span::raw(" "),
                    Span::styled(
                        format!("{:?}", c),
                        Style::default().fg(c.to_ratatui_color()),
                    ),
                ])
            })
            .collect();

        let colors_para = Paragraph::new(color_text)
            .block(Block::default().title(" Colors ").borders(Borders::ALL));
        frame.render_widget(colors_para, chunks[1]);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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

fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}
