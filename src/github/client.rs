use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use reqwest::header::{AUTHORIZATION, USER_AGENT};
use serde::de::DeserializeOwned;
use serde::Deserialize;

use crate::classify::classify;
use crate::github::models::{
    Activity, Check, CiStatus, MergeState, PullRequest, Review, ReviewDecision, ReviewState,
    Urgency,
};

const GRAPHQL_URL: &str = "https://api.github.com/graphql";
const CLIENT_USER_AGENT: &str = concat!("gitwatch-tui/", env!("CARGO_PKG_VERSION"));

const OPEN_PRS_QUERY: &str = r#"{
  search(query: "is:open is:pr author:@me archived:false", type: ISSUE, first: 100) {
    nodes {
      ... on PullRequest {
        number
        title
        headRefName
        baseRefName
        createdAt
        updatedAt
        isDraft
        reviewDecision
        mergeable
        repository { nameWithOwner }
        commits(last: 1) {
          nodes { commit { statusCheckRollup {
            state
            contexts(first: 20) {
              nodes {
                ... on CheckRun { name conclusion status }
                ... on StatusContext { context state }
              }
            }
          } } }
        }
        latestReviews(first: 10) {
          nodes { author { login } state }
        }
        comments(last: 10) {
          nodes { author { login } createdAt bodyText }
        }
      }
    }
  }
}"#;

const REPOS_QUERY: &str = r#"{
  viewer {
    repositories(first: 100, affiliations: [OWNER, COLLABORATOR, ORGANIZATION_MEMBER], orderBy: {field: PUSHED_AT, direction: DESC}) {
      nodes { nameWithOwner }
    }
  }
}"#;

pub struct Client {
    http: reqwest::Client,
    token: String,
}

#[derive(Deserialize)]
struct GraphqlResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GraphqlError>>,
}

#[derive(Deserialize)]
struct GraphqlError {
    message: String,
}

#[derive(Deserialize)]
struct ViewerData {
    viewer: Viewer,
}

#[derive(Deserialize)]
struct Viewer {
    login: String,
}

#[derive(Deserialize)]
struct ReposData {
    viewer: ReposViewer,
}

#[derive(Deserialize)]
struct ReposViewer {
    repositories: RepoConnection,
}

#[derive(Deserialize)]
struct RepoConnection {
    nodes: Vec<RepoNode>,
}

#[derive(Deserialize)]
struct SearchData {
    search: Search,
}

