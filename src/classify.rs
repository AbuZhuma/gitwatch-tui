use chrono::{DateTime, Utc};

use crate::github::models::{CiStatus, MergeState, PullRequest, ReviewDecision, Urgency};

const STALE_REVIEW_DAYS: i64 = 2;

pub fn classify(pr: &PullRequest, now: DateTime<Utc>) -> Urgency {
    if pr.ci == CiStatus::Failing {
        return Urgency::Now;
    }

    let ready_to_merge = pr.review == ReviewDecision::Approved
        && pr.mergeable != MergeState::Conflicting
        && !pr.is_draft;
    if ready_to_merge {
        return Urgency::Now;
    }

    if pr.mergeable == MergeState::Conflicting {
        return Urgency::Soon;
    }

    if pr.review == ReviewDecision::ChangesRequested {
        return Urgency::Soon;
    }

    let unreviewed_for_days = (now - pr.created_at).num_days();
    if pr.review == ReviewDecision::ReviewRequired && unreviewed_for_days >= STALE_REVIEW_DAYS {
        return Urgency::Soon;
    }

    Urgency::Background
}
