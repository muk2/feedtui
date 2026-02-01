use super::youtube_oauth::YouTubeOAuth;
use super::{FeedData, FeedFetcher, YoutubeVideo};
use crate::config::YoutubeFeedType;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::Deserialize;

const YOUTUBE_API_BASE: &str = "https://www.googleapis.com/youtube/v3";

pub struct YoutubeFetcher {
    api_key: Option<String>,
    channels: Vec<String>,
    search_query: Option<String>,
    max_videos: usize,
    client: reqwest::Client,
    feed_type: YoutubeFeedType,
    oauth: Option<YouTubeOAuth>,
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
        api_key: Option<String>,
        channels: Vec<String>,
        search_query: Option<String>,
        max_videos: usize,
        feed_type: YoutubeFeedType,
        client_id: Option<String>,
        client_secret: Option<String>,
    ) -> Self {
        let oauth = match (&client_id, &client_secret) {
            (Some(id), Some(secret)) => Some(YouTubeOAuth::new(id.clone(), secret.clone())),
            _ => None,
        };

        Self {
            api_key,
            channels,
            search_query,
            max_videos,
            client: reqwest::Client::new(),
            feed_type,
            oauth,
        }
    }

    /// Get the API key, required for public API access
    fn get_api_key(&self) -> Result<&str> {
        self.api_key
            .as_deref()
            .ok_or_else(|| anyhow!("API key required for public YouTube access"))
    }

    async fn search_videos(&self, query: &str) -> Result<Vec<YoutubeVideo>> {
        let api_key = self.get_api_key()?;
        let url = format!(
            "{}/search?part=snippet&q={}&type=video&maxResults={}&key={}",
            YOUTUBE_API_BASE,
            urlencoding::encode(query),
            self.max_videos,
            api_key
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
        let api_key = self.get_api_key()?;
        let url = format!(
            "{}/search?part=snippet&channelId={}&type=video&order=date&maxResults={}&key={}",
            YOUTUBE_API_BASE, channel_id, self.max_videos, api_key
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

        // Use OAuth token if available, otherwise API key
        let url = if let Some(ref oauth) = self.oauth {
            if oauth.has_valid_tokens() {
                format!(
                    "{}/videos?part=snippet,statistics,contentDetails&id={}",
                    YOUTUBE_API_BASE, ids_param
                )
            } else {
                let api_key = self.get_api_key()?;
                format!(
                    "{}/videos?part=snippet,statistics,contentDetails&id={}&key={}",
                    YOUTUBE_API_BASE, ids_param, api_key
                )
            }
        } else {
            let api_key = self.get_api_key()?;
            format!(
                "{}/videos?part=snippet,statistics,contentDetails&id={}&key={}",
                YOUTUBE_API_BASE, ids_param, api_key
            )
        };

        let response = if let Some(ref oauth) = self.oauth {
            if oauth.has_valid_tokens() {
                let token = oauth.get_access_token().await?;
                self.client
                    .get(&url)
                    .header("Authorization", format!("Bearer {}", token))
                    .send()
                    .await?
            } else {
                self.client.get(&url).send().await?
            }
        } else {
            self.client.get(&url).send().await?
        };

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

    /// Fetch videos from user's subscriptions (requires OAuth)
    async fn get_subscriptions_feed(&self) -> Result<Vec<YoutubeVideo>> {
        let oauth = self.oauth.as_ref().ok_or_else(|| {
            anyhow!("OAuth not configured. Set client_id and client_secret in config.")
        })?;

        let token = oauth.get_access_token().await?;

        // First, get the user's subscribed channels
        let subs_url = format!(
            "{}/subscriptions?part=snippet&mine=true&maxResults=25",
            YOUTUBE_API_BASE
        );

        let response = self
            .client
            .get(&subs_url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "YouTube API error (status {}): {}",
                status,
                error_text
            ));
        }

        #[derive(Deserialize)]
        struct SubscriptionsResponse {
            items: Vec<SubscriptionItem>,
        }
        #[derive(Deserialize)]
        struct SubscriptionItem {
            snippet: SubscriptionSnippet,
        }
        #[derive(Deserialize)]
        struct SubscriptionSnippet {
            #[serde(rename = "resourceId")]
            resource_id: ResourceId,
        }
        #[derive(Deserialize)]
        struct ResourceId {
            #[serde(rename = "channelId")]
            channel_id: String,
        }

        let subs_response: SubscriptionsResponse = response.json().await?;
        let channel_ids: Vec<String> = subs_response
            .items
            .into_iter()
            .map(|item| item.snippet.resource_id.channel_id)
            .collect();

        if channel_ids.is_empty() {
            return Ok(vec![]);
        }

        // Get recent videos from subscribed channels using activities endpoint
        let mut all_videos = Vec::new();
        for channel_id in channel_ids.iter().take(10) {
            // Limit to avoid quota issues
            match self.get_channel_videos_oauth(&token, channel_id).await {
                Ok(mut videos) => all_videos.append(&mut videos),
                Err(_) => continue,
            }
        }

        all_videos.sort_by(|a, b| b.published.cmp(&a.published)); // Most recent first
        all_videos.truncate(self.max_videos);
        Ok(all_videos)
    }

    /// Get channel videos using OAuth token
    async fn get_channel_videos_oauth(
        &self,
        token: &str,
        channel_id: &str,
    ) -> Result<Vec<YoutubeVideo>> {
        let url = format!(
            "{}/search?part=snippet&channelId={}&type=video&order=date&maxResults=5",
            YOUTUBE_API_BASE, channel_id
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to fetch channel videos"));
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

    /// Fetch user's liked videos playlist (requires OAuth)
    async fn get_liked_videos(&self) -> Result<Vec<YoutubeVideo>> {
        let oauth = self.oauth.as_ref().ok_or_else(|| {
            anyhow!("OAuth not configured. Set client_id and client_secret in config.")
        })?;

        let token = oauth.get_access_token().await?;

        // The liked videos playlist ID is "LL" for the authenticated user
        let url = format!(
            "{}/playlistItems?part=snippet&playlistId=LL&maxResults={}",
            YOUTUBE_API_BASE, self.max_videos
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "YouTube API error (status {}): {}",
                status,
                error_text
            ));
        }

        #[derive(Deserialize)]
        struct PlaylistResponse {
            items: Vec<PlaylistItem>,
        }
        #[derive(Deserialize)]
        struct PlaylistItem {
            snippet: PlaylistSnippet,
        }
        #[derive(Deserialize)]
        struct PlaylistSnippet {
            #[serde(rename = "resourceId")]
            resource_id: PlaylistResourceId,
        }
        #[derive(Deserialize)]
        struct PlaylistResourceId {
            #[serde(rename = "videoId")]
            video_id: String,
        }

        let playlist_response: PlaylistResponse = response.json().await?;
        let video_ids: Vec<String> = playlist_response
            .items
            .into_iter()
            .map(|item| item.snippet.resource_id.video_id)
            .collect();

        if video_ids.is_empty() {
            return Ok(vec![]);
        }

        self.get_video_details(&video_ids).await
    }

    /// Fetch user's Watch Later playlist (requires OAuth)
    async fn get_watch_later(&self) -> Result<Vec<YoutubeVideo>> {
        let oauth = self.oauth.as_ref().ok_or_else(|| {
            anyhow!("OAuth not configured. Set client_id and client_secret in config.")
        })?;

        let token = oauth.get_access_token().await?;

        // First, get the user's channel to find the Watch Later playlist ID
        let channels_url = format!(
            "{}/channels?part=contentDetails&mine=true",
            YOUTUBE_API_BASE
        );

        let response = self
            .client
            .get(&channels_url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to get channel info"));
        }

        #[derive(Deserialize)]
        struct ChannelsResponse {
            items: Vec<ChannelItem>,
        }
        #[derive(Deserialize)]
        struct ChannelItem {
            #[serde(rename = "contentDetails")]
            content_details: ChannelContentDetails,
        }
        #[derive(Deserialize)]
        struct ChannelContentDetails {
            #[serde(rename = "relatedPlaylists")]
            related_playlists: RelatedPlaylists,
        }
        #[derive(Deserialize)]
        struct RelatedPlaylists {
            #[serde(rename = "watchLater")]
            watch_later: Option<String>,
        }

        let channels_response: ChannelsResponse = response.json().await?;
        let watch_later_id = channels_response
            .items
            .first()
            .and_then(|c| c.content_details.related_playlists.watch_later.clone())
            .ok_or_else(|| anyhow!("Watch Later playlist not found"))?;

        // Fetch the Watch Later playlist items
        let url = format!(
            "{}/playlistItems?part=snippet&playlistId={}&maxResults={}",
            YOUTUBE_API_BASE, watch_later_id, self.max_videos
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to fetch Watch Later playlist"));
        }

        #[derive(Deserialize)]
        struct PlaylistResponse {
            items: Vec<PlaylistItem>,
        }
        #[derive(Deserialize)]
        struct PlaylistItem {
            snippet: PlaylistSnippet,
        }
        #[derive(Deserialize)]
        struct PlaylistSnippet {
            #[serde(rename = "resourceId")]
            resource_id: PlaylistResourceId,
        }
        #[derive(Deserialize)]
        struct PlaylistResourceId {
            #[serde(rename = "videoId")]
            video_id: String,
        }

        let playlist_response: PlaylistResponse = response.json().await?;
        let video_ids: Vec<String> = playlist_response
            .items
            .into_iter()
            .map(|item| item.snippet.resource_id.video_id)
            .collect();

        if video_ids.is_empty() {
            return Ok(vec![]);
        }

        self.get_video_details(&video_ids).await
    }
}

