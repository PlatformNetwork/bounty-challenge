use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::storage;

const GITHUB_REPO_OWNER: &str = "PlatformNetwork";
const GITHUB_REPO_NAME: &str = "bounty-challenge";
const MIN_ISSUES_TO_FETCH: usize = 400;
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
}

fn http_get(url: &str) -> Option<Vec<u8>> {
    let mut headers = BTreeMap::new();
    headers.insert(
        String::from("Accept"),
        String::from("application/vnd.github.v3+json"),
    );
    headers.insert(
        String::from("User-Agent"),
        String::from("platform-validator"),
    );

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

pub fn fetch_and_process_issues() -> SyncStats {
    let mut stats = SyncStats {
        fetched: 0,
        awarded: 0,
        penalized: 0,
    };

    let since = build_since_param();
    let mut all_issues: Vec<GitHubIssue> = Vec::new();

    // Fetch pages until we have at least MIN_ISSUES_TO_FETCH or no more results
    let mut page = 1u32;
    loop {
        let mut url = String::from("https://api.github.com/repos/");
        use core::fmt::Write;
        let _ = write!(
            url,
            "{}/{}/issues?state=all&sort=updated&direction=desc&per_page={}&page={}&since={}",
            GITHUB_REPO_OWNER, GITHUB_REPO_NAME, ISSUES_PER_PAGE, page, since
        );

        let body = match http_get(&url) {
            Some(b) => b,
            None => break,
        };

        let issues: Vec<GitHubIssue> = match serde_json::from_slice(&body) {
            Ok(v) => v,
            Err(_) => break,
        };

        let count = issues.len();
        all_issues.extend(issues);

        // Stop if last page (fewer results than page size)
        if count < ISSUES_PER_PAGE {
            break;
        }
        // Stop once we have enough
        if all_issues.len() >= MIN_ISSUES_TO_FETCH {
            break;
        }
        page += 1;
    }

    stats.fetched = all_issues.len() as u32;

    for issue in &all_issues {
        let author = match &issue.user {
            Some(u) => u.login.to_lowercase(),
            None => continue,
        };

        let label_names: Vec<String> = issue.labels.iter().map(|l| l.name.to_lowercase()).collect();
        let has_valid = label_names.iter().any(|l| l == "valid");
        let has_invalid = label_names.iter().any(|l| l == "invalid");
        let has_ide = label_names.iter().any(|l| l == "ide");
        let is_closed = issue.state == "closed";

        // Only process issues that have relevant labels
        if !has_valid && !has_invalid {
            continue;
        }

        // Find registered hotkey for this GitHub username
        let hotkey = match storage::get_hotkey_by_github(&author) {
            Some(h) => h,
            None => continue,
        };

        // Skip if already processed
        if storage::is_issue_recorded(GITHUB_REPO_OWNER, GITHUB_REPO_NAME, issue.number) {
            continue;
        }

        if has_valid && has_ide {
            // Award points
            if storage::record_valid_issue(
                issue.number,
                GITHUB_REPO_OWNER,
                GITHUB_REPO_NAME,
                &author,
                &hotkey,
            ) {
                stats.awarded += 1;
            }
        } else if has_invalid {
            // Penalize
            let reason = if !is_closed {
                "Issue marked invalid (not closed)"
            } else {
                "Issue marked invalid"
            };
            if storage::record_invalid_issue(
                issue.number,
                GITHUB_REPO_OWNER,
                GITHUB_REPO_NAME,
                &author,
                Some(reason),
            ) {
                stats.penalized += 1;
            }
        }
    }

    stats
}
