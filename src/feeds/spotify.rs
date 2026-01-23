use super::{FeedData, FeedFetcher, SpotifyPlayback};
use anyhow::Result;
use async_trait::async_trait;
use rspotify::{
    AuthCodeSpotify, Credentials, OAuth,
    clients::BaseClient,
    model::{PlayableItem, RepeatState},
    prelude::*,
};

pub struct SpotifyFetcher {
    client: AuthCodeSpotify,
}

impl SpotifyFetcher {
    pub fn new(client_id: String, client_secret: String, refresh_token: String) -> Self {
        let creds = Credentials::new(&client_id, &client_secret);
        let oauth = OAuth {
            redirect_uri: "http://localhost:8888/callback".to_string(),
            scopes: rspotify::scopes!(
                "user-read-playback-state",
                "user-modify-playback-state",
                "user-read-currently-playing"
            ),
            ..Default::default()
        };

        let mut client = AuthCodeSpotify::new(creds, oauth);

        // Set the refresh token
        if let Ok(mut token) = client.token.lock() {
            *token = Some(rspotify::Token {
                refresh_token: Some(refresh_token),
                ..Default::default()
            });
        }

        Self { client }
    }

    pub async fn play_pause(&self) -> Result<()> {
        if let Some(context) = self.client.current_playback(None, None::<Vec<_>>).await? {
            if context.is_playing {
                self.client.pause_playback(None).await?;
            } else {
                self.client.resume_playback(None, None).await?;
            }
        }
        Ok(())
    }

    pub async fn next_track(&self) -> Result<()> {
        self.client.next_track(None).await?;
        Ok(())
    }

    pub async fn previous_track(&self) -> Result<()> {
        self.client.previous_track(None).await?;
        Ok(())
    }
}

#[async_trait]
impl FeedFetcher for SpotifyFetcher {
    async fn fetch(&self) -> Result<FeedData> {
        // Refresh token if needed
        if let Err(e) = self.client.refetch_token().await {
            return Ok(FeedData::Error(format!("Failed to refresh token: {}", e)));
        }

        match self.client.current_playback(None, None::<Vec<_>>).await {
            Ok(Some(playback)) => {
                let (track_name, artist_name, album_name, duration_ms) = match playback.item {
                    Some(PlayableItem::Track(track)) => {
                        let artists = track
                            .artists
                            .iter()
                            .map(|a| a.name.clone())
                            .collect::<Vec<_>>()
                            .join(", ");
                        (
                            Some(track.name),
                            Some(artists),
                            Some(track.album.name),
                            Some(track.duration.as_millis() as u32),
                        )
                    }
                    Some(PlayableItem::Episode(episode)) => (
                        Some(episode.name),
                        Some(episode.show.publisher),
                        Some(episode.show.name),
                        Some(episode.duration.as_millis() as u32),
                    ),
                    None => (None, None, None, None),
                };

                let repeat_state = match playback.repeat_state {
                    RepeatState::Off => "off".to_string(),
                    RepeatState::Track => "track".to_string(),
                    RepeatState::Context => "context".to_string(),
                };

                Ok(FeedData::Spotify(SpotifyPlayback {
                    is_playing: playback.is_playing,
                    track_name,
                    artist_name,
                    album_name,
                    progress_ms: playback.progress.map(|d| d.as_millis() as u32),
                    duration_ms,
                    shuffle_state: playback.shuffle_state,
                    repeat_state,
                }))
            }
            Ok(None) => Ok(FeedData::Spotify(SpotifyPlayback {
                is_playing: false,
                track_name: Some("No active playback".to_string()),
                artist_name: None,
                album_name: None,
                progress_ms: None,
                duration_ms: None,
                shuffle_state: false,
                repeat_state: "off".to_string(),
            })),
            Err(e) => Ok(FeedData::Error(format!("Spotify API error: {}", e))),
        }
    }
}