#[derive(Deserialize)]
struct Search {
    nodes: Vec<PrNode>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PrNode {
    number: u64,
    title: String,
    head_ref_name: String,
    base_ref_name: String,
    created_at: String,
    updated_at: String,
    is_draft: bool,
    review_decision: Option<String>,
    mergeable: String,
    repository: RepoNode,
    commits: Commits,
    latest_reviews: ReviewConnection,
    comments: CommentConnection,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RepoNode {
    name_with_owner: String,
}

#[derive(Deserialize)]
struct Commits {
    nodes: Vec<CommitNode>,
}

#[derive(Deserialize)]
struct CommitNode {
    commit: Commit,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Commit {
    status_check_rollup: Option<StatusRollup>,
}

#[derive(Deserialize)]
struct StatusRollup {
    state: String,
    contexts: Contexts,
}

#[derive(Deserialize)]
struct Contexts {
    nodes: Vec<ContextNode>,
}

#[derive(Deserialize)]
struct ContextNode {
    name: Option<String>,
    conclusion: Option<String>,
    status: Option<String>,
    context: Option<String>,
    state: Option<String>,
}

#[derive(Deserialize)]
struct ReviewConnection {
    nodes: Vec<ReviewNode>,
}

#[derive(Deserialize)]
struct ReviewNode {
    author: Option<Author>,
    state: String,
}

#[derive(Deserialize)]
struct CommentConnection {
    nodes: Vec<CommentNode>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CommentNode {
    author: Option<Author>,
    created_at: String,
    body_text: String,
}

#[derive(Deserialize)]
struct Author {
    login: String,
}

impl Client {
    pub fn new(token: String) -> Result<Self> {
        let http = reqwest::Client::builder()
            .build()
            .context("failed to build HTTP client")?;
        Ok(Self { http, token })
    }

    pub async fn viewer_login(&self) -> Result<String> {
        let data: ViewerData = self.graphql("{ viewer { login } }").await?;
        Ok(data.viewer.login)
    }

    pub async fn repositories(&self) -> Result<Vec<String>> {
        let data: ReposData = self.graphql(REPOS_QUERY).await?;
        Ok(data
            .viewer
            .repositories
            .nodes
            .into_iter()
            .map(|node| node.name_with_owner)
            .collect())
    }

    pub async fn open_pull_requests(&self, viewer: &str) -> Result<Vec<PullRequest>> {
        let data: SearchData = self.graphql(OPEN_PRS_QUERY).await?;
        let now = Utc::now();

        let mut pull_requests = Vec::with_capacity(data.search.nodes.len());
        for node in data.search.nodes {
            let created_at = parse_time(&node.created_at)?;
            let updated_at = parse_time(&node.updated_at)?;
            let rollup = node
                .commits
                .nodes
                .first()
                .and_then(|commit| commit.commit.status_check_rollup.as_ref());

            let mut pr = PullRequest {
                repo: node.repository.name_with_owner,
                number: node.number,
                title: node.title,
                head_ref: node.head_ref_name,
                base_ref: node.base_ref_name,
                created_at,
                updated_at,
                ci: ci_status(rollup),
                review: review_decision(node.review_decision.as_deref()),
                mergeable: mergeable(&node.mergeable),
                is_draft: node.is_draft,
                mention_at: latest_mention(&node.comments, viewer),
                urgency: Urgency::Background,
                checks: checks_of(rollup),
                reviews: reviews_of(&node.latest_reviews),
                activity: activity_of(&node.comments),
            };
            pr.urgency = classify(&pr, now);
            pull_requests.push(pr);
        }

        pull_requests.sort_by_key(|pr| std::cmp::Reverse(pr.updated_at));
        Ok(pull_requests)
    }

    async fn graphql<T: DeserializeOwned>(&self, query: &str) -> Result<T> {
        let response = self
            .http
            .post(GRAPHQL_URL)
            .header(AUTHORIZATION, format!("Bearer {}", self.token))
            .header(USER_AGENT, CLIENT_USER_AGENT)
            .json(&serde_json::json!({ "query": query }))
            .send()
            .await
            .context("request to the GitHub API failed")?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            bail!("GitHub API returned HTTP {status}: {}", body.trim());
        }

        let body: GraphqlResponse<T> = response
            .json()
            .await
            .context("failed to parse the GitHub API response")?;

        if let Some(errors) = body.errors {
            let joined = errors
                .into_iter()
                .map(|e| e.message)
                .collect::<Vec<_>>()
                .join("; ");
            bail!("GitHub API error: {joined}");
        }

        body.data.context("GitHub API returned no data")
    }
}

fn parse_time(value: &str) -> Result<DateTime<Utc>> {
    Ok(DateTime::parse_from_rfc3339(value)
        .with_context(|| format!("unexpected timestamp value: {value}"))?
        .with_timezone(&Utc))
}

fn ci_status(rollup: Option<&StatusRollup>) -> CiStatus {
    match rollup.map(|rollup| rollup.state.as_str()) {
        Some("SUCCESS") => CiStatus::Passing,
        Some("FAILURE" | "ERROR") => CiStatus::Failing,
        Some("PENDING" | "EXPECTED") => CiStatus::Pending,
        _ => CiStatus::None,
    }
}

fn checks_of(rollup: Option<&StatusRollup>) -> Vec<Check> {
    let Some(rollup) = rollup else {
        return Vec::new();
    };

    rollup
        .contexts
        .nodes
        .iter()
        .map(|node| {
            if let Some(name) = &node.name {
                Check {
                    name: name.clone(),
                    state: check_run_state(node.conclusion.as_deref(), node.status.as_deref()),
                }
            } else {
                Check {
                    name: node.context.clone().unwrap_or_default(),
                    state: status_context_state(node.state.as_deref()),
                }
            }
        })
        .collect()
}

fn check_run_state(conclusion: Option<&str>, status: Option<&str>) -> CiStatus {
    if status != Some("COMPLETED") && conclusion.is_none() {
        return CiStatus::Pending;
    }
    match conclusion {
        Some("SUCCESS") => CiStatus::Passing,
        Some("FAILURE" | "TIMED_OUT" | "CANCELLED" | "ERROR" | "STARTUP_FAILURE") => {
            CiStatus::Failing
        }
        Some("NEUTRAL" | "SKIPPED" | "STALE") => CiStatus::None,
        _ => CiStatus::Pending,
    }
}

fn status_context_state(state: Option<&str>) -> CiStatus {
    match state {
        Some("SUCCESS") => CiStatus::Passing,
        Some("FAILURE" | "ERROR") => CiStatus::Failing,
        Some("PENDING" | "EXPECTED") => CiStatus::Pending,
        _ => CiStatus::None,
    }
}

fn reviews_of(reviews: &ReviewConnection) -> Vec<Review> {
    reviews
        .nodes
        .iter()
        .filter_map(|node| {
            node.author.as_ref().map(|author| Review {
                login: author.login.clone(),
                state: review_state(&node.state),
            })
        })
        .collect()
}

fn review_state(state: &str) -> ReviewState {
    match state {
        "APPROVED" => ReviewState::Approved,
        "CHANGES_REQUESTED" => ReviewState::ChangesRequested,
        "COMMENTED" => ReviewState::Commented,
        _ => ReviewState::Other,
    }
}

fn activity_of(comments: &CommentConnection) -> Vec<Activity> {
    comments
        .nodes
        .iter()
        .filter_map(|comment| {
            let at = DateTime::parse_from_rfc3339(&comment.created_at)
                .ok()?
                .with_timezone(&Utc);
            let author = comment
                .author
                .as_ref()
                .map(|author| author.login.clone())
                .unwrap_or_else(|| "ghost".to_owned());
            Some(Activity {
                author,
                at,
                summary: summarize(&comment.body_text),
            })
        })
        .collect()
}

fn summarize(body: &str) -> String {
    body.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("")
        .to_owned()
}

fn review_decision(value: Option<&str>) -> ReviewDecision {
    match value {
        Some("APPROVED") => ReviewDecision::Approved,
        Some("CHANGES_REQUESTED") => ReviewDecision::ChangesRequested,
        Some("REVIEW_REQUIRED") => ReviewDecision::ReviewRequired,
        _ => ReviewDecision::None,
    }
}

fn mergeable(value: &str) -> MergeState {
    match value {
        "MERGEABLE" => MergeState::Mergeable,
        "CONFLICTING" => MergeState::Conflicting,
        _ => MergeState::Unknown,
    }
}

fn latest_mention(comments: &CommentConnection, viewer: &str) -> Option<DateTime<Utc>> {
    comments
        .nodes
        .iter()
        .filter(|comment| {
            comment
                .author
                .as_ref()
                .is_some_and(|author| !author.login.eq_ignore_ascii_case(viewer))
        })
        .filter(|comment| mentions_user(&comment.body_text, viewer))
        .filter_map(|comment| DateTime::parse_from_rfc3339(&comment.created_at).ok())
        .map(|time| time.with_timezone(&Utc))
        .max()
}

fn mentions_user(body: &str, viewer: &str) -> bool {
    let needle = format!("@{}", viewer.to_lowercase());
    let body = body.to_lowercase();

    let mut start = 0;
    while let Some(offset) = body[start..].find(&needle) {
        let after = start + offset + needle.len();
        let boundary = body[after..]
            .chars()
            .next()
            .is_none_or(|c| !c.is_alphanumeric() && c != '-' && c != '_' && c != '/');
        if boundary {
            return true;
        }
        start = after;
    }
    false
}
