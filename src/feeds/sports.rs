use super::{FeedData, FeedFetcher, SportsEvent};
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;

pub struct SportsFetcher {
    leagues: Vec<String>,
    client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct EspnResponse {
    events: Option<Vec<EspnEvent>>,
}

#[derive(Debug, Deserialize)]
struct EspnEvent {
    name: String,
    status: EspnStatus,
    competitions: Vec<EspnCompetition>,
}

#[derive(Debug, Deserialize)]
struct EspnStatus {
    #[serde(rename = "type")]
    status_type: EspnStatusType,
}

#[derive(Debug, Deserialize)]
struct EspnStatusType {
    description: String,
}

#[derive(Debug, Deserialize)]
struct EspnCompetition {
    competitors: Vec<EspnCompetitor>,
    #[serde(rename = "startDate")]
    start_date: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EspnCompetitor {
    #[serde(rename = "homeAway")]
    home_away: String,
    team: EspnTeam,
    score: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EspnTeam {
    #[serde(rename = "displayName")]
    display_name: String,
}

impl SportsFetcher {
    pub fn new(leagues: Vec<String>) -> Self {
        Self {
            leagues,
            client: reqwest::Client::new(),
        }
    }

    fn league_to_espn_endpoint(league: &str) -> Option<&'static str> {
        match league.to_lowercase().as_str() {
            "nba" => Some("basketball/nba"),
            "nfl" => Some("football/nfl"),
            "mlb" => Some("baseball/mlb"),
            "nhl" => Some("hockey/nhl"),
            "mls" => Some("soccer/usa.1"),
            "epl" | "premier-league" => Some("soccer/eng.1"),
            "ncaaf" | "college-football" => Some("football/college-football"),
            "ncaab" | "college-basketball" => Some("basketball/mens-college-basketball"),
            _ => None,
        }
    }

    async fn fetch_league(&self, league: &str) -> Result<Vec<SportsEvent>> {
        let endpoint = Self::league_to_espn_endpoint(league)
            .ok_or_else(|| anyhow::anyhow!("Unknown league: {}", league))?;

        let url = format!(
            "https://site.api.espn.com/apis/site/v2/sports/{}/scoreboard",
            endpoint
        );

        let response = self.client.get(&url).send().await?;
        let data: EspnResponse = response.json().await?;

        let events = data.events.unwrap_or_default();

        let sports_events: Vec<SportsEvent> = events
            .into_iter()
            .filter_map(|event| {
                let competition = event.competitions.first()?;

                let home = competition
                    .competitors
                    .iter()
                    .find(|c| c.home_away == "home")?;
                let away = competition
                    .competitors
                    .iter()
                    .find(|c| c.home_away == "away")?;

                Some(SportsEvent {
                    league: league.to_uppercase(),
                    home_team: home.team.display_name.clone(),
                    away_team: away.team.display_name.clone(),
                    home_score: away.score.as_ref().and_then(|s| s.parse().ok()),
                    away_score: home.score.as_ref().and_then(|s| s.parse().ok()),
                    status: event.status.status_type.description.clone(),
                    start_time: competition.start_date.clone(),
                })
            })
            .collect();

        Ok(sports_events)
    }
}

#[async_trait]
impl FeedFetcher for SportsFetcher {
    async fn fetch(&self) -> Result<FeedData> {
        let mut all_events = Vec::new();

        for league in &self.leagues {
            match self.fetch_league(league).await {
                Ok(events) => all_events.extend(events),
                Err(_) => continue,
            }
        }

        Ok(FeedData::Sports(all_events))
    }
}
