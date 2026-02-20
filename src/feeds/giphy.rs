use super::{FeedData, FeedFetcher, GiphyGif};
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;

pub struct GiphyFetcher {
    api_key: String,
    query: Option<String>,
    mode: GiphyMode,
    client: reqwest::Client,
}

#[derive(Debug, Clone)]
pub enum GiphyMode {
    Search,
    Trending,
    Random,
}

#[derive(Debug, Deserialize)]
struct GiphyResponse {
    data: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct GiphyGifData {
    id: String,
    title: String,
    images: GiphyImages,
}

#[derive(Debug, Deserialize)]
struct GiphyImages {
    fixed_height_small: GiphyImageVariant,
}

#[derive(Debug, Deserialize)]
struct GiphyImageVariant {
    url: String,
    width: String,
    height: String,
}

impl GiphyFetcher {
    pub fn new(api_key: String, query: Option<String>, mode: GiphyMode) -> Self {
        Self {
            api_key,
            query,
            mode,
            client: reqwest::Client::new(),
        }
    }

    async fn fetch_gif_url(&self) -> Result<Option<GiphyGif>> {
        let url = match self.mode {
            GiphyMode::Search => {
                let q = self.query.as_deref().unwrap_or("cat");
                format!(
                    "https://api.giphy.com/v1/gifs/search?api_key={}&q={}&limit=1&rating=g",
                    self.api_key,
                    urlencoding::encode(q)
                )
            }
            GiphyMode::Trending => {
                format!(
                    "https://api.giphy.com/v1/gifs/trending?api_key={}&limit=1&rating=g",
                    self.api_key
                )
            }
            GiphyMode::Random => {
                let tag = self.query.as_deref().unwrap_or("");
                format!(
                    "https://api.giphy.com/v1/gifs/random?api_key={}&tag={}&rating=g",
                    self.api_key,
                    urlencoding::encode(tag)
                )
            }
        };

        let response = self.client.get(&url).send().await?;
        let giphy_resp: GiphyResponse = response.json().await?;

        // Parse based on mode (random returns a single object, others return arrays)
        let gif_data: Option<GiphyGifData> = match self.mode {
            GiphyMode::Random => serde_json::from_value(giphy_resp.data).ok(),
            _ => {
                let arr: Vec<GiphyGifData> =
                    serde_json::from_value(giphy_resp.data).unwrap_or_default();
                arr.into_iter().next()
            }
        };

        let Some(gif) = gif_data else {
            return Ok(None);
        };

        let width = gif.images.fixed_height_small.width.parse().unwrap_or(100);
        let height = gif.images.fixed_height_small.height.parse().unwrap_or(100);

        // Download the GIF and decode frames into ASCII
        let gif_url = &gif.images.fixed_height_small.url;
        let gif_bytes = self.client.get(gif_url).send().await?.bytes().await?;
        let frames = decode_gif_to_ascii(&gif_bytes, width, height);

        Ok(Some(GiphyGif {
            id: gif.id,
            title: gif.title,
            frames,
        }))
    }
}

/// Decode a GIF byte stream into a vector of ASCII art frames
fn decode_gif_to_ascii(data: &[u8], target_width: u32, target_height: u32) -> Vec<Vec<String>> {
    use gif::DecodeOptions;
    use std::io::Cursor;

    let ascii_chars: &[u8] = b" .:-=+*#%@";

    let mut opts = DecodeOptions::new();
    opts.set_color_output(gif::ColorOutput::RGBA);

    let Ok(mut decoder) = opts.read_info(Cursor::new(data)) else {
        return vec![vec!["Error decoding GIF".to_string()]];
    };

    let gif_width = decoder.width() as u32;
    let gif_height = decoder.height() as u32;

    // Scale to fit a reasonable terminal size (max ~60 cols, ~20 rows)
    let max_cols = target_width.min(60);
    let max_rows = target_height.min(20);

    let scale_x = if gif_width > 0 {
        gif_width / max_cols.max(1)
    } else {
        1
    }
    .max(1);
    let scale_y = if gif_height > 0 {
        gif_height / max_rows.max(1)
    } else {
        1
    }
    .max(1);

    let out_w = gif_width / scale_x;
    let out_h = gif_height / scale_y;

    let mut frames = Vec::new();
    let max_frames = 30; // Cap frames to prevent memory issues

    while let Ok(Some(frame)) = decoder.read_next_frame() {
        if frames.len() >= max_frames {
            break;
        }

        let buf = &frame.buffer;
        let frame_w = frame.width as u32;
        let frame_h = frame.height as u32;

        let mut lines = Vec::with_capacity(out_h as usize);

        for row in 0..out_h {
            let mut line = String::with_capacity(out_w as usize);
            for col in 0..out_w {
                let src_x = (col * scale_x).min(frame_w.saturating_sub(1));
                let src_y = (row * scale_y).min(frame_h.saturating_sub(1));
                let idx = ((src_y * frame_w + src_x) * 4) as usize;

                if idx + 2 < buf.len() {
                    let r = buf[idx] as f32;
                    let g = buf[idx + 1] as f32;
                    let b = buf[idx + 2] as f32;
                    // Luminance formula
                    let luminance = 0.299 * r + 0.587 * g + 0.114 * b;
                    let char_idx = ((luminance / 255.0) * (ascii_chars.len() - 1) as f32) as usize;
                    line.push(ascii_chars[char_idx.min(ascii_chars.len() - 1)] as char);
                } else {
                    line.push(' ');
                }
            }
            lines.push(line);
        }

        frames.push(lines);
    }

    if frames.is_empty() {
        vec![vec!["No frames decoded".to_string()]]
    } else {
        frames
    }
}

#[async_trait]
impl FeedFetcher for GiphyFetcher {
    async fn fetch(&self) -> Result<FeedData> {
        match self.fetch_gif_url().await {
            Ok(Some(gif)) => Ok(FeedData::Giphy(gif)),
            Ok(None) => Ok(FeedData::Error("No GIF found".to_string())),
            Err(e) => Ok(FeedData::Error(format!("Giphy error: {}", e))),
        }
    }
}
