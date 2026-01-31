use crate::config::{Config, WidgetConfig};
use crate::creature::persistence::{default_creature_path, load_or_create_creature, save_creature};
use crate::creature::Creature;
use crate::event::{Event, EventHandler};
use crate::feeds::{FeedData, FeedMessage};
use crate::ui::creature_menu::CreatureMenu;
use crate::ui::widgets::{
    creature::CreatureWidget, github::GithubWidget, hackernews::HackernewsWidget, rss::RssWidget,
    sports::SportsWidget, stocks::StocksWidget, youtube::YoutubeWidget, FeedWidget,
};
use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Frame, Terminal,
};
use std::io::{self, Stdout};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

pub struct App {
    config: Config,
    widgets: Vec<Box<dyn FeedWidget>>,
    selected_widget: usize,
    should_quit: bool,
    feed_rx: mpsc::UnboundedReceiver<FeedMessage>,
    feed_tx: mpsc::UnboundedSender<FeedMessage>,
    creature_path: PathBuf,
    creature_widget_idx: Option<usize>,
    last_xp_tick: Instant,
    creature_menu: CreatureMenu,
}

impl App {
    pub fn new(config: Config) -> Self {
        let (feed_tx, feed_rx) = mpsc::unbounded_channel();

        // Load or create creature
        let creature_path = default_creature_path();
        let creature = load_or_create_creature(&creature_path).unwrap_or_else(|e| {
            eprintln!("Warning: Could not load creature: {}", e);
            Creature::default()
        });

        let mut widgets: Vec<Box<dyn FeedWidget>> = Vec::new();
        let mut creature_widget_idx = None;

        for widget_config in &config.widgets {
            let widget: Box<dyn FeedWidget> = match widget_config {
                WidgetConfig::Hackernews(cfg) => Box::new(HackernewsWidget::new(cfg.clone())),
                WidgetConfig::Stocks(cfg) => Box::new(StocksWidget::new(cfg.clone())),
                WidgetConfig::Rss(cfg) => Box::new(RssWidget::new(cfg.clone())),
                WidgetConfig::Sports(cfg) => Box::new(SportsWidget::new(cfg.clone())),
                WidgetConfig::Github(cfg) => Box::new(GithubWidget::new(cfg.clone())),
                WidgetConfig::Youtube(cfg) => Box::new(YoutubeWidget::new(cfg.clone())),
                WidgetConfig::Creature(cfg) => {
                    creature_widget_idx = Some(widgets.len());
                    Box::new(CreatureWidget::new(cfg.clone(), creature.clone()))
                }
            };
            widgets.push(widget);
        }

        Self {
            config,
            widgets,
            selected_widget: 0,
            should_quit: false,
            feed_rx,
            feed_tx,
            creature_path,
            creature_widget_idx,
            last_xp_tick: Instant::now(),
            creature_menu: CreatureMenu::default(),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut terminal = Self::setup_terminal()?;

        // Set up panic hook to restore terminal
        let original_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic| {
            Self::restore_terminal_panic();
            original_hook(panic);
        }));

        // Start feed fetchers
        self.start_feed_fetchers();

        // Event handler
        let tick_rate = Duration::from_millis(250);
        let mut events = EventHandler::new(tick_rate);

        // Main loop
        while !self.should_quit {
            // Update creature
            self.tick_creature();

            // Draw UI
            terminal.draw(|frame| self.render(frame))?;

            // Handle events
            tokio::select! {
                event = events.next() => {
                    if let Ok(event) = event {
                        self.handle_event(event);
                    }
                }
                Some(msg) = self.feed_rx.recv() => {
                    self.handle_feed_message(msg);
                }
            }
        }

        // Save creature state before exiting
        self.save_creature_state();

