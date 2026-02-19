pub mod github;
pub mod hackernews;
pub mod rss;
pub mod sports;
pub mod stocks;
pub mod twitter_archive;
pub mod youtube;

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
    Youtube(Vec<YoutubeVideo>),
    TwitterArchive(Vec<TwitterArchiveItem>),
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
    #[allow(dead_code)]
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct RssItem {
    pub title: String,
    pub link: Option<String>,
    pub published: Option<String>,
    pub source: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SportsEvent {
    pub league: String,
    pub home_team: String,
    pub away_team: String,
    pub home_score: Option<u32>,
    pub away_score: Option<u32>,
    pub status: String,
    #[allow(dead_code)]
    pub start_time: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GithubNotification {
    #[allow(dead_code)]
    pub id: String,
    pub title: String,
    pub notification_type: String,
    pub repository: String,
    #[allow(dead_code)]
    pub url: String,
    pub unread: bool,
    #[allow(dead_code)]
    pub updated_at: String,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct GithubPullRequest {
    #[allow(dead_code)]
    pub id: u64,
    pub number: u32,
    pub title: String,
    pub repository: String,
    pub state: String,
    pub author: String,
    #[allow(dead_code)]
    pub created_at: String,
    #[allow(dead_code)]
    pub updated_at: String,
    pub draft: bool,
    #[allow(dead_code)]
    pub mergeable: Option<bool>,
    pub comments: u32,
    #[allow(dead_code)]
    pub review_comments: u32,
    #[allow(dead_code)]
    pub additions: u32,
    #[allow(dead_code)]
    pub deletions: u32,
}

#[derive(Debug, Clone)]
pub struct GithubCommit {
    pub sha: String,
    pub message: String,
    pub author: String,
    pub repository: String,
    pub branch: String,
    #[allow(dead_code)]
    pub timestamp: String,
    #[allow(dead_code)]
    pub additions: u32,
    #[allow(dead_code)]
    pub deletions: u32,
    #[allow(dead_code)]
    pub url: String,
}

#[derive(Debug, Clone, Default)]
pub struct GithubDashboard {
    pub notifications: Vec<GithubNotification>,
    pub pull_requests: Vec<GithubPullRequest>,
    pub commits: Vec<GithubCommit>,
}

#[derive(Debug, Clone)]
pub struct YoutubeVideo {
    pub id: String,
    pub title: String,
    pub channel: String,
    pub published: String,
    pub description: String,
    #[allow(dead_code)]
    pub thumbnail_url: Option<String>,
    pub view_count: Option<String>,
    pub duration: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TwitterArchiveItem {
    #[allow(dead_code)]
    pub timestamp: String,
    pub original_url: String,
    pub archive_url: String,
    pub tweet_text: Option<String>,
    pub author: Option<String>,
    pub date_display: String,
}

#[async_trait]
pub trait FeedFetcher: Send + Sync {
    async fn fetch(&self) -> Result<FeedData>;
}
