use super::{FeedData, FeedFetcher, YoutubeVideo};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::Deserialize;

const YOUTUBE_API_BASE: &str = "https://www.googleapis.com/youtube/v3";

pub struct YoutubeFetcher {
    api_key: String,
    channels: Vec<String>,
    search_query: Option<String>,
    max_videos: usize,
    client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct YoutubeSearchResponse {
    items: Vec<SearchItem>,
}

#[derive(Debug, Deserialize)]
struct SearchItem {
    id: VideoId,
    snippet: Snippet,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum VideoId {
    Video {
        #[serde(rename = "videoId")]
        video_id: String,
    },
    Channel {
        #[serde(rename = "channelId")]
        channel_id: String,
    },
    Playlist {
        #[serde(rename = "playlistId")]
        playlist_id: String,
    },
}

#[derive(Debug, Deserialize)]
struct Snippet {
    title: String,
    description: String,
    #[serde(rename = "channelTitle")]
    channel_title: String,
    #[serde(rename = "publishedAt")]
    published_at: String,
    thumbnails: Option<Thumbnails>,
}

#[derive(Debug, Deserialize)]
struct Thumbnails {
    default: Option<ThumbnailInfo>,
    medium: Option<ThumbnailInfo>,
    high: Option<ThumbnailInfo>,
}

#[derive(Debug, Deserialize)]
struct ThumbnailInfo {
    url: String,
}

#[derive(Debug, Deserialize)]
struct VideoDetailsResponse {
    items: Vec<VideoDetails>,
}

#[derive(Debug, Deserialize)]
struct VideoDetails {
    id: String,
    snippet: Snippet,
    statistics: Option<Statistics>,
    #[serde(rename = "contentDetails")]
    content_details: Option<ContentDetails>,
}

#[derive(Debug, Deserialize)]
struct Statistics {
    #[serde(rename = "viewCount")]
    view_count: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ContentDetails {
    duration: String,
}

impl YoutubeFetcher {
    pub fn new(
        api_key: String,
        channels: Vec<String>,
        search_query: Option<String>,
        max_videos: usize,
    ) -> Self {
        Self {
            api_key,
            channels,
            search_query,
            max_videos,
            client: reqwest::Client::new(),
        }
    }

    async fn search_videos(&self, query: &str) -> Result<Vec<YoutubeVideo>> {
        let url = format!(
            "{}/search?part=snippet&q={}&type=video&maxResults={}&key={}",
            YOUTUBE_API_BASE,
            urlencoding::encode(query),
            self.max_videos,
            self.api_key
        );

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "YouTube API error (status {}): {}",
                status,
                error_text
            ));
        }

        let search_response: YoutubeSearchResponse = response.json().await?;

        let video_ids: Vec<String> = search_response
            .items
            .iter()
            .filter_map(|item| {
                if let VideoId::Video { video_id } = &item.id {
                    Some(video_id.clone())
                } else {
                    None
                }
            })
            .collect();

        if video_ids.is_empty() {
            return Ok(vec![]);
        }

        self.get_video_details(&video_ids).await
    }

