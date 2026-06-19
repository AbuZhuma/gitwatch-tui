use std::collections::{BTreeSet, HashMap, HashSet};

use chrono::{DateTime, Utc};
use crossterm::event::KeyCode;

use crate::config::{self, Group};
use crate::github::models::{CiStatus, MergeState, PullRequest, ReviewDecision, Urgency};

pub type PrKey = (String, u64);

type Snapshot = (DateTime<Utc>, CiStatus, ReviewDecision, MergeState);

pub enum Effect {
    None,
    Refresh,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Filter {
    All,
    Group(String),
    Repo(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Groups,
    List,
}

#[derive(Debug)]
pub enum Mode {
    Browse,
    GroupDetail(String),
    NameInput(String),
    RepoSelect(RepoSelect),
}

#[derive(Debug)]
pub struct RepoSelect {
    pub name: String,
    pub candidates: Vec<String>,
    pub chosen: HashSet<String>,
    pub cursor: usize,
}

pub struct SidebarEntry {
    pub label: String,
    pub filter: Filter,
    pub pr_count: usize,
    pub new_count: usize,
    pub is_group: bool,
}

pub struct App {
    pub running: bool,
    pub viewer: String,
    pub pull_requests: Vec<PullRequest>,
    pub selected: usize,
    pub detail_open: bool,
    pub refreshing: bool,
    pub last_updated: Option<DateTime<Utc>>,
    pub error: Option<String>,
    pub highlighted: HashSet<PrKey>,
    pub groups: Vec<Group>,
    pub filter: Filter,
    pub focus: Focus,
    pub mode: Mode,
    pub groups_selected: usize,
    available_repos: Vec<String>,
    previous: HashMap<PrKey, Snapshot>,
    mentions_seen: HashMap<PrKey, DateTime<Utc>>,
    loaded_once: bool,
}

impl App {
    pub fn new(viewer: String, groups: Vec<Group>, available_repos: Vec<String>) -> Self {
        Self {
            running: true,
            viewer,
            pull_requests: Vec::new(),
            selected: 0,
            detail_open: false,
            refreshing: false,
            last_updated: None,
            error: None,
            highlighted: HashSet::new(),
            groups,
            filter: Filter::All,
            focus: Focus::List,
            mode: Mode::Browse,
            groups_selected: 0,
            available_repos,
            previous: HashMap::new(),
            mentions_seen: HashMap::new(),
            loaded_once: false,
        }
    }

    pub fn set_pull_requests(&mut self, mut pull_requests: Vec<PullRequest>) {
        let first_load = !self.loaded_once;
        let selected_key = self
            .visible()
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

        self.previous = next_snapshots;
        self.mentions_seen = next_mentions;
        self.highlighted = highlighted;
        self.pull_requests = pull_requests;
        self.error = None;
        self.last_updated = Some(Utc::now());
        self.loaded_once = true;

        let restored = {
            let visible = self.visible();
            selected_key
                .and_then(|key| {
                    visible
                        .iter()
                        .position(|pr| pr.repo == key.0 && pr.number == key.1)
                })
                .unwrap_or(0)
                .min(visible.len().saturating_sub(1))
        };
        self.selected = restored;
    }

    pub fn set_error(&mut self, message: String) {
        self.error = Some(message);
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn visible(&self) -> Vec<&PullRequest> {
        self.pull_requests
            .iter()
            .filter(|pr| self.filter_matches(pr))
            .collect()
    }

    fn filter_matches(&self, pr: &PullRequest) -> bool {
        match &self.filter {
            Filter::All => true,
            Filter::Repo(repo) => &pr.repo == repo,
            Filter::Group(name) => self
                .groups
                .iter()
                .find(|group| &group.name == name)
                .is_some_and(|group| group.repos.contains(&pr.repo)),
        }
    }

    pub fn is_new(&self, pr: &PullRequest) -> bool {
        self.highlighted.contains(&(pr.repo.clone(), pr.number))
    }

    pub fn sidebar_entries(&self) -> Vec<SidebarEntry> {
        let mut entries = vec![SidebarEntry {
            label: "All".to_owned(),
            filter: Filter::All,
            pr_count: self.pull_requests.len(),
            new_count: self.highlighted.len(),
            is_group: false,
        }];

        let mut group_entries: Vec<SidebarEntry> = self
            .groups
            .iter()
            .map(|group| {
                let prs: Vec<&PullRequest> = self
                    .pull_requests
                    .iter()
                    .filter(|pr| group.repos.contains(&pr.repo))
                    .collect();
                let new_count = prs.iter().filter(|pr| self.is_new(pr)).count();
                SidebarEntry {
                    label: group.name.clone(),
                    filter: Filter::Group(group.name.clone()),
                    pr_count: prs.len(),
                    new_count,
                    is_group: true,
                }
            })
            .collect();
        group_entries.sort_by(|a, b| b.pr_count.cmp(&a.pr_count).then(a.label.cmp(&b.label)));
        entries.append(&mut group_entries);

        let repos: BTreeSet<&String> = self.pull_requests.iter().map(|pr| &pr.repo).collect();
        let mut repo_entries: Vec<SidebarEntry> = repos
            .into_iter()
            .map(|repo| {
                let prs: Vec<&PullRequest> = self
                    .pull_requests
                    .iter()
                    .filter(|pr| &pr.repo == repo)
                    .collect();
                let new_count = prs.iter().filter(|pr| self.is_new(pr)).count();
                SidebarEntry {
                    label: repo.clone(),
                    filter: Filter::Repo(repo.clone()),
                    pr_count: prs.len(),
                    new_count,
                    is_group: false,
                }
            })
            .collect();
        repo_entries.sort_by(|a, b| b.pr_count.cmp(&a.pr_count).then(a.label.cmp(&b.label)));
        entries.append(&mut repo_entries);

        entries
    }

    pub fn group_repo_counts(&self, name: &str) -> Vec<(String, usize, usize)> {
        let Some(group) = self.groups.iter().find(|group| group.name == name) else {
            return Vec::new();
        };

        let mut counts: Vec<(String, usize, usize)> = group
            .repos
            .iter()
            .map(|repo| {
                let count = self
                    .pull_requests
                    .iter()
                    .filter(|pr| &pr.repo == repo)
                    .count();
                let new = self
                    .pull_requests
                    .iter()
                    .filter(|pr| &pr.repo == repo && self.is_new(pr))
                    .count();
                (repo.clone(), count, new)
            })
            .collect();
        counts.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        counts
    }

    pub fn add_group(&mut self, name: String, repos: Vec<String>) {
        self.groups.push(Group { name, repos });
    }

    pub fn handle_key(&mut self, code: KeyCode) -> Effect {
        match std::mem::replace(&mut self.mode, Mode::Browse) {
            Mode::Browse => self.handle_browse(code),
            Mode::GroupDetail(name) => {
                self.mode = Mode::GroupDetail(name);
                self.handle_group_detail(code);
                Effect::None
            }
            Mode::NameInput(buffer) => {
                self.handle_name_input(code, buffer);
                Effect::None
            }
            Mode::RepoSelect(state) => {
                self.handle_repo_select(code, state);
                Effect::None
            }
        }
    }

    fn handle_browse(&mut self, code: KeyCode) -> Effect {
        match code {
            KeyCode::Char('q') => {
                self.quit();
                Effect::None
            }
            KeyCode::Char('r') => Effect::Refresh,
            KeyCode::Char('n') => {
                self.mode = Mode::NameInput(String::new());
                Effect::None
            }
            KeyCode::Esc => {
                if self.detail_open {
                    self.detail_open = false;
                } else if self.focus == Focus::List {
                    self.focus = Focus::Groups;
                } else {
                    self.quit();
                }
                Effect::None
            }
            _ => {
                match self.focus {
                    Focus::Groups => self.handle_groups_key(code),
                    Focus::List => self.handle_list_key(code),
                }
                Effect::None
            }
        }
    }

    fn handle_groups_key(&mut self, code: KeyCode) {
        let entries = self.sidebar_entries();
        match code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.groups_selected = self.groups_selected.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let last = entries.len().saturating_sub(1);
                if self.groups_selected >= last {
                    self.focus = Focus::List;
                    self.selected = 0;
                } else {
                    self.groups_selected += 1;
                }
            }
            KeyCode::Enter => {
                if let Some(entry) = entries.get(self.groups_selected) {
                    self.filter = entry.filter.clone();
                    self.selected = 0;
                    self.detail_open = false;
                    self.focus = Focus::List;
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if let Some(entry) = entries.get(self.groups_selected) {
                    if let Filter::Group(name) = &entry.filter {
                        self.mode = Mode::GroupDetail(name.clone());
                    }
                }
            }
            KeyCode::Backspace => {
                if let Some(Filter::Group(name)) = entries
                    .get(self.groups_selected)
                    .map(|entry| entry.filter.clone())
                {
                    self.remove_group(&name);
                    if let Err(e) = config::save(&self.groups) {
                        self.error = Some(format!("could not save groups: {e:#}"));
                    }
                    self.groups_selected = self
                        .groups_selected
                        .min(self.sidebar_entries().len().saturating_sub(1));
                }
            }
            _ => {}
        }
    }

    pub fn remove_group(&mut self, name: &str) {
        if let Some(pos) = self.groups.iter().position(|group| group.name == name) {
            self.groups.remove(pos);
        }
        let still_present = self.groups.iter().any(|group| group.name == name);
        if matches!(&self.filter, Filter::Group(n) if n.as_str() == name) && !still_present {
            self.filter = Filter::All;
        }
    }

    fn handle_list_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected == 0 {
                    self.focus = Focus::Groups;
                    self.groups_selected = self.sidebar_entries().len().saturating_sub(1);
                } else {
                    self.select_prev();
                }
            }
            KeyCode::Down | KeyCode::Char('j') => self.select_next(),
            KeyCode::Right | KeyCode::Enter | KeyCode::Char('l') => {
                if !self.visible().is_empty() {
                    self.detail_open = true;
                }
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if self.detail_open {
                    self.detail_open = false;
                } else {
                    self.focus = Focus::Groups;
                }
            }
            _ => {}
        }
    }

    fn handle_group_detail(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') => self.quit(),
            KeyCode::Esc | KeyCode::Left | KeyCode::Char('h') => self.mode = Mode::Browse,
            _ => {}
        }
    }

    fn handle_name_input(&mut self, code: KeyCode, mut buffer: String) {
        match code {
            KeyCode::Esc => self.mode = Mode::Browse,
            KeyCode::Enter => {
                let name = buffer.trim().to_owned();
                if name.is_empty() {
                    self.mode = Mode::NameInput(buffer);
                } else {
                    self.mode = Mode::RepoSelect(RepoSelect {
                        name,
                        candidates: self.candidate_repos(),
                        chosen: HashSet::new(),
                        cursor: 0,
                    });
                }
            }
            KeyCode::Backspace => {
                buffer.pop();
                self.mode = Mode::NameInput(buffer);
            }
            KeyCode::Char(c) if !c.is_control() => {
                buffer.push(c);
                self.mode = Mode::NameInput(buffer);
            }
            _ => self.mode = Mode::NameInput(buffer),
        }
    }

    fn handle_repo_select(&mut self, code: KeyCode, mut state: RepoSelect) {
        match code {
            KeyCode::Esc => self.mode = Mode::Browse,
            KeyCode::Up | KeyCode::Char('k') => {
                state.cursor = state.cursor.saturating_sub(1);
                self.mode = Mode::RepoSelect(state);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                state.cursor = (state.cursor + 1).min(state.candidates.len().saturating_sub(1));
                self.mode = Mode::RepoSelect(state);
            }
            KeyCode::Char(' ') => {
                if let Some(repo) = state.candidates.get(state.cursor) {
                    if state.chosen.contains(repo) {
                        state.chosen.remove(repo);
                    } else {
                        state.chosen.insert(repo.clone());
                    }
                }
                self.mode = Mode::RepoSelect(state);
            }
            KeyCode::Enter => {
                let repos: Vec<String> = state
                    .candidates
                    .iter()
                    .filter(|repo| state.chosen.contains(*repo))
                    .cloned()
                    .collect();
                self.add_group(state.name, repos);
                if let Err(e) = config::save(&self.groups) {
                    self.error = Some(format!("could not save groups: {e:#}"));
                }
                self.mode = Mode::Browse;
                self.focus = Focus::Groups;
            }
            _ => self.mode = Mode::RepoSelect(state),
        }
    }

    fn candidate_repos(&self) -> Vec<String> {
        let mut set: BTreeSet<String> = self.available_repos.iter().cloned().collect();
        for pr in &self.pull_requests {
            set.insert(pr.repo.clone());
        }
        for group in &self.groups {
            for repo in &group.repos {
                set.insert(repo.clone());
            }
        }
        set.into_iter().collect()
    }

    fn select_next(&mut self) {
        let len = self.visible().len();
        if len > 0 {
            self.selected = (self.selected + 1).min(len - 1);
        }
    }

    fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
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

    fn app() -> App {
        App::new("me".to_owned(), Vec::new(), Vec::new())
    }

    #[test]
    fn first_load_highlights_nothing() {
        let mut app = app();
        app.set_pull_requests(vec![pr("a/b", 1, CiStatus::None, 10)]);
        assert!(app.highlighted.is_empty());
    }

    #[test]
    fn changed_pr_is_highlighted() {
        let mut app = app();
        app.set_pull_requests(vec![pr("a/b", 1, CiStatus::None, 10)]);
        app.set_pull_requests(vec![pr("a/b", 1, CiStatus::Failing, 20)]);
        assert!(app.highlighted.contains(&("a/b".to_owned(), 1)));
    }

    #[test]
    fn new_mention_forces_now() {
        let mut app = app();
        app.set_pull_requests(vec![pr_full("a/b", 1, CiStatus::None, 10, None)]);
        app.set_pull_requests(vec![pr_full("a/b", 1, CiStatus::None, 60, Some(50))]);
        assert_eq!(app.pull_requests[0].urgency, Urgency::Now);
    }

    #[test]
    fn now_sorts_before_background() {
        let mut app = app();
        let mut background = pr("bg/repo", 1, CiStatus::Passing, 100);
        background.urgency = Urgency::Background;
        let mut now = pr("now/repo", 2, CiStatus::Failing, 10);
        now.urgency = Urgency::Now;
        app.set_pull_requests(vec![background, now]);
        assert_eq!(app.pull_requests[0].repo, "now/repo");
    }

    #[test]
    fn filter_by_group_limits_visible() {
        let mut app = App::new(
            "me".to_owned(),
            vec![Group {
                name: "backend".to_owned(),
                repos: vec!["org/api".to_owned()],
            }],
            Vec::new(),
        );
        app.set_pull_requests(vec![
            pr("org/api", 1, CiStatus::None, 10),
            pr("org/web", 2, CiStatus::None, 20),
        ]);
        app.filter = Filter::Group("backend".to_owned());
        let visible = app.visible();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].repo, "org/api");
    }

    #[test]
    fn sidebar_lists_groups_with_prs_and_repos() {
        let mut app = App::new(
            "me".to_owned(),
            vec![
                Group {
                    name: "backend".to_owned(),
                    repos: vec!["org/api".to_owned()],
                },
                Group {
                    name: "empty".to_owned(),
                    repos: vec!["org/none".to_owned()],
                },
            ],
            Vec::new(),
        );
        app.set_pull_requests(vec![pr("org/api", 1, CiStatus::None, 10)]);

        let labels: Vec<String> = app.sidebar_entries().into_iter().map(|e| e.label).collect();
        assert!(labels.contains(&"All".to_owned()));
        assert!(labels.contains(&"backend".to_owned()));
        assert!(labels.contains(&"org/api".to_owned()));
        assert!(labels.contains(&"empty".to_owned()));
    }

    #[test]
    fn groups_sorted_by_pr_count_desc() {
        let mut app = App::new(
            "me".to_owned(),
            vec![
                Group {
                    name: "small".to_owned(),
                    repos: vec!["org/a".to_owned()],
                },
                Group {
                    name: "big".to_owned(),
                    repos: vec!["org/b".to_owned(), "org/c".to_owned()],
                },
            ],
            Vec::new(),
        );
        app.set_pull_requests(vec![
            pr("org/a", 1, CiStatus::None, 10),
            pr("org/b", 2, CiStatus::None, 20),
            pr("org/c", 3, CiStatus::None, 30),
        ]);

        let groups: Vec<String> = app
            .sidebar_entries()
            .into_iter()
            .filter(|e| e.is_group)
            .map(|e| e.label)
            .collect();
        assert_eq!(groups, vec!["big".to_owned(), "small".to_owned()]);
    }

    #[test]
    fn group_detail_shows_all_repos_even_without_prs() {
        let mut app = App::new(
            "me".to_owned(),
            vec![Group {
                name: "backend".to_owned(),
                repos: vec!["org/api".to_owned(), "org/idle".to_owned()],
            }],
            Vec::new(),
        );
        app.set_pull_requests(vec![pr("org/api", 1, CiStatus::None, 10)]);

        let counts = app.group_repo_counts("backend");
        assert_eq!(counts.len(), 2);
        assert_eq!(counts[0], ("org/api".to_owned(), 1, 0));
        assert_eq!(counts[1], ("org/idle".to_owned(), 0, 0));
    }

    #[test]
    fn add_group_appends() {
        let mut app = app();
        app.add_group("frontend".to_owned(), vec!["org/web".to_owned()]);
        assert_eq!(app.groups.len(), 1);
        assert_eq!(app.groups[0].name, "frontend");
    }

    #[test]
    fn remove_group_deletes_and_resets_filter() {
        let mut app = App::new(
            "me".to_owned(),
            vec![Group {
                name: "backend".to_owned(),
                repos: vec!["org/api".to_owned()],
            }],
            Vec::new(),
        );
        app.filter = Filter::Group("backend".to_owned());
        app.remove_group("backend");
        assert!(app.groups.is_empty());
        assert_eq!(app.filter, Filter::All);
    }

    #[test]
    fn down_at_last_group_entry_enters_list() {
        let mut app = app();
        app.set_pull_requests(vec![pr("a/b", 1, CiStatus::None, 10)]);
        app.focus = Focus::Groups;
        app.groups_selected = app.sidebar_entries().len() - 1;
        app.handle_key(KeyCode::Down);
        assert_eq!(app.focus, Focus::List);
    }

    #[test]
    fn up_at_first_pr_enters_groups() {
        let mut app = app();
        app.set_pull_requests(vec![pr("a/b", 1, CiStatus::None, 10)]);
        app.focus = Focus::List;
        app.selected = 0;
        app.handle_key(KeyCode::Up);
        assert_eq!(app.focus, Focus::Groups);
    }
}
