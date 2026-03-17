use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::storage;

const GITHUB_REPO_OWNER: &str = "PlatformNetwork";
const GITHUB_REPO_NAME: &str = "bounty-challenge";
const MAX_PAGES: u32 = 500;
const ISSUES_PER_PAGE: usize = 100;
const SECONDS_24H: i64 = 86_400;

#[derive(Serialize, Deserialize)]
struct HttpGetRequest {
    pub url: String,
    pub headers: BTreeMap<String, String>,
}

#[derive(Deserialize)]
struct HttpResponse {
    pub status: u16,
    #[allow(dead_code)]
    pub headers: BTreeMap<String, String>,
    pub body: Vec<u8>,
}

#[derive(Deserialize)]
struct GitHubIssue {
    pub number: u32,
    pub user: Option<GitHubUser>,
    pub labels: Vec<GitHubLabel>,
    pub state: String,
    pub created_at: Option<String>,
}

#[derive(Deserialize)]
struct GitHubUser {
    pub login: String,
}

#[derive(Deserialize)]
struct GitHubLabel {
    pub name: String,
}

pub struct SyncStats {
    pub fetched: u32,
    pub awarded: u32,
    pub penalized: u32,
    pub last_error: Option<String>,
}

fn http_get(url: &str, github_token: Option<&str>) -> Option<Vec<u8>> {
    let mut headers = BTreeMap::new();
    headers.insert(
        String::from("Accept"),
        String::from("application/vnd.github.v3+json"),
    );
    headers.insert(
        String::from("User-Agent"),
        String::from("platform-validator"),
    );
    if let Some(token) = github_token {
        headers.insert(
            String::from("Authorization"),
            alloc::format!("Bearer {}", token),
        );
    }

    let req = HttpGetRequest {
        url: String::from(url),
        headers,
    };

    let req_bytes = bincode::serialize(&req).ok()?;
    let resp_bytes = platform_challenge_sdk_wasm::host_functions::host_http_get(&req_bytes).ok()?;
    let result: Result<HttpResponse, String> = bincode::deserialize(&resp_bytes).ok()?;
    let resp = result.ok()?;

    if resp.status == 200 {
        Some(resp.body)
    } else {
        None
    }
}

fn build_since_param() -> String {
    let now_ms = platform_challenge_sdk_wasm::host_functions::host_get_timestamp();
    let now = now_ms / 1000; // host_get_timestamp returns milliseconds
    let since_ts = now - SECONDS_24H;
    // Format as ISO 8601: YYYY-MM-DDTHH:MM:SSZ
    let secs_per_day: i64 = 86400;
    let secs_per_hour: i64 = 3600;
    let secs_per_min: i64 = 60;

    // Days since epoch
    let mut remaining = since_ts;
    let days = remaining / secs_per_day;
    remaining %= secs_per_day;
    if remaining < 0 {
        remaining += secs_per_day;
    }
    let hour = remaining / secs_per_hour;
    remaining %= secs_per_hour;
    let min = remaining / secs_per_min;
    let sec = remaining % secs_per_min;

    // Convert days since epoch to Y-M-D
    let (year, month, day) = days_to_ymd(days);

    let mut s = String::new();
    use core::fmt::Write;
    let _ = write!(
        s,
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hour, min, sec
    );
    s
}