    async fn get_channel_videos(&self, channel_id: &str) -> Result<Vec<YoutubeVideo>> {
        let url = format!(
            "{}/search?part=snippet&channelId={}&type=video&order=date&maxResults={}&key={}",
            YOUTUBE_API_BASE, channel_id, self.max_videos, self.api_key
        );

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "YouTube API error (status {}): {}",
                status,
                error_text
            ));
        }

        let search_response: YoutubeSearchResponse = response.json().await?;

        let video_ids: Vec<String> = search_response
            .items
            .iter()
            .filter_map(|item| {
                if let VideoId::Video { video_id } = &item.id {
                    Some(video_id.clone())
                } else {
                    None
                }
            })
            .collect();

        if video_ids.is_empty() {
            return Ok(vec![]);
        }

        self.get_video_details(&video_ids).await
    }

    async fn get_video_details(&self, video_ids: &[String]) -> Result<Vec<YoutubeVideo>> {
        let ids_param = video_ids.join(",");
        let url = format!(
            "{}/videos?part=snippet,statistics,contentDetails&id={}&key={}",
            YOUTUBE_API_BASE, ids_param, self.api_key
        );

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "YouTube API error (status {}): {}",
                status,
                error_text
            ));
        }

        let details_response: VideoDetailsResponse = response.json().await?;

        Ok(details_response
            .items
            .into_iter()
            .map(|video| {
                let thumbnail_url = video
                    .snippet
                    .thumbnails
                    .and_then(|t| t.medium.or(t.high).or(t.default))
                    .map(|info| info.url);

                let view_count = video
                    .statistics
                    .and_then(|s| s.view_count)
                    .map(|v| format_view_count(&v));

                let duration = video
                    .content_details
                    .map(|cd| format_duration(&cd.duration));

                YoutubeVideo {
                    id: video.id,
                    title: video.snippet.title,
                    channel: video.snippet.channel_title,
                    published: format_published_date(&video.snippet.published_at),
                    description: truncate_description(&video.snippet.description),
                    thumbnail_url,
                    view_count,
                    duration,
                }
            })
            .collect())
    }
}

#[async_trait]
impl FeedFetcher for YoutubeFetcher {
    async fn fetch(&self) -> Result<FeedData> {
        let mut all_videos = Vec::new();

        // Fetch from search query if provided
        if let Some(query) = &self.search_query {
            match self.search_videos(query).await {
                Ok(mut videos) => all_videos.append(&mut videos),
                Err(e) => return Ok(FeedData::Error(format!("Search error: {}", e))),
            }
        }

        // Fetch from channels
        for channel_id in &self.channels {
            match self.get_channel_videos(channel_id).await {
                Ok(mut videos) => all_videos.append(&mut videos),
                Err(e) => {
                    eprintln!("Error fetching channel {}: {}", channel_id, e);
                    continue;
                }
            }
        }

        // Limit total videos
        all_videos.truncate(self.max_videos);

        if all_videos.is_empty() && self.search_query.is_none() && self.channels.is_empty() {
            return Ok(FeedData::Error(
                "No search query or channels configured".to_string(),
            ));
        }

        Ok(FeedData::Youtube(all_videos))
    }
}

fn format_view_count(count: &str) -> String {
    if let Ok(num) = count.parse::<u64>() {
        if num >= 1_000_000 {
            format!("{:.1}M views", num as f64 / 1_000_000.0)
        } else if num >= 1_000 {
            format!("{:.1}K views", num as f64 / 1_000.0)
        } else {
            format!("{} views", num)
        }
    } else {
        count.to_string()
    }
}

fn format_duration(iso_duration: &str) -> String {
    // Parse ISO 8601 duration (e.g., PT1H2M10S)
    let duration = iso_duration.trim_start_matches("PT");

    let mut hours = 0;
    let mut minutes = 0;
    let mut seconds = 0;

    let mut current = String::new();
    for ch in duration.chars() {
        if ch.is_ascii_digit() {
            current.push(ch);
        } else {
            let value: u32 = current.parse().unwrap_or(0);
            match ch {
                'H' => hours = value,
                'M' => minutes = value,
                'S' => seconds = value,
                _ => {}
            }
            current.clear();
        }
    }

    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{}:{:02}", minutes, seconds)
    }
}

fn format_published_date(iso_date: &str) -> String {
    // Simple formatting - just extract date portion
    iso_date.split('T').next().unwrap_or(iso_date).to_string()
}
fn truncate_description(desc: &str) -> String {
    let char_count = desc.chars().count();
    if char_count > 100 {
        let truncated: String = desc.chars().take(97).collect();
        format!("{}...", truncated)
    } else {
        desc.to_string()
    }
}
