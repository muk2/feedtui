use super::{FeedData, FeedFetcher, TwitterArchiveItem};
use anyhow::Result;
use async_trait::async_trait;
use futures::stream::{self, StreamExt};
use scraper::{Html, Selector};
use std::time::Duration;

pub struct TwitterArchiveFetcher {
    archive_query: String,
    max_items: usize,
    client: reqwest::Client,
}

impl TwitterArchiveFetcher {
    pub fn new(archive_query: String, max_items: usize) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .user_agent("Mozilla/5.0 (compatible; feedtui/1.0; +https://github.com/muk2/feedtui)")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            archive_query,
            max_items,
            client,
        }
    }

    /// Fetch capture records from the Wayback Machine CDX API.
    /// Returns JSON rows: [urlkey, timestamp, original, mimetype, statuscode, digest, length]
    async fn fetch_cdx_records(&self) -> Result<Vec<TwitterArchiveItem>> {
        // Ensure the query targets tweet URLs specifically (/status/*)
        // so we don't waste our limit on profile/media pages
        let query = if self.archive_query.contains("/status") {
            self.archive_query.clone()
        } else {
            let base = self
                .archive_query
                .trim_end_matches('*')
                .trim_end_matches('/');
            format!("{}/status/*", base)
        };

        let url = format!(
            "https://web.archive.org/cdx/search/cdx?url={}&output=json&limit={}&fl=timestamp,original,statuscode&filter=statuscode:200&collapse=urlkey",
            urlencoding::encode(&query),
            self.max_items + 1, // +1 because first row is the header
        );

        let response = self.client.get(&url).send().await?;

        let body = response.text().await?;
        let rows: Vec<Vec<String>> = serde_json::from_str(&body)?;

        // First row is the header: ["timestamp", "original", "statuscode"]
        let items: Vec<TwitterArchiveItem> = rows
            .into_iter()
            .skip(1) // skip header row
            .filter_map(|row| {
                if row.len() < 3 {
                    return None;
                }

                let timestamp = &row[0];
                let original = &row[1];

                // Filter to only clean tweet URLs (contain /status/<id>)
                // Skip bare /status pages, URLs with query params or encoding artifacts
                if !original.contains("/status/") {
                    return None;
                }
                // Extract the part after /status/ and check it starts with a digit
                let after_status = original.split("/status/").nth(1).unwrap_or("");
                let tweet_id = after_status
                    .split(&['?', '%', '#', '"'][..])
                    .next()
                    .unwrap_or("");
                if tweet_id.is_empty()
                    || !tweet_id.chars().next().is_some_and(|c| c.is_ascii_digit())
                {
                    return None;
                }

                // Use if_ modifier to get raw original HTML without Wayback toolbar
                let archive_url =
                    format!("https://web.archive.org/web/{}if_/{}", timestamp, original);

                // Parse timestamp (format: YYYYMMDDHHmmss) into readable date
                let date_display = format_wayback_timestamp(timestamp);

                // Extract author from URL pattern twitter.com/{author}/status/...
                let author = extract_author_from_url(original);

                Some(TwitterArchiveItem {
                    timestamp: timestamp.clone(),
                    original_url: original.clone(),
                    archive_url,
                    tweet_text: None,
                    author,
                    date_display,
                })
            })
            .take(self.max_items)
            .collect();

        Ok(items)
    }
}

/// Fetch the archived HTML page and extract tweet text using multiple strategies.
async fn fetch_tweet_text_with_client(
    client: &reqwest::Client,
    archive_url: &str,
) -> Option<String> {
    let response = match client.get(archive_url).send().await {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("Failed to fetch archive page {}: {}", archive_url, e);
            return None;
        }
    };

    if !response.status().is_success() {
        eprintln!(
            "Archive page returned HTTP {}: {}",
            response.status(),
            archive_url
        );
        return None;
    }

    let html = match response.text().await {
        Ok(text) => text,
        Err(e) => {
            eprintln!("Failed to read response body from {}: {}", archive_url, e);
            return None;
        }
    };

    extract_tweet_text(&html)
}

