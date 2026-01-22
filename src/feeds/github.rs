use super::{
    FeedData, FeedFetcher, GithubCommit, GithubDashboard, GithubNotification, GithubPullRequest,
};
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;

const GITHUB_API_BASE: &str = "https://api.github.com";

pub struct GithubFetcher {
    token: String,
    username: String,
    show_notifications: bool,
    show_pull_requests: bool,
    show_commits: bool,
    max_notifications: usize,
    max_pull_requests: usize,
    max_commits: usize,
    client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct GithubApiNotification {
    id: String,
    subject: Subject,
    repository: Repository,
    unread: bool,
    updated_at: String,
    reason: String,
}

#[derive(Debug, Deserialize)]
struct Subject {
    title: String,
    #[serde(rename = "type")]
    notification_type: String,
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Repository {
    full_name: String,
}

#[derive(Debug, Deserialize)]
struct GithubApiPullRequest {
    id: u64,
    number: u32,
    title: String,
    state: String,
    user: User,
    created_at: String,
    updated_at: String,
    draft: bool,
    mergeable: Option<bool>,
    comments: u32,
    review_comments: u32,
    additions: u32,
    deletions: u32,
}

#[derive(Debug, Deserialize)]
struct User {
    login: String,
}

#[derive(Debug, Deserialize)]
struct GithubApiCommit {
    sha: String,
    commit: CommitDetails,
    author: Option<User>,
    html_url: String,
}

#[derive(Debug, Deserialize)]
struct CommitDetails {
    message: String,
    author: CommitAuthor,
}

#[derive(Debug, Deserialize)]
struct CommitAuthor {
    name: String,
    date: String,
}

#[derive(Debug, Deserialize)]
struct GithubApiRepo {
    full_name: String,
}

#[derive(Debug, Deserialize)]
struct GithubApiEvent {
    #[serde(rename = "type")]
    event_type: String,
    repo: GithubApiRepo,
    payload: EventPayload,
    created_at: String,
}

#[derive(Debug, Deserialize)]
struct EventPayload {
    commits: Option<Vec<EventCommit>>,
}

#[derive(Debug, Deserialize)]
struct EventCommit {
    sha: String,
    message: String,
    author: EventCommitAuthor,
}

#[derive(Debug, Deserialize)]
struct EventCommitAuthor {
    name: String,
}

impl GithubFetcher {
    pub fn new(
        token: String,
        username: String,
        show_notifications: bool,
        show_pull_requests: bool,
        show_commits: bool,
        max_notifications: usize,
        max_pull_requests: usize,
        max_commits: usize,
    ) -> Self {
        Self {
            token,
            username,
            show_notifications,
            show_pull_requests,
            show_commits,
            max_notifications,
            max_pull_requests,
            max_commits,
            client: reqwest::Client::new(),
        }
    }

