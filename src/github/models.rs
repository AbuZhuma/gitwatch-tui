use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CiStatus {
    Passing,
    Failing,
    Pending,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewDecision {
    Approved,
    ChangesRequested,
    ReviewRequired,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewState {
    Approved,
    ChangesRequested,
    Commented,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeState {
    Mergeable,
    Conflicting,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Urgency {
    Now,
    Soon,
    Background,
}

#[derive(Debug, Clone)]
pub struct Check {
    pub name: String,
    pub state: CiStatus,
}

#[derive(Debug, Clone)]
pub struct Review {
    pub login: String,
    pub state: ReviewState,
}

#[derive(Debug, Clone)]
pub struct Activity {
    pub author: String,
    pub at: DateTime<Utc>,
    pub summary: String,
}

#[derive(Debug, Clone)]
pub struct PullRequest {
    pub repo: String,
    pub number: u64,
    pub title: String,
    pub head_ref: String,
    pub base_ref: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub ci: CiStatus,
    pub review: ReviewDecision,
    pub mergeable: MergeState,
    pub is_draft: bool,
    pub mention_at: Option<DateTime<Utc>>,
    pub urgency: Urgency,
    pub checks: Vec<Check>,
    pub reviews: Vec<Review>,
    pub activity: Vec<Activity>,
}
