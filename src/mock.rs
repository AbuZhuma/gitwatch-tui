use chrono::{Duration, Utc};

use crate::config::Group;
use crate::github::models::{
    Activity, Check, CiStatus, MergeState, PullRequest, Review, ReviewDecision, ReviewState,
    Urgency,
};

pub fn groups() -> Vec<Group> {
    vec![
        Group {
            name: "Backend".to_owned(),
            repos: vec![
                "acme/api-gateway".to_owned(),
                "acme/billing".to_owned(),
                "acme/core".to_owned(),
                "acme/auth".to_owned(),
                "acme/notifications".to_owned(),
            ],
        },
        Group {
            name: "Frontend".to_owned(),
            repos: vec!["acme/web-client".to_owned()],
        },
    ]
}

pub fn repos() -> Vec<String> {
    groups().into_iter().flat_map(|group| group.repos).collect()
}

pub fn pull_requests() -> Vec<PullRequest> {
    let now = Utc::now();
    let hours = |h: i64| now - Duration::hours(h);
    let days = |d: i64| now - Duration::days(d);

    vec![
        PullRequest {
            repo: "acme/api-gateway".to_owned(),
            number: 142,
            title: "Fix auth token refresh".to_owned(),
            head_ref: "fix/auth-refresh".to_owned(),
            base_ref: "main".to_owned(),
            created_at: days(1),
            updated_at: hours(2),
            ci: CiStatus::Failing,
            review: ReviewDecision::ChangesRequested,
            mergeable: MergeState::Mergeable,
            is_draft: false,
            mention_at: None,
            urgency: Urgency::Now,
            checks: vec![
                Check {
                    name: "build".to_owned(),
                    state: CiStatus::Passing,
                },
                Check {
                    name: "test".to_owned(),
                    state: CiStatus::Failing,
                },
                Check {
                    name: "lint".to_owned(),
                    state: CiStatus::Passing,
                },
            ],
            reviews: vec![Review {
                login: "bob".to_owned(),
                state: ReviewState::ChangesRequested,
            }],
            activity: vec![Activity {
                author: "bob".to_owned(),
                at: hours(2),
                summary: "the token refresh test is failing on CI".to_owned(),
            }],
        },
        PullRequest {
            repo: "acme/web-client".to_owned(),
            number: 88,
            title: "Add dark mode".to_owned(),
            head_ref: "feat/dark-mode".to_owned(),
            base_ref: "main".to_owned(),
            created_at: days(3),
            updated_at: hours(5),
            ci: CiStatus::Passing,
            review: ReviewDecision::Approved,
            mergeable: MergeState::Mergeable,
            is_draft: false,
            mention_at: None,
            urgency: Urgency::Now,
            checks: vec![
                Check {
                    name: "build".to_owned(),
                    state: CiStatus::Passing,
                },
                Check {
                    name: "test".to_owned(),
                    state: CiStatus::Passing,
                },
            ],
            reviews: vec![Review {
                login: "alice".to_owned(),
                state: ReviewState::Approved,
            }],
            activity: vec![Activity {
                author: "alice".to_owned(),
                at: hours(5),
                summary: "looks great, approving".to_owned(),
            }],
        },
        PullRequest {
            repo: "acme/billing".to_owned(),
            number: 57,
            title: "Refactor invoice generation".to_owned(),
            head_ref: "refactor/invoices".to_owned(),
            base_ref: "main".to_owned(),
            created_at: days(2),
            updated_at: hours(1),
            ci: CiStatus::Passing,
            review: ReviewDecision::ReviewRequired,
            mergeable: MergeState::Mergeable,
            is_draft: false,
            mention_at: Some(hours(1)),
            urgency: Urgency::Now,
            checks: vec![Check {
                name: "build".to_owned(),
                state: CiStatus::Passing,
            }],
            reviews: vec![],
            activity: vec![Activity {
                author: "carol".to_owned(),
                at: hours(1),
                summary: "@octocat can you take a look at the rounding here?".to_owned(),
            }],
        },
        PullRequest {
            repo: "acme/core".to_owned(),
            number: 31,
            title: "Update dependencies".to_owned(),
            head_ref: "chore/deps".to_owned(),
            base_ref: "main".to_owned(),
            created_at: days(4),
            updated_at: days(4),
            ci: CiStatus::Passing,
            review: ReviewDecision::ReviewRequired,
            mergeable: MergeState::Mergeable,
            is_draft: false,
            mention_at: None,
            urgency: Urgency::Soon,
            checks: vec![Check {
                name: "build".to_owned(),
                state: CiStatus::Passing,
            }],
            reviews: vec![],
            activity: vec![],
        },
        PullRequest {
            repo: "acme/web-client".to_owned(),
            number: 90,
            title: "Tweak responsive layout".to_owned(),
            head_ref: "fix/layout".to_owned(),
            base_ref: "main".to_owned(),
            created_at: days(1),
            updated_at: hours(8),
            ci: CiStatus::Passing,
            review: ReviewDecision::ReviewRequired,
            mergeable: MergeState::Conflicting,
            is_draft: false,
            mention_at: None,
            urgency: Urgency::Soon,
            checks: vec![Check {
                name: "build".to_owned(),
                state: CiStatus::Passing,
            }],
            reviews: vec![],
            activity: vec![],
        },
        PullRequest {
            repo: "acme/auth".to_owned(),
            number: 12,
            title: "Add OIDC provider".to_owned(),
            head_ref: "feat/oidc".to_owned(),
            base_ref: "main".to_owned(),
            created_at: days(2),
            updated_at: hours(20),
            ci: CiStatus::Pending,
            review: ReviewDecision::ChangesRequested,
            mergeable: MergeState::Mergeable,
            is_draft: false,
            mention_at: None,
            urgency: Urgency::Soon,
            checks: vec![
                Check {
                    name: "build".to_owned(),
                    state: CiStatus::Passing,
                },
                Check {
                    name: "integration".to_owned(),
                    state: CiStatus::Pending,
                },
            ],
            reviews: vec![Review {
                login: "dave".to_owned(),
                state: ReviewState::ChangesRequested,
            }],
            activity: vec![],
        },
        PullRequest {
            repo: "acme/api-gateway".to_owned(),
            number: 150,
            title: "Cleanup request logging".to_owned(),
            head_ref: "chore/logging".to_owned(),
            base_ref: "main".to_owned(),
            created_at: hours(10),
            updated_at: hours(6),
            ci: CiStatus::Passing,
            review: ReviewDecision::None,
            mergeable: MergeState::Mergeable,
            is_draft: false,
            mention_at: None,
            urgency: Urgency::Background,
            checks: vec![Check {
                name: "build".to_owned(),
                state: CiStatus::Passing,
            }],
            reviews: vec![],
            activity: vec![],
        },
        PullRequest {
            repo: "acme/core".to_owned(),
            number: 33,
            title: "Fix typo in docs".to_owned(),
            head_ref: "docs/typo".to_owned(),
            base_ref: "main".to_owned(),
            created_at: hours(12),
            updated_at: hours(9),
            ci: CiStatus::None,
            review: ReviewDecision::None,
            mergeable: MergeState::Mergeable,
            is_draft: true,
            mention_at: None,
            urgency: Urgency::Background,
            checks: vec![],
            reviews: vec![],
            activity: vec![],
        },
    ]
}