    async fn fetch_notifications(&self) -> Result<Vec<GithubNotification>> {
        let url = format!("{}/notifications", GITHUB_API_BASE);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("token {}", self.token))
            .header("User-Agent", "feedtui")
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "GitHub API error (notifications): {}",
                response.status()
            ));
        }

        let api_notifications: Vec<GithubApiNotification> = response.json().await?;

        let notifications: Vec<GithubNotification> = api_notifications
            .into_iter()
            .take(self.max_notifications)
            .map(|n| GithubNotification {
                id: n.id,
                title: n.subject.title,
                notification_type: n.subject.notification_type,
                repository: n.repository.full_name,
                url: n.subject.url.unwrap_or_else(|| "N/A".to_string()),
                unread: n.unread,
                updated_at: n.updated_at,
                reason: n.reason,
            })
            .collect();

        Ok(notifications)
    }

    async fn fetch_pull_requests(&self) -> Result<Vec<GithubPullRequest>> {
        let url = format!(
            "{}/search/issues?q=involves:{}+type:pr+state:open&sort=updated&per_page={}",
            GITHUB_API_BASE, self.username, self.max_pull_requests
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("token {}", self.token))
            .header("User-Agent", "feedtui")
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "GitHub API error (pull requests): {}",
                response.status()
            ));
        }

        #[derive(Debug, Deserialize)]
        struct SearchResponse {
            items: Vec<SearchItem>,
        }

        #[derive(Debug, Deserialize)]
        struct SearchItem {
            number: u32,
            title: String,
            state: String,
            user: User,
            created_at: String,
            updated_at: String,
            draft: Option<bool>,
            comments: u32,
            pull_request: PullRequestRef,
        }

        #[derive(Debug, Deserialize)]
        struct PullRequestRef {
            url: String,
        }

        let search_response: SearchResponse = response.json().await?;
        let mut pull_requests = Vec::new();

        for item in search_response.items.iter().take(self.max_pull_requests) {
            // Extract repository from PR URL
            let repo = item
                .pull_request
                .url
                .trim_start_matches("https://api.github.com/repos/")
                .split("/pulls/")
                .next()
                .unwrap_or("unknown/unknown")
                .to_string();

            pull_requests.push(GithubPullRequest {
                id: item.number as u64,
                number: item.number,
                title: item.title.clone(),
                repository: repo,
                state: item.state.clone(),
                author: item.user.login.clone(),
                created_at: item.created_at.clone(),
                updated_at: item.updated_at.clone(),
                draft: item.draft.unwrap_or(false),
                mergeable: None,
                comments: item.comments,
                review_comments: 0,
                additions: 0,
                deletions: 0,
            });
        }

        Ok(pull_requests)
    }

    async fn fetch_commits(&self) -> Result<Vec<GithubCommit>> {
        let url = format!("{}/users/{}/events", GITHUB_API_BASE, self.username);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("token {}", self.token))
            .header("User-Agent", "feedtui")
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "GitHub API error (commits): {}",
                response.status()
            ));
        }

        let events: Vec<GithubApiEvent> = response.json().await?;
        let mut commits = Vec::new();

        for event in events {
            if event.event_type == "PushEvent" {
                if let Some(event_commits) = event.payload.commits {
                    for commit in event_commits {
                        commits.push(GithubCommit {
                            sha: commit.sha[..7].to_string(),
                            message: commit
                                .message
                                .lines()
                                .next()
                                .unwrap_or(&commit.message)
                                .to_string(),
                            author: commit.author.name,
                            repository: event.repo.full_name.clone(),
                            branch: "main".to_string(), // GitHub events don't always include branch
                            timestamp: event.created_at.clone(),
                            additions: 0,
                            deletions: 0,
                            url: format!(
                                "https://github.com/{}/commit/{}",
                                event.repo.full_name, commit.sha
                            ),
                        });

                        if commits.len() >= self.max_commits {
                            return Ok(commits);
                        }
                    }
                }
            }
        }

        Ok(commits)
    }
}

#[async_trait]
impl FeedFetcher for GithubFetcher {
    async fn fetch(&self) -> Result<FeedData> {
        let mut dashboard = GithubDashboard::default();

        // Fetch notifications if enabled
        if self.show_notifications {
            dashboard.notifications = self.fetch_notifications().await.unwrap_or_else(|e| {
                eprintln!("Failed to fetch notifications: {}", e);
                Vec::new()
            });
        }

        // Fetch pull requests if enabled
        if self.show_pull_requests {
            dashboard.pull_requests = self.fetch_pull_requests().await.unwrap_or_else(|e| {
                eprintln!("Failed to fetch pull requests: {}", e);
                Vec::new()
            });
        }

        // Fetch commits if enabled
        if self.show_commits {
            dashboard.commits = self.fetch_commits().await.unwrap_or_else(|e| {
                eprintln!("Failed to fetch commits: {}", e);
                Vec::new()
            });
        }

        Ok(FeedData::Github(dashboard))
    }
}
