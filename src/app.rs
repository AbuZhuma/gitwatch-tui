use crate::github::models::PullRequest;

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub viewer: String,
    pub pull_requests: Vec<PullRequest>,
}

impl App {
    pub fn new(viewer: String, pull_requests: Vec<PullRequest>) -> Self {
        Self {
            running: true,
            viewer,
            pull_requests,
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}