fn days_to_ymd(days_since_epoch: i64) -> (i64, i64, i64) {
    // Algorithm from http://howardhinnant.github.io/date_algorithms.html
    let z = days_since_epoch + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

/// Parse ISO 8601 timestamp (e.g. "2026-03-04T12:00:00Z") to milliseconds since epoch.
fn parse_iso8601_to_ms(s: &str) -> Option<i64> {
    // Expected format: YYYY-MM-DDTHH:MM:SSZ
    if s.len() < 20 {
        return None;
    }
    let year: i64 = s[0..4].parse().ok()?;
    let month: i64 = s[5..7].parse().ok()?;
    let day: i64 = s[8..10].parse().ok()?;
    let hour: i64 = s[11..13].parse().ok()?;
    let min: i64 = s[14..16].parse().ok()?;
    let sec: i64 = s[17..19].parse().ok()?;

    // Convert to days since epoch using inverse of days_to_ymd
    let (y, m) = if month <= 2 {
        (year - 1, month + 9)
    } else {
        (year, month - 3)
    };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * m + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe - 719468;

    let ts = days * 86400 + hour * 3600 + min * 60 + sec;
    Some(ts * 1000)
}

pub fn fetch_and_process_issues() -> SyncStats {
    let env_token = platform_challenge_sdk_wasm::host_functions::host_env_get("GITHUB_TOKEN")
        .and_then(|b| alloc::string::String::from_utf8(b).ok());
    fetch_and_process_issues_with_token(env_token.as_deref())
}

pub fn fetch_and_process_issues_with_token(github_token: Option<&str>) -> SyncStats {
    let mut stats = SyncStats {
        fetched: 0,
        awarded: 0,
        penalized: 0,
        last_error: None,
    };

    let _since = build_since_param();
    let mut all_issues: Vec<GitHubIssue> = Vec::new();

    let mut page = 1u32;
    loop {
        let mut url = String::from("https://api.github.com/repos/");
        use core::fmt::Write;
        // Fetch all issues sorted by creation date (newest first).
        // We filter by created_at in post-processing. Avoid using &since=
        // which filters on updated_at and causes accuracy issues.
        let _ = write!(
            url,
            "{}/{}/issues?state=all&sort=created&direction=desc&per_page={}&page={}",
            GITHUB_REPO_OWNER, GITHUB_REPO_NAME, ISSUES_PER_PAGE, page
        );

        let body = match http_get(&url, github_token) {
            Some(b) => b,
            None => break,
        };

        let issues: Vec<GitHubIssue> = match serde_json::from_slice(&body) {
            Ok(v) => v,
            Err(e) => {
                stats.last_error = Some(alloc::format!(
                    "JSON parse error: {} (body_len={}, first_200={:?})",
                    e,
                    body.len(),
                    core::str::from_utf8(&body[..body.len().min(200)]).unwrap_or("non-utf8")
                ));
                break;
            }
        };

        let count = issues.len();

        // Early exit: since results are sorted by created desc, if the
        // last issue on this page was created before the 24h cutoff we
        // have all recent issues and can stop paginating.
        let now_ms_check = platform_challenge_sdk_wasm::host_functions::host_get_timestamp();
        let cutoff_check = now_ms_check - (SECONDS_24H * 1000);
        let mut all_old = false;
        if let Some(last) = issues.last() {
            if let Some(ref created) = last.created_at {
                if let Some(created_ms) = parse_iso8601_to_ms(created) {
                    if created_ms < cutoff_check {
                        all_old = true;
                    }
                }
            }
        }

        all_issues.extend(issues);

        if all_old || count < ISSUES_PER_PAGE {
            break;
        }
        page += 1;
        if page > MAX_PAGES {
            break;
        }
    }

    stats.fetched = all_issues.len() as u32;

    // Ensure github:{username} -> hotkey index is populated
    storage::rebuild_github_index();

    // Build the complete issue list from the 24h fetch, then overwrite the
    // synced_issues blob in one shot so old issues never accumulate.
    let epoch = platform_challenge_sdk_wasm::host_functions::host_consensus_get_epoch() as u64;
    let now_ms = platform_challenge_sdk_wasm::host_functions::host_get_timestamp();
    let cutoff_ms = now_ms - (SECONDS_24H * 1000);
    let mut records: Vec<crate::types::IssueRecord> = Vec::new();

    for issue in &all_issues {
        let issue_created_ms = issue
            .created_at
            .as_ref()
            .and_then(|c| parse_iso8601_to_ms(c))
            .unwrap_or(0);

        // Filter out issues created more than 24h ago
        if issue_created_ms > 0 && issue_created_ms < cutoff_ms {
            continue;
        }

        let author = match &issue.user {
            Some(u) => u.login.to_lowercase(),
            None => continue,
        };

        let label_names: Vec<String> = issue.labels.iter().map(|l| l.name.to_lowercase()).collect();
        let has_ide = label_names.iter().any(|l| l == "ide");
        if !has_ide {
            continue;
        }
        let has_valid = label_names.iter().any(|l| l == "valid");
        let has_invalid = label_names.iter().any(|l| l == "invalid");
        let has_duplicate = label_names
            .iter()
            .any(|l| l == "duplicate" || l == "duplicated");
        let has_malicious = label_names.iter().any(|l| l == "malicious");
        let is_closed = issue.state == "closed";

        // Find registered hotkey for this GitHub username
        let hotkey = storage::get_hotkey_by_github(&author);

        if has_malicious || has_invalid || has_duplicate {
            stats.penalized += 1;
        } else if has_valid {
            stats.awarded += 1;
        }

        records.push(crate::types::IssueRecord {
            issue_number: issue.number,
            repo_owner: GITHUB_REPO_OWNER.into(),
            repo_name: GITHUB_REPO_NAME.into(),
            author: author.clone(),
            is_closed,
            has_valid_label: has_valid,
            has_invalid_label: has_invalid,
            has_ide_label: has_ide,
            claimed_by_hotkey: hotkey,
            recorded_epoch: epoch,
            has_duplicate_label: has_duplicate,
            has_malicious_label: has_malicious,
            created_at_ms: issue_created_ms,
        });
    }

    // Merge with existing issues: keep existing data that wasn't re-fetched
    // (e.g. due to rate limits) but update any that were re-fetched with
    // fresh label state. Purge issues older than 24h.
    let existing = storage::get_synced_issues();
    let mut merged: Vec<crate::types::IssueRecord> = Vec::new();

    // Index new records by (repo_owner, repo_name, issue_number) for fast lookup
    let mut new_index = alloc::collections::BTreeMap::new();
    for r in &records {
        new_index.insert(
            (r.repo_owner.clone(), r.repo_name.clone(), r.issue_number),
            r,
        );
    }

    // Keep existing issues that are still within 24h and were NOT re-fetched
    // (re-fetched ones will be replaced by fresh data with updated labels)
    for existing_issue in &existing {
        let key = (
            existing_issue.repo_owner.clone(),
            existing_issue.repo_name.clone(),
            existing_issue.issue_number,
        );
        if new_index.contains_key(&key) {
            continue; // will be replaced by fresh data
        }
        // Purge issues older than 24h (if we know their creation time)
        if existing_issue.created_at_ms > 0 && existing_issue.created_at_ms < cutoff_ms {
            continue;
        }
        merged.push(existing_issue.clone());
    }

    // Add all freshly fetched records (they have up-to-date labels)
    merged.extend(records);

    if !merged.is_empty() {
        storage::store_issue_data(&merged);
    }

    // Recount all balances from scratch
    storage::recount_all_balances();

    stats
}
