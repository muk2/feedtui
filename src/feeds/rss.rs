use super::{FeedData, FeedFetcher, RssItem};
use anyhow::Result;
use async_trait::async_trait;

pub struct RssFetcher {
    feeds: Vec<String>,
    max_items: usize,
    client: reqwest::Client,
}

impl RssFetcher {
    pub fn new(feeds: Vec<String>, max_items: usize) -> Self {
        Self {
            feeds,
            max_items,
            client: reqwest::Client::new(),
        }
    }

    async fn fetch_feed(&self, url: &str) -> Result<Vec<RssItem>> {
        let response = self
            .client
            .get(url)
            .header("User-Agent", "feedtui/1.0")
            .send()
            .await?;

        let body = response.bytes().await?;
        let feed = feed_rs::parser::parse(&body[..])?;

        let source_name = feed
            .title
            .map(|t| t.content)
            .unwrap_or_else(|| "Unknown".to_string());

        let items: Vec<RssItem> = feed
            .entries
            .into_iter()
            .take(self.max_items)
            .map(|entry| {
                // Get description from summary or content
                let description = entry
                    .summary
                    .map(|s| s.content)
                    .or_else(|| entry.content.and_then(|c| c.body));

                RssItem {
                    title: entry
                        .title
                        .map(|t| t.content)
                        .unwrap_or_else(|| "No title".to_string()),
                    link: entry.links.first().map(|l| l.href.clone()),
                    published: entry
                        .published
                        .map(|d| d.format("%Y-%m-%d %H:%M").to_string()),
                    source: source_name.clone(),
                    description,
                }
            })
            .collect();

        Ok(items)
    }
}

#[async_trait]
impl FeedFetcher for RssFetcher {
    async fn fetch(&self) -> Result<FeedData> {
        let mut all_items = Vec::new();

        for feed_url in &self.feeds {
            match self.fetch_feed(feed_url).await {
                Ok(items) => all_items.extend(items),
                Err(_) => continue,
            }
        }

        // Sort by date if available, limit to max_items
        all_items.truncate(self.max_items);

        Ok(FeedData::Rss(all_items))
    }
}
