use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};

use crate::github::models::{CiStatus, MergeState, PullRequest, ReviewDecision, Urgency};

pub type PrKey = (String, u64);

type Snapshot = (DateTime<Utc>, CiStatus, ReviewDecision, MergeState);

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub viewer: String,
    pub pull_requests: Vec<PullRequest>,
    pub selected: usize,
    pub refreshing: bool,
    pub last_updated: Option<DateTime<Utc>>,
    pub error: Option<String>,
    pub highlighted: HashSet<PrKey>,
    previous: HashMap<PrKey, Snapshot>,
    mentions_seen: HashMap<PrKey, DateTime<Utc>>,
    loaded_once: bool,
}

impl App {
    pub fn new(viewer: String) -> Self {
        Self {
            running: true,
            viewer,
            pull_requests: Vec::new(),
            selected: 0,
            refreshing: false,
            last_updated: None,
            error: None,
            highlighted: HashSet::new(),
            previous: HashMap::new(),
            mentions_seen: HashMap::new(),
            loaded_once: false,
        }
    }

    pub fn set_pull_requests(&mut self, mut pull_requests: Vec<PullRequest>) {
        let first_load = !self.loaded_once;
        let selected_key = self
            .pull_requests
            .get(self.selected)
            .map(|pr| (pr.repo.clone(), pr.number));

        let mut highlighted = HashSet::new();
        let mut next_snapshots = HashMap::with_capacity(pull_requests.len());
        let mut next_mentions = HashMap::new();

        for pr in &mut pull_requests {
            let key = (pr.repo.clone(), pr.number);

            if let Some(at) = pr.mention_at {
                let is_new = self.mentions_seen.get(&key).is_none_or(|seen| at > *seen);
                if is_new && !first_load {
                    pr.urgency = Urgency::Now;
                    highlighted.insert(key.clone());
                }
                next_mentions.insert(key.clone(), at);
            }

            let snapshot = snapshot_of(pr);
            let changed = self.previous.get(&key).is_none_or(|prev| *prev != snapshot);
            if changed && !first_load {
                highlighted.insert(key.clone());
            }
            next_snapshots.insert(key, snapshot);
        }

        pull_requests.sort_by(|a, b| {
            urgency_rank(a.urgency)
                .cmp(&urgency_rank(b.urgency))
                .then(b.updated_at.cmp(&a.updated_at))
        });

        self.selected = selected_key
            .and_then(|key| {
                pull_requests
                    .iter()
                    .position(|pr| pr.repo == key.0 && pr.number == key.1)
            })
            .unwrap_or(0)
            .min(pull_requests.len().saturating_sub(1));

        self.previous = next_snapshots;
        self.mentions_seen = next_mentions;
        self.highlighted = highlighted;
        self.pull_requests = pull_requests;
        self.error = None;
        self.last_updated = Some(Utc::now());
        self.loaded_once = true;
    }

    pub fn select_next(&mut self) {
        if !self.pull_requests.is_empty() {
            self.selected = (self.selected + 1).min(self.pull_requests.len() - 1);
        }
    }

