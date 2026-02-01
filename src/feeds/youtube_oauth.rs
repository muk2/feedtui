//! YouTube OAuth 2.0 authentication module
//!
//! Handles OAuth device flow for CLI/TUI applications and token management.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const GOOGLE_DEVICE_AUTH_URL: &str = "https://oauth2.googleapis.com/device/code";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

/// OAuth tokens for YouTube API access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: u64, // Unix timestamp
    pub token_type: String,
}

impl OAuthTokens {
    /// Check if the access token is expired (with 5 minute buffer)
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now >= self.expires_at.saturating_sub(300) // 5 minute buffer
    }
}

/// Device authorization response from Google
#[derive(Debug, Deserialize)]
pub struct DeviceAuthResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_url: String,
    pub expires_in: u64,
    pub interval: u64,
}

/// Token response from Google
#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: u64,
    token_type: String,
}

/// Error response from token endpoint
#[derive(Debug, Deserialize)]
struct TokenError {
    error: String,
    error_description: Option<String>,
}

/// YouTube OAuth manager
pub struct YouTubeOAuth {
    client_id: String,
    client_secret: String,
    client: reqwest::Client,
    tokens_path: PathBuf,
}

impl YouTubeOAuth {
    pub fn new(client_id: String, client_secret: String) -> Self {
        let tokens_path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".feedtui")
            .join("youtube_tokens.json");

        Self {
            client_id,
            client_secret,
            client: reqwest::Client::new(),
            tokens_path,
        }
    }

    /// Load saved tokens from disk
    pub fn load_tokens(&self) -> Option<OAuthTokens> {
        std::fs::read_to_string(&self.tokens_path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
    }

    /// Save tokens to disk
    pub fn save_tokens(&self, tokens: &OAuthTokens) -> Result<()> {
        if let Some(parent) = self.tokens_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(tokens)?;
        std::fs::write(&self.tokens_path, content)?;
        Ok(())
    }

    /// Get a valid access token, refreshing if necessary
    pub async fn get_access_token(&self) -> Result<String> {
        let tokens = self.load_tokens().ok_or_else(|| {
            anyhow!("No OAuth tokens found. Run 'feedtui youtube-auth' to authenticate.")
        })?;

        if tokens.is_expired() {
            let refreshed = self.refresh_tokens(&tokens.refresh_token).await?;
            self.save_tokens(&refreshed)?;
            Ok(refreshed.access_token)
        } else {
            Ok(tokens.access_token)
        }
    }

    /// Start the device authorization flow
    /// Returns the device auth response containing the user code to display
    pub async fn start_device_flow(&self) -> Result<DeviceAuthResponse> {
        let params = [
            ("client_id", self.client_id.as_str()),
            ("scope", "https://www.googleapis.com/auth/youtube.readonly"),
        ];

        let response = self
            .client
            .post(GOOGLE_DEVICE_AUTH_URL)
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Device auth failed: {}", error_text));
        }

        let auth_response: DeviceAuthResponse = response.json().await?;
        Ok(auth_response)
    }

    /// Poll for the token after user authorizes the device
    pub async fn poll_for_token(&self, device_code: &str, interval: u64) -> Result<OAuthTokens> {
        let params = [
            ("client_id", self.client_id.as_str()),
            ("client_secret", self.client_secret.as_str()),
            ("device_code", device_code),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
        ];

        loop {
            tokio::time::sleep(Duration::from_secs(interval)).await;

            let response = self
                .client
                .post(GOOGLE_TOKEN_URL)
                .form(&params)
                .send()
                .await?;

            let status = response.status();
            let body = response.text().await?;

            if status.is_success() {
                let token_response: TokenResponse = serde_json::from_str(&body)?;
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                let tokens = OAuthTokens {
                    access_token: token_response.access_token,
                    refresh_token: token_response
                        .refresh_token
                        .unwrap_or_else(|| String::new()),
                    expires_at: now + token_response.expires_in,
                    token_type: token_response.token_type,
                };

                self.save_tokens(&tokens)?;
                return Ok(tokens);
            }

            // Check for pending or error
            if let Ok(error) = serde_json::from_str::<TokenError>(&body) {
                match error.error.as_str() {
                    "authorization_pending" => continue,
                    "slow_down" => {
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        continue;
                    }
                    "access_denied" => return Err(anyhow!("User denied access")),
                    "expired_token" => return Err(anyhow!("Device code expired")),
                    _ => {
                        return Err(anyhow!(
                            "Token error: {} - {}",
                            error.error,
                            error.error_description.unwrap_or_default()
                        ))
                    }
                }
            }
        }
    }

    /// Refresh the access token using the refresh token
    async fn refresh_tokens(&self, refresh_token: &str) -> Result<OAuthTokens> {
        let params = [
            ("client_id", self.client_id.as_str()),
            ("client_secret", self.client_secret.as_str()),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
        ];

        let response = self
            .client
            .post(GOOGLE_TOKEN_URL)
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Token refresh failed: {}", error_text));
        }

        let token_response: TokenResponse = response.json().await?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Ok(OAuthTokens {
            access_token: token_response.access_token,
            refresh_token: token_response
                .refresh_token
                .unwrap_or_else(|| refresh_token.to_string()),
            expires_at: now + token_response.expires_in,
            token_type: token_response.token_type,
        })
    }

    /// Check if we have valid saved tokens
    pub fn has_valid_tokens(&self) -> bool {
        self.load_tokens().map(|t| !t.is_expired()).unwrap_or(false)
    }

    /// Delete saved tokens
    pub fn clear_tokens(&self) -> Result<()> {
        if self.tokens_path.exists() {
            std::fs::remove_file(&self.tokens_path)?;
        }
        Ok(())
    }
}
