use alloc::string::String;
use alloc::vec::Vec;
use platform_challenge_sdk_wasm::host_functions::{host_storage_get, host_storage_set};

use crate::types::{IssueRecord, IssueValidityProposal};

const ISSUE_PROPOSALS_KEY: &[u8] = b"issue_validity_proposals";
const SYNC_PROPOSALS_KEY: &[u8] = b"sync_proposals";

pub fn propose_issue_validity(
    validator_id: &str,
    issue_number: u32,
    repo_owner: &str,
    repo_name: &str,
    is_valid: bool,
) -> bool {
    let mut proposals: Vec<IssueValidityProposal> = host_storage_get(ISSUE_PROPOSALS_KEY)
        .ok()
        .and_then(|d| {
            if d.is_empty() {
                None
            } else {
                bincode::deserialize(&d).ok()
            }
        })
        .unwrap_or_default();

    if let Some(pos) = proposals.iter().position(|p| {
        p.validator_id == validator_id
            && p.issue_number == issue_number
            && p.repo_owner == repo_owner
            && p.repo_name == repo_name
    }) {
        proposals[pos].is_valid = is_valid;
    } else {
        proposals.push(IssueValidityProposal {
            validator_id: String::from(validator_id),
            issue_number,
            repo_owner: String::from(repo_owner),
            repo_name: String::from(repo_name),
            is_valid,
        });
    }

    if let Ok(data) = bincode::serialize(&proposals) {
        return host_storage_set(ISSUE_PROPOSALS_KEY, &data).is_ok();
    }
    false
}

pub fn check_issue_consensus(issue_number: u32, repo_owner: &str, repo_name: &str) -> Option<bool> {
    let proposals: Vec<IssueValidityProposal> = host_storage_get(ISSUE_PROPOSALS_KEY)
        .ok()
        .and_then(|d| {
            if d.is_empty() {
                None
            } else {
                bincode::deserialize(&d).ok()
            }
        })
        .unwrap_or_default();

    let relevant: Vec<&IssueValidityProposal> = proposals
        .iter()
        .filter(|p| {
            p.issue_number == issue_number && p.repo_owner == repo_owner && p.repo_name == repo_name
        })
        .collect();

    if relevant.is_empty() {
        return None;
    }

    let total = relevant.len();
    let threshold = (total / 2) + 1;
    let valid_count = relevant.iter().filter(|p| p.is_valid).count();
    let invalid_count = total - valid_count;

    if valid_count >= threshold {
        Some(true)
    } else if invalid_count >= threshold {
        Some(false)
    } else {
        None
    }
}

pub fn propose_sync_data(validator_id: &str, issues: &[IssueRecord]) -> bool {
    let mut proposals: Vec<(String, Vec<IssueRecord>)> = host_storage_get(SYNC_PROPOSALS_KEY)
        .ok()
        .and_then(|d| {
            if d.is_empty() {
                None
            } else {
                bincode::deserialize(&d).ok()
            }
        })
        .unwrap_or_default();

    if let Some(pos) = proposals.iter().position(|(v, _)| v == validator_id) {
        proposals[pos].1 = issues.to_vec();
    } else {
        proposals.push((String::from(validator_id), issues.to_vec()));
    }

    if let Ok(data) = bincode::serialize(&proposals) {
        return host_storage_set(SYNC_PROPOSALS_KEY, &data).is_ok();
    }
    false
}

pub fn check_sync_consensus() -> Option<Vec<IssueRecord>> {
    let proposals: Vec<(String, Vec<IssueRecord>)> = host_storage_get(SYNC_PROPOSALS_KEY)
        .ok()
        .and_then(|d| {
            if d.is_empty() {
                None
            } else {
                bincode::deserialize(&d).ok()
            }
        })
        .unwrap_or_default();

    if proposals.is_empty() {
        return None;
    }

    let validator_count = proposals.len();
    let threshold = (validator_count / 2) + 1;

    let mut counts: Vec<(Vec<u32>, usize, usize)> = Vec::new();
    for (_, issues) in &proposals {
        let mut issue_nums: Vec<u32> = issues.iter().map(|i| i.issue_number).collect();
        issue_nums.sort_unstable();

        if let Some(entry) = counts.iter_mut().find(|(k, _, _)| *k == issue_nums) {
            entry.1 += 1;
        } else {
            let idx = proposals
                .iter()
                .position(|(_, pi)| {
                    let mut nums: Vec<u32> = pi.iter().map(|i| i.issue_number).collect();
                    nums.sort_unstable();
                    nums == issue_nums
                })
                .unwrap_or(0);
            counts.push((issue_nums, 1, idx));
        }
    }

    for (_, count, idx) in &counts {
        if *count >= threshold {
            return Some(proposals[*idx].1.clone());
        }
    }
    None
}

#[allow(dead_code)]
pub fn clear_proposals() {
    let _ = host_storage_set(ISSUE_PROPOSALS_KEY, &[]);
    let _ = host_storage_set(SYNC_PROPOSALS_KEY, &[]);
}