    pub fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn set_error(&mut self, message: String) {
        self.error = Some(message);
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}

fn urgency_rank(urgency: Urgency) -> u8 {
    match urgency {
        Urgency::Now => 0,
        Urgency::Soon => 1,
        Urgency::Background => 2,
    }
}

fn snapshot_of(pr: &PullRequest) -> Snapshot {
    (pr.updated_at, pr.ci, pr.review, pr.mergeable)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn pr(repo: &str, number: u64, ci: CiStatus, updated: i64) -> PullRequest {
        pr_full(repo, number, ci, updated, None)
    }

    fn pr_full(
        repo: &str,
        number: u64,
        ci: CiStatus,
        updated: i64,
        mention: Option<i64>,
    ) -> PullRequest {
        PullRequest {
            repo: repo.to_owned(),
            number,
            title: "title".to_owned(),
            head_ref: "feature".to_owned(),
            base_ref: "main".to_owned(),
            created_at: Utc.timestamp_opt(0, 0).unwrap(),
            updated_at: Utc.timestamp_opt(updated, 0).unwrap(),
            ci,
            review: ReviewDecision::None,
            mergeable: MergeState::Unknown,
            is_draft: false,
            mention_at: mention.map(|s| Utc.timestamp_opt(s, 0).unwrap()),
            urgency: Urgency::Background,
            checks: Vec::new(),
            reviews: Vec::new(),
            activity: Vec::new(),
        }
    }

    #[test]
    fn first_load_highlights_nothing() {
        let mut app = App::new("me".to_owned());
        app.set_pull_requests(vec![pr("a/b", 1, CiStatus::None, 10)]);
        assert!(app.highlighted.is_empty());
    }

    #[test]
    fn changed_pr_is_highlighted() {
        let mut app = App::new("me".to_owned());
        app.set_pull_requests(vec![pr("a/b", 1, CiStatus::None, 10)]);
        app.set_pull_requests(vec![pr("a/b", 1, CiStatus::Failing, 20)]);
        assert!(app.highlighted.contains(&("a/b".to_owned(), 1)));
    }

    #[test]
    fn unchanged_pr_is_not_highlighted() {
        let mut app = App::new("me".to_owned());
        app.set_pull_requests(vec![pr("a/b", 1, CiStatus::None, 10)]);
        app.set_pull_requests(vec![pr("a/b", 1, CiStatus::None, 10)]);
        assert!(app.highlighted.is_empty());
    }

    #[test]
    fn new_pr_after_first_load_is_highlighted() {
        let mut app = App::new("me".to_owned());
        app.set_pull_requests(vec![pr("a/b", 1, CiStatus::None, 10)]);
        app.set_pull_requests(vec![
            pr("a/b", 1, CiStatus::None, 10),
            pr("c/d", 2, CiStatus::None, 5),
        ]);
        assert!(app.highlighted.contains(&("c/d".to_owned(), 2)));
        assert_eq!(app.highlighted.len(), 1);
    }

    #[test]
    fn new_mention_forces_now_and_highlight() {
        let mut app = App::new("me".to_owned());
        app.set_pull_requests(vec![pr_full("a/b", 1, CiStatus::None, 10, None)]);
        app.set_pull_requests(vec![pr_full("a/b", 1, CiStatus::None, 60, Some(50))]);
        assert_eq!(app.pull_requests[0].urgency, Urgency::Now);
        assert!(app.highlighted.contains(&("a/b".to_owned(), 1)));
    }

    #[test]
    fn mention_on_first_load_does_not_fire() {
        let mut app = App::new("me".to_owned());
        app.set_pull_requests(vec![pr_full("a/b", 1, CiStatus::None, 60, Some(50))]);
        assert_eq!(app.pull_requests[0].urgency, Urgency::Background);
        assert!(app.highlighted.is_empty());
    }

    #[test]
    fn same_mention_does_not_refire() {
        let mut app = App::new("me".to_owned());
        app.set_pull_requests(vec![pr_full("a/b", 1, CiStatus::None, 60, Some(50))]);
        app.set_pull_requests(vec![pr_full("a/b", 1, CiStatus::None, 60, Some(50))]);
        assert_eq!(app.pull_requests[0].urgency, Urgency::Background);
        assert!(app.highlighted.is_empty());
    }

    #[test]
    fn now_sorts_before_background() {
        let mut app = App::new("me".to_owned());
        let mut background = pr("bg/repo", 1, CiStatus::Passing, 100);
        background.urgency = Urgency::Background;
        let mut now = pr("now/repo", 2, CiStatus::Failing, 10);
        now.urgency = Urgency::Now;

        app.set_pull_requests(vec![background, now]);
        assert_eq!(app.pull_requests[0].urgency, Urgency::Now);
        assert_eq!(app.pull_requests[0].repo, "now/repo");
    }

    #[test]
    fn selection_follows_pr_across_refresh() {
        let mut app = App::new("me".to_owned());
        app.set_pull_requests(vec![
            pr("a/b", 1, CiStatus::Failing, 100),
            pr("c/d", 2, CiStatus::Passing, 50),
        ]);
        app.select_next();
        let selected = app.pull_requests[app.selected].repo.clone();
        app.set_pull_requests(vec![
            pr("c/d", 2, CiStatus::Failing, 200),
            pr("a/b", 1, CiStatus::Failing, 100),
        ]);
        assert_eq!(app.pull_requests[app.selected].repo, selected);
    }
}