        Self::restore_terminal(&mut terminal)?;
        Ok(())
    }

    fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(terminal)
    }

    fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
        Ok(())
    }

    fn restore_terminal_panic() {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
    }

    fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key) => {
                // If creature menu is visible, route events there
                if self.creature_menu.visible {
                    match key.code {
                        KeyCode::Char('t') | KeyCode::Esc => self.creature_menu.toggle(),
                        KeyCode::Tab => self.creature_menu.next_tab(),
                        KeyCode::BackTab => self.creature_menu.prev_tab(),
                        KeyCode::Down | KeyCode::Char('j') => {
                            if let Some(creature) = self.get_creature() {
                                self.creature_menu.scroll_down(&creature);
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') => self.creature_menu.scroll_up(),
                        KeyCode::Enter => {
                            if let Some(idx) = self.creature_widget_idx {
                                if let Some(widget) = self.widgets.get_mut(idx) {
                                    if let Some(creature_widget) = widget
                                        .as_any_mut()
                                        .and_then(|w| w.downcast_mut::<CreatureWidget>())
                                    {
                                        self.creature_menu.select(creature_widget.creature_mut());
                                    }
                                }
                            }
                        }
                        KeyCode::Char('q') => self.should_quit = true,
                        _ => {}
                    }
                    return;
                }

                // Normal event handling
                match key.code {
                    KeyCode::Char('q') => self.should_quit = true,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.should_quit = true
                    }
                    KeyCode::Char('r') => self.refresh_all(),
                    KeyCode::Char('t') => self.toggle_creature_menu(),
                    KeyCode::Tab => self.next_widget(),
                    KeyCode::BackTab => self.prev_widget(),
                    KeyCode::Down | KeyCode::Char('j') => self.scroll_down(),
                    KeyCode::Up | KeyCode::Char('k') => self.scroll_up(),
                    KeyCode::Left | KeyCode::Char('h') => self.switch_tab_prev(),
                    KeyCode::Right | KeyCode::Char('l') => self.switch_tab_next(),
                    KeyCode::Enter => self.open_selected_url(),
                    _ => {}
                }
            }
            Event::Tick => {}
            Event::Resize(_, _) => {}
            Event::Mouse(_) => {}
        }
    }

    fn handle_feed_message(&mut self, msg: FeedMessage) {
        for widget in &mut self.widgets {
            if widget.id() == msg.widget_id {
                widget.update_data(msg.data.clone());
                break;
            }
        }
    }

    fn start_feed_fetchers(&self) {
        for widget in &self.widgets {
            let tx = self.feed_tx.clone();
            let widget_id = widget.id();
            let fetcher = widget.create_fetcher();
            let refresh_interval = Duration::from_secs(self.config.general.refresh_interval_secs);

            tokio::spawn(async move {
                loop {
                    match fetcher.fetch().await {
                        Ok(data) => {
                            let _ = tx.send(FeedMessage {
                                widget_id: widget_id.clone(),
                                data,
                            });
                        }
                        Err(e) => {
                            let _ = tx.send(FeedMessage {
                                widget_id: widget_id.clone(),
                                data: FeedData::Error(e.to_string()),
                            });
                        }
                    }
                    tokio::time::sleep(refresh_interval).await;
                }
            });
        }
    }

    fn refresh_all(&self) {
        // Fetchers run continuously, so this triggers an immediate refresh
        // by restarting the fetchers (simplified for now)
    }

    fn toggle_creature_menu(&mut self) {
        self.creature_menu.toggle();
    }

    fn get_creature(&self) -> Option<Creature> {
        if let Some(idx) = self.creature_widget_idx {
            if let Some(widget) = self.widgets.get(idx) {
                if let Some(creature_widget) = widget
                    .as_any()
                    .and_then(|w| w.downcast_ref::<CreatureWidget>())
                {
                    return Some(creature_widget.creature().clone());
                }
            }
        }
        None
    }

    fn next_widget(&mut self) {
        if !self.widgets.is_empty() {
            self.widgets[self.selected_widget].set_selected(false);
            self.selected_widget = (self.selected_widget + 1) % self.widgets.len();
            self.widgets[self.selected_widget].set_selected(true);
        }
    }

    fn prev_widget(&mut self) {
        if !self.widgets.is_empty() {
            self.widgets[self.selected_widget].set_selected(false);
            self.selected_widget = if self.selected_widget == 0 {
                self.widgets.len() - 1
            } else {
                self.selected_widget - 1
            };
            self.widgets[self.selected_widget].set_selected(true);
        }
    }

    fn scroll_down(&mut self) {
        if !self.widgets.is_empty() {
            self.widgets[self.selected_widget].scroll_down();
        }
    }

    fn scroll_up(&mut self) {
        if !self.widgets.is_empty() {
            self.widgets[self.selected_widget].scroll_up();
        }
    }

    fn switch_tab_next(&mut self) {
        if !self.widgets.is_empty() {
            if let Some(widget) = self.widgets.get_mut(self.selected_widget) {
                if let Some(github_widget) = widget
                    .as_any_mut()
                    .and_then(|w| w.downcast_mut::<GithubWidget>())
                {
                    github_widget.next_tab();
                }
            }
        }
    }

    fn switch_tab_prev(&mut self) {
        if !self.widgets.is_empty() {
            if let Some(widget) = self.widgets.get_mut(self.selected_widget) {
                if let Some(github_widget) = widget
                    .as_any_mut()
                    .and_then(|w| w.downcast_mut::<GithubWidget>())
                {
                    github_widget.prev_tab();
                }
            }
        }
    }

    /// Open the selected item's URL in the default browser
    fn open_selected_url(&self) {
        if !self.widgets.is_empty() {
            if let Some(url) = self.widgets[self.selected_widget].get_selected_url() {
                let _ = open::that(&url);
            }
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();

        // Calculate grid dimensions
        let (max_row, max_col) = self.calculate_grid_dimensions();

        // Create row constraints
        let row_constraints: Vec<Constraint> = (0..=max_row)
            .map(|_| Constraint::Ratio(1, (max_row + 1) as u32))
            .collect();

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(row_constraints)
            .split(area);

        // Create column constraints for each row
        for row_idx in 0..=max_row {
            let col_constraints: Vec<Constraint> = (0..=max_col)
                .map(|_| Constraint::Ratio(1, (max_col + 1) as u32))
                .collect();

            let cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(col_constraints)
                .split(rows[row_idx]);

            // Render widgets in their positions
            for (widget_idx, widget) in self.widgets.iter().enumerate() {
                let pos = widget.position();
                if pos.0 == row_idx && pos.1 <= max_col {
                    let cell = cols[pos.1];
                    widget.render(frame, cell, widget_idx == self.selected_widget);
                }
            }
        }

        // Render creature menu overlay if visible
        if self.creature_menu.visible {
            if let Some(creature) = self.get_creature() {
                self.creature_menu.render(frame, area, &creature);
            }
        }
    }

    fn calculate_grid_dimensions(&self) -> (usize, usize) {
        let mut max_row = 0;
        let mut max_col = 0;

        for widget in &self.widgets {
            let (row, col) = widget.position();
            max_row = max_row.max(row);
            max_col = max_col.max(col);
        }

        (max_row, max_col)
    }

    /// Tick the creature widget for animations and XP
    fn tick_creature(&mut self) {
        if let Some(idx) = self.creature_widget_idx {
            // Tick animation
            if let Some(widget) = self.widgets.get_mut(idx) {
                if let Some(creature_widget) = widget
                    .as_any_mut()
                    .and_then(|w| w.downcast_mut::<CreatureWidget>())
                {
                    creature_widget.tick();

                    // Award XP every 10 seconds
                    if self.last_xp_tick.elapsed().as_secs() >= 10 {
                        let xp = creature_widget.creature_mut().tick_session(10);
                        creature_widget.creature_mut().add_experience(xp);
                        self.last_xp_tick = Instant::now();
                    }
                }
            }
        }
    }

    /// Save creature state to disk
    fn save_creature_state(&self) {
        if let Some(idx) = self.creature_widget_idx {
            if let Some(widget) = self.widgets.get(idx) {
                if let Some(creature_widget) = widget
                    .as_any()
                    .and_then(|w| w.downcast_ref::<CreatureWidget>())
                {
                    if let Err(e) = save_creature(creature_widget.creature(), &self.creature_path) {
                        eprintln!("Warning: Could not save creature state: {}", e);
                    }
                }
            }
        }
    }
}