/// Extract tweet text from archived HTML using a cascade of strategies.
fn extract_tweet_text(html: &str) -> Option<String> {
    let document = Html::parse_document(html);

    // Strategy 1: og:description meta tag
    if let Some(text) = extract_meta_content(&document, r#"meta[property="og:description"]"#) {
        return Some(text);
    }

    // Strategy 2: twitter:description meta tag
    if let Some(text) = extract_meta_content(&document, r#"meta[name="twitter:description"]"#) {
        return Some(text);
    }

    // Strategy 3: generic description meta tag (filter boilerplate)
    if let Some(text) = extract_meta_content(&document, r#"meta[name="description"]"#) {
        let lower = text.to_lowercase();
        // Skip generic Twitter/X boilerplate descriptions
        if !lower.starts_with("from breaking news")
            && !lower.starts_with("the latest posts")
            && !lower.starts_with("post with")
            && !lower.contains("on twitter")
            && !lower.contains("on x")
            && text.len() > 20
        {
            return Some(text);
        }
    }

    // Strategy 4: p.tweet-text element (old Twitter DOM, pre-2018 archives)
    if let Some(text) = extract_element_text(&document, "p.tweet-text") {
        return Some(text);
    }

    // Strategy 5: JSON-LD articleBody
    if let Some(text) = extract_json_ld_article_body(&document) {
        return Some(text);
    }

    None
}

