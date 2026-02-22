use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::debug;
use std::{collections::HashMap, thread, time::Duration};

use crate::appstate::{AppState, BASE_URL};

#[derive(Debug, Deserialize)]
pub struct Location {
    host: String,
    begin_at: DateTime<Utc>,
    end_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct HostTime {
    pub host: String,
    pub total_ms: i64,
    pub total: String,
}

#[derive(Debug, Serialize)]
pub struct LongestSession {
    pub host: String,
    pub total_ms: i64,
    pub total: String,
    pub begin_at: DateTime<Utc>,
    pub end_at: String,
}

pub async fn get_all_locations(state: &AppState, token: &str, user: &str) -> Result<Vec<Location>> {
    let client = &state.client;
    let db = &state.db;

    if db.can_request(user).await? {
        debug!("Getting all locations from api for {user}:");
        let latest_begin = db.latest_begin(user).await?;

        let mut page = 1;
        let page_size = 100;

        loop {
            let url = format!(
                "{}/v2/users/{}/locations?page[number]={}&page[size]={}",
                BASE_URL, user, page, page_size
            );

            let api_locations: Vec<Location> = client
                .get(&url)
                .bearer_auth(token)
                .send()
                .await?
                .json()
                .await?;

            if api_locations.is_empty() {
                break;
            }

            debug!("\tFetched page {}, got {} records", page, api_locations.len());

            let mut all_new = true;
            for loc in &api_locations {
                let is_new = match &latest_begin {
                    Some(latest) => loc.begin_at.to_rfc3339() > *latest,
                    None => true,
                };

                if is_new {
                    db.insert_location(
                        user,
                        &loc.host,
                        &loc.begin_at.to_rfc3339(),
                        loc.end_at.map(|e| e.to_rfc3339()),
                    ).await?;
                } else {
                    all_new = false;
                }
            }

            if !all_new || api_locations.len() < page_size {
                break;
            }

            page += 1;
            thread::sleep(Duration::from_millis(500));
        }
    }

    Ok(db.get_locations(user).await?
        .into_iter()
        .map(|(host, begin_at, end_at)| Location {
            host,
            begin_at: begin_at.parse().unwrap(),
            end_at: end_at.map(|e| e.parse().unwrap()),
        })
        .collect())
}

fn format_duration(ms: i64) -> String {
    let total_seconds = ms / 1000;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    format!("{}h {}m {}s", hours, minutes, seconds)
}

pub fn compute_time_per_host(locations: &[Location]) -> Vec<HostTime> {
    let mut time_per_host: HashMap<String, i64> = HashMap::new();
    let now = Utc::now();

    for loc in locations {
        let end = loc.end_at.unwrap_or(now);
        let duration = end.signed_duration_since(loc.begin_at).num_milliseconds();

        *time_per_host.entry(loc.host.clone()).or_insert(0) += duration;
    }

    let mut results: Vec<_> = time_per_host
        .into_iter()
        .map(|(host, ms)| HostTime {
            host,
            total_ms: ms,
            total: format_duration(ms),
        })
        .collect();

    results.sort_by(|a, b| b.total_ms.cmp(&a.total_ms));
    results
}

pub fn get_longest_session_per_host(locations: &[Location]) -> Vec<LongestSession> {
    let mut longest_per_host: HashMap<String, (i64, &Location)> = HashMap::new();
    let now = Utc::now();

    for loc in locations {
        let end = loc.end_at.unwrap_or(now);
        let duration = end.signed_duration_since(loc.begin_at).num_milliseconds();

        match longest_per_host.get(&loc.host) {
            Some((existing_duration, _)) if *existing_duration >= duration => {}
            _ => {
                longest_per_host.insert(loc.host.clone(), (duration, loc));
            }
        }
    }

    let mut results: Vec<LongestSession> = longest_per_host
        .into_iter()
        .map(|(host, (total_ms, loc))| {
            let end_str = match loc.end_at {
                Some(end) => end.to_rfc3339(),
                None => "still active".to_string(),
            };

            LongestSession {
                host,
                total_ms,
                total: format_duration(total_ms),
                begin_at: loc.begin_at,
                end_at: end_str,
            }
        })
        .collect();

    results.sort_by(|a, b| b.total_ms.cmp(&a.total_ms));
    results
}
