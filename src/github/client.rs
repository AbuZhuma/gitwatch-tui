use anyhow::{bail, Context, Result};
use reqwest::header::{AUTHORIZATION, USER_AGENT};
use serde::de::DeserializeOwned;
use serde::Deserialize;

const GRAPHQL_URL: &str = "https://api.github.com/graphql";
const CLIENT_USER_AGENT: &str = concat!("gitwatch-tui/", env!("CARGO_PKG_VERSION"));

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
