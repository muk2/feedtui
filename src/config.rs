use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub widgets: Vec<WidgetConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_refresh_interval")]
    pub refresh_interval_secs: u64,
    #[serde(default = "default_theme")]
    pub theme: String,
}

fn default_refresh_interval() -> u64 {
    60
}

fn default_theme() -> String {
    "dark".to_string()
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            refresh_interval_secs: default_refresh_interval(),
            theme: default_theme(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum WidgetConfig {
    Stocks(StocksConfig),
    Hackernews(HackernewsConfig),
    Sports(SportsConfig),
    Rss(RssConfig),
    Creature(CreatureConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatureConfig {
    #[serde(default = "default_creature_title")]
    pub title: String,
    #[serde(default)]
    pub show_on_startup: bool,
    pub position: Position,
}

fn default_creature_title() -> String {
    "Tui".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StocksConfig {
    #[serde(default = "default_stocks_title")]
    pub title: String,
    pub symbols: Vec<String>,
    pub position: Position,
}

fn default_stocks_title() -> String {
    "Stocks".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HackernewsConfig {
    #[serde(default = "default_hn_title")]
    pub title: String,
    #[serde(default = "default_story_count")]
    pub story_count: usize,
    #[serde(default = "default_story_type")]
    pub story_type: String,
    pub position: Position,
}

fn default_hn_title() -> String {
    "Hacker News".to_string()
}

fn default_story_count() -> usize {
    10
}

fn default_story_type() -> String {
    "top".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SportsConfig {
    #[serde(default = "default_sports_title")]
    pub title: String,
    pub leagues: Vec<String>,
    pub position: Position,
}

fn default_sports_title() -> String {
    "Sports".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RssConfig {
    #[serde(default = "default_rss_title")]
    pub title: String,
    pub feeds: Vec<String>,
    #[serde(default = "default_max_items")]
    pub max_items: usize,
    pub position: Position,
}

fn default_rss_title() -> String {
    "RSS Feed".to_string()
}

fn default_max_items() -> usize {
    15
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            widgets: vec![
                WidgetConfig::Creature(CreatureConfig {
                    title: "Tui".to_string(),
                    show_on_startup: true,
                    position: Position { row: 0, col: 0 },
                }),
                WidgetConfig::Hackernews(HackernewsConfig {
                    title: "Hacker News".to_string(),
                    story_count: 10,
                    story_type: "top".to_string(),
                    position: Position { row: 0, col: 1 },
                }),
                WidgetConfig::Stocks(StocksConfig {
                    title: "Stocks".to_string(),
                    symbols: vec![
                        "AAPL".to_string(),
                        "GOOGL".to_string(),
                        "MSFT".to_string(),
                        "NVDA".to_string(),
                    ],
                    position: Position { row: 1, col: 0 },
                }),
                WidgetConfig::Rss(RssConfig {
                    title: "Tech News".to_string(),
                    feeds: vec!["https://feeds.arstechnica.com/arstechnica/technology-lab".to_string()],
                    max_items: 10,
                    position: Position { row: 1, col: 1 },
                }),
                WidgetConfig::Sports(SportsConfig {
                    title: "Sports".to_string(),
                    leagues: vec!["nba".to_string(), "nfl".to_string()],
                    position: Position { row: 2, col: 0 },
                }),
            ],
        }
    }
}