/// Extract the `content` attribute from a meta tag matching the given CSS selector.
fn extract_meta_content(document: &Html, selector_str: &str) -> Option<String> {
    let selector = Selector::parse(selector_str).ok()?;
    let element = document.select(&selector).next()?;
    let content = element.value().attr("content")?;
    let text = content.trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

/// Extract the inner text content from the first element matching the given CSS selector.
fn extract_element_text(document: &Html, selector_str: &str) -> Option<String> {
    let selector = Selector::parse(selector_str).ok()?;
    let element = document.select(&selector).next()?;
    let text: String = element
        .text()
        .collect::<Vec<_>>()
        .join("")
        .trim()
        .to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

/// Extract `articleBody` from JSON-LD structured data in `<script type="application/ld+json">`.
fn extract_json_ld_article_body(document: &Html) -> Option<String> {
    let selector = Selector::parse(r#"script[type="application/ld+json"]"#).ok()?;

    for element in document.select(&selector) {
        let json_text: String = element.text().collect();
        // Try parsing as a single object
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&json_text) {
            if let Some(body) = extract_article_body_from_value(&val) {
                return Some(body);
            }
            // Could be an array of objects
            if let Some(arr) = val.as_array() {
                for item in arr {
                    if let Some(body) = extract_article_body_from_value(item) {
                        return Some(body);
                    }
                }
            }
        }
    }

    None
}

fn extract_article_body_from_value(val: &serde_json::Value) -> Option<String> {
    let body = val.get("articleBody")?.as_str()?;
    let text = body.trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

/// Parse Wayback Machine timestamp (YYYYMMDDHHmmss) into a human-readable date.
fn format_wayback_timestamp(ts: &str) -> String {
    if ts.len() >= 8 {
        let year = &ts[0..4];
        let month = &ts[4..6];
        let day = &ts[6..8];
        let time = if ts.len() >= 12 {
            format!(" {}:{}", &ts[8..10], &ts[10..12])
        } else {
            String::new()
        };
        format!("{}-{}-{}{}", year, month, day, time)
    } else {
        ts.to_string()
    }
}

/// Extract the Twitter author handle from a URL like twitter.com/username/status/...
fn extract_author_from_url(url: &str) -> Option<String> {
    // Handle both http and https, with or without www
    let path = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);

    let path = path.strip_prefix("www.").unwrap_or(path);

    let path = path
        .strip_prefix("twitter.com/")
        .or_else(|| path.strip_prefix("x.com/"))?;

    let username = path.split('/').next()?;
    if username.is_empty() {
        None
    } else {
        Some(format!("@{}", username))
    }
}

#[async_trait]
impl FeedFetcher for TwitterArchiveFetcher {
    async fn fetch(&self) -> Result<FeedData> {
        let mut items = self.fetch_cdx_records().await?;

        // Fetch tweet text concurrently (up to 3 at a time to avoid rate limiting)
        let urls: Vec<String> = items.iter().map(|item| item.archive_url.clone()).collect();
        let client = self.client.clone();
        let texts: Vec<Option<String>> = stream::iter(urls.into_iter().map(move |url| {
            let client = client.clone();
            async move { fetch_tweet_text_with_client(&client, &url).await }
        }))
        .buffer_unordered(3)
        .collect()
        .await;

        for (item, text) in items.iter_mut().zip(texts) {
            item.tweet_text = text;
        }

        Ok(FeedData::TwitterArchive(items))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_wayback_timestamp_full() {
        assert_eq!(
            format_wayback_timestamp("20230615143022"),
            "2023-06-15 14:30"
        );
    }

    #[test]
    fn test_format_wayback_timestamp_date_only() {
        assert_eq!(format_wayback_timestamp("20230615"), "2023-06-15");
    }

    #[test]
    fn test_format_wayback_timestamp_short() {
        assert_eq!(format_wayback_timestamp("2023"), "2023");
    }

    #[test]
    fn test_extract_author_https() {
        assert_eq!(
            extract_author_from_url("https://twitter.com/gethigher77/status/123456"),
            Some("@gethigher77".to_string())
        );
    }

    #[test]
    fn test_extract_author_http() {
        assert_eq!(
            extract_author_from_url("http://twitter.com/someuser/status/789"),
            Some("@someuser".to_string())
        );
    }

    #[test]
    fn test_extract_author_x_dot_com() {
        assert_eq!(
            extract_author_from_url("https://x.com/testuser/status/111"),
            Some("@testuser".to_string())
        );
    }

    #[test]
    fn test_extract_author_www() {
        assert_eq!(
            extract_author_from_url("https://www.twitter.com/handle/status/999"),
            Some("@handle".to_string())
        );
    }

    #[test]
    fn test_extract_author_no_match() {
        assert_eq!(extract_author_from_url("https://example.com/page"), None);
    }

    #[test]
    fn test_extract_author_empty_username() {
        assert_eq!(extract_author_from_url("https://twitter.com/"), None);
    }

    #[test]
    fn test_fetcher_new() {
        let fetcher = TwitterArchiveFetcher::new("twitter.com/test*".to_string(), 10);
        assert_eq!(fetcher.archive_query, "twitter.com/test*");
        assert_eq!(fetcher.max_items, 10);
    }

    #[test]
    fn test_extract_tweet_text_og_description() {
        let html = r#"<html><head><meta property="og:description" content="Hello world, this is a tweet!"></head><body></body></html>"#;
        assert_eq!(
            extract_tweet_text(html),
            Some("Hello world, this is a tweet!".to_string())
        );
    }

    #[test]
    fn test_extract_tweet_text_twitter_description() {
        let html = r#"<html><head><meta name="twitter:description" content="Tweet via twitter card"></head><body></body></html>"#;
        assert_eq!(
            extract_tweet_text(html),
            Some("Tweet via twitter card".to_string())
        );
    }

    #[test]
    fn test_extract_tweet_text_meta_description() {
        let html = r#"<html><head><meta name="description" content="This is a sufficiently long tweet text from the description tag"></head><body></body></html>"#;
        assert_eq!(
            extract_tweet_text(html),
            Some("This is a sufficiently long tweet text from the description tag".to_string())
        );
    }

    #[test]
    fn test_extract_tweet_text_meta_description_boilerplate_filtered() {
        let html = r#"<html><head><meta name="description" content="From breaking news and entertainment to sports"></head><body></body></html>"#;
        assert_eq!(extract_tweet_text(html), None);
    }

    #[test]
    fn test_extract_tweet_text_p_tweet_text() {
        let html = r#"<html><body><div class="tweet"><p class="tweet-text">Old school tweet text here</p></div></body></html>"#;
        assert_eq!(
            extract_tweet_text(html),
            Some("Old school tweet text here".to_string())
        );
    }

    #[test]
    fn test_extract_tweet_text_json_ld() {
        let html = r#"<html><head><script type="application/ld+json">{"@type":"SocialMediaPosting","articleBody":"Tweet from JSON-LD"}</script></head><body></body></html>"#;
        assert_eq!(
            extract_tweet_text(html),
            Some("Tweet from JSON-LD".to_string())
        );
    }

    #[test]
    fn test_extract_tweet_text_og_wins_over_p_tweet_text() {
        let html = r#"<html><head><meta property="og:description" content="OG wins"></head><body><p class="tweet-text">Should not be picked</p></body></html>"#;
        assert_eq!(extract_tweet_text(html), Some("OG wins".to_string()));
    }

    #[test]
    fn test_extract_tweet_text_empty_og_falls_through() {
        let html = r#"<html><head><meta property="og:description" content=""><meta name="twitter:description" content="Fallback to twitter card"></head><body></body></html>"#;
        assert_eq!(
            extract_tweet_text(html),
            Some("Fallback to twitter card".to_string())
        );
    }

    #[test]
    fn test_extract_tweet_text_no_matches() {
        let html = r#"<html><head><title>Nothing here</title></head><body><p>Just a paragraph</p></body></html>"#;
        assert_eq!(extract_tweet_text(html), None);
    }

    #[test]
    fn test_extract_tweet_text_malformed_json_ld() {
        let html = r#"<html><head><script type="application/ld+json">{not valid json</script></head><body></body></html>"#;
        assert_eq!(extract_tweet_text(html), None);
    }
}
