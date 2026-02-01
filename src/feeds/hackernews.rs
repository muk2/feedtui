use super::{FeedData, FeedFetcher, HnComment, HnStory};
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;

const HN_API_BASE: &str = "https://hacker-news.firebaseio.com/v0";

pub struct HnFetcher {
    story_type: String,
    story_count: usize,
    client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct HnItem {
    id: u64,
    title: Option<String>,
    url: Option<String>,
    score: Option<u32>,
    by: Option<String>,
    descendants: Option<u32>,
    text: Option<String>,
    kids: Option<Vec<u64>>,
}

impl HnFetcher {
    pub fn new(story_type: String, story_count: usize) -> Self {
        Self {
            story_type,
            story_count,
            client: reqwest::Client::new(),
        }
    }

    async fn fetch_story_ids(&self) -> Result<Vec<u64>> {
        let url = format!("{}/{}stories.json", HN_API_BASE, self.story_type);
        let ids: Vec<u64> = self.client.get(&url).send().await?.json().await?;
        Ok(ids.into_iter().take(self.story_count).collect())
    }

    async fn fetch_story(&self, id: u64) -> Result<HnStory> {
        let url = format!("{}/item/{}.json", HN_API_BASE, id);
        let item: HnItem = self.client.get(&url).send().await?.json().await?;

        // Fetch top-level comments (limit to first 5 for performance)
        let mut comments = Vec::new();
        if let Some(kids) = &item.kids {
            for &kid_id in kids.iter().take(5) {
                if let Ok(comment) = self.fetch_comment(kid_id).await {
                    comments.push(comment);
                }
            }
        }

        Ok(HnStory {
            id: item.id,
            title: item.title.unwrap_or_else(|| "No title".to_string()),
            url: item.url,
            score: item.score.unwrap_or(0),
            by: item.by.unwrap_or_else(|| "unknown".to_string()),
            descendants: item.descendants.unwrap_or(0),
            text: item.text,
            comments,
        })
    }

    async fn fetch_comment(&self, id: u64) -> Result<HnComment> {
        let url = format!("{}/item/{}.json", HN_API_BASE, id);
        let item: HnItem = self.client.get(&url).send().await?.json().await?;

        Ok(HnComment {
            by: item.by.unwrap_or_else(|| "unknown".to_string()),
            text: item.text.unwrap_or_else(|| "".to_string()),
            time: item.id, // Using id as a placeholder for time
        })
    }
}

#[async_trait]
impl FeedFetcher for HnFetcher {
    async fn fetch(&self) -> Result<FeedData> {
        let ids = self.fetch_story_ids().await?;

        let mut stories = Vec::new();
        for id in ids {
            match self.fetch_story(id).await {
                Ok(story) => stories.push(story),
                Err(_) => continue,
            }
        }

        Ok(FeedData::HackerNews(stories))
    }
}
