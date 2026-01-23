pub mod github;
pub mod hackernews;
pub mod rss;
pub mod sports;
pub mod spotify;
pub mod stocks;

use anyhow::Result;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct FeedMessage {
    pub widget_id: String,
    pub data: FeedData,
}

#[derive(Debug, Clone)]
pub enum FeedData {
    HackerNews(Vec<HnStory>),
    Stocks(Vec<StockQuote>),
    Rss(Vec<RssItem>),
    Sports(Vec<SportsEvent>),
    Github(GithubDashboard),
    Spotify(SpotifyPlayback),
    Loading,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct HnStory {
    pub id: u64,
    pub title: String,
    pub url: Option<String>,
    pub score: u32,
    pub by: String,
    pub descendants: u32,
}

#[derive(Debug, Clone)]
pub struct StockQuote {
    pub symbol: String,
    pub price: f64,
    pub change: f64,
    pub change_percent: f64,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct RssItem {
    pub title: String,
    pub link: Option<String>,
    pub published: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct SportsEvent {
    pub league: String,
    pub home_team: String,
    pub away_team: String,
    pub home_score: Option<u32>,
    pub away_score: Option<u32>,
    pub status: String,
    pub start_time: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GithubNotification {
    pub id: String,
    pub title: String,
    pub notification_type: String,
    pub repository: String,
    pub url: String,
    pub unread: bool,
    pub updated_at: String,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct GithubPullRequest {
    pub id: u64,
    pub number: u32,
    pub title: String,
    pub repository: String,
    pub state: String,
    pub author: String,
    pub created_at: String,
    pub updated_at: String,
    pub draft: bool,
    pub mergeable: Option<bool>,
    pub comments: u32,
    pub review_comments: u32,
    pub additions: u32,
    pub deletions: u32,
}

#[derive(Debug, Clone)]
pub struct GithubCommit {
    pub sha: String,
    pub message: String,
    pub author: String,
    pub repository: String,
    pub branch: String,
    pub timestamp: String,
    pub additions: u32,
    pub deletions: u32,
    pub url: String,
}

#[derive(Debug, Clone, Default)]
pub struct GithubDashboard {
    pub notifications: Vec<GithubNotification>,
    pub pull_requests: Vec<GithubPullRequest>,
    pub commits: Vec<GithubCommit>,
}

#[derive(Debug, Clone, Default)]
pub struct SpotifyPlayback {
    pub is_playing: bool,
    pub track_name: Option<String>,
    pub artist_name: Option<String>,
    pub album_name: Option<String>,
    pub progress_ms: Option<u32>,
    pub duration_ms: Option<u32>,
    pub shuffle_state: bool,
    pub repeat_state: String,
}

#[async_trait]
pub trait FeedFetcher: Send + Sync {
    async fn fetch(&self) -> Result<FeedData>;
}