#[async_trait]
impl FeedFetcher for YoutubeFetcher {
    async fn fetch(&self) -> Result<FeedData> {
        // Handle different feed types
        match self.feed_type {
            YoutubeFeedType::Subscriptions => match self.get_subscriptions_feed().await {
                Ok(videos) if videos.is_empty() => {
                    Ok(FeedData::Error("No subscription videos found".to_string()))
                }
                Ok(videos) => Ok(FeedData::Youtube(videos)),
                Err(e) => Ok(FeedData::Error(format!("Subscriptions error: {}", e))),
            },
            YoutubeFeedType::LikedVideos => match self.get_liked_videos().await {
                Ok(videos) if videos.is_empty() => {
                    Ok(FeedData::Error("No liked videos found".to_string()))
                }
                Ok(videos) => Ok(FeedData::Youtube(videos)),
                Err(e) => Ok(FeedData::Error(format!("Liked videos error: {}", e))),
            },
            YoutubeFeedType::WatchLater => match self.get_watch_later().await {
                Ok(videos) if videos.is_empty() => {
                    Ok(FeedData::Error("Watch Later is empty".to_string()))
                }
                Ok(videos) => Ok(FeedData::Youtube(videos)),
                Err(e) => Ok(FeedData::Error(format!("Watch Later error: {}", e))),
            },
            YoutubeFeedType::Public => {
                // Original public feed behavior
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

                if all_videos.is_empty() && self.search_query.is_none() && self.channels.is_empty()
                {
                    return Ok(FeedData::Error(
                        "No search query or channels configured".to_string(),
                    ));
                }

                Ok(FeedData::Youtube(all_videos))
            }
        }
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
