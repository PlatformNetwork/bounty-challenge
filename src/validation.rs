use alloc::string::String;
use alloc::vec::Vec;

use crate::storage;
use crate::types::{BountySubmission, ClaimResult, ClaimedIssue, IssueRecord, RejectedIssue};

pub fn validate_submission(submission: &BountySubmission) -> bool {
    if submission.hotkey.is_empty() {
        return false;
    }
    if submission.github_username.is_empty() {
        return false;
    }
    if submission.issue_numbers.is_empty() {
        return false;
    }
    if submission.signature.is_empty() {
        return false;
    }
    if submission.repo_owner.is_empty() || submission.repo_name.is_empty() {
        return false;
    }
    true
}

pub fn validate_issue(issue: &IssueRecord, expected_author: &str) -> (bool, Option<String>) {
    if !issue.is_closed {
        return (false, Some(String::from("Issue is not closed")));
    }
    if !issue.has_valid_label {
        return (false, Some(String::from("Issue missing 'valid' label")));
    }
    if issue.has_invalid_label {
        return (false, Some(String::from("Issue has 'invalid' label")));
    }
    if issue.author.to_lowercase() != expected_author.to_lowercase() {
        let mut msg = String::from("Author mismatch: expected ");
        msg.push_str(expected_author);
        msg.push_str(", got ");
        msg.push_str(&issue.author);
        return (false, Some(msg));
    }
    if issue.claimed_by_hotkey.is_some() {
        return (false, Some(String::from("Issue already claimed")));
    }
    (true, None)
}

pub fn process_claims(submission: &BountySubmission, synced_issues: &[IssueRecord]) -> ClaimResult {
    let mut claimed = Vec::new();
    let mut rejected = Vec::new();

    for &issue_number in &submission.issue_numbers {
        if storage::is_issue_recorded(&submission.repo_owner, &submission.repo_name, issue_number) {
            rejected.push(RejectedIssue {
                issue_number,
                reason: String::from("Issue already claimed"),
            });
            continue;
        }

        let issue = synced_issues.iter().find(|i| {
            i.issue_number == issue_number
                && i.repo_owner == submission.repo_owner
                && i.repo_name == submission.repo_name
        });

        match issue {
            Some(issue_record) => {
                let (valid, reason) = validate_issue(issue_record, &submission.github_username);

                if valid {
                    let recorded = storage::record_valid_issue(
                        issue_number,
                        &submission.repo_owner,
                        &submission.repo_name,
                        &submission.github_username,
                        &submission.hotkey,
                    );

                    if recorded {
                        claimed.push(ClaimedIssue { issue_number });
                    } else {
                        rejected.push(RejectedIssue {
                            issue_number,
                            reason: String::from("Failed to record issue"),
                        });
                    }
                } else {
                    rejected.push(RejectedIssue {
                        issue_number,
                        reason: reason.unwrap_or_else(|| String::from("Validation failed")),
                    });
                }
            }
            None => {
                rejected.push(RejectedIssue {
                    issue_number,
                    reason: String::from("Issue not found in synced data"),
                });
            }
        }
    }

    let balance = storage::get_user_balance(&submission.hotkey);
    let score =
        crate::scoring::calculate_weight_from_points(balance.valid_count, balance.star_count);

    ClaimResult {
        claimed,
        rejected,
        total_valid: balance.valid_count,
        score,
    }
}
