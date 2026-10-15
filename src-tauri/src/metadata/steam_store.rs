//! Keyless Steam store metadata.
//!
//! Endpoints:
//! - `https://store.steampowered.com/api/appdetails?appids=<id>` — genres,
//!   developers, publishers, feature-flag categories, release date,
//!   header/capsule images, and the Metacritic critic score.
//! - `https://store.steampowered.com/actions/ajaxgetstoretags?appid=<id>` —
//!   Steam community tags.

use std::collections::BTreeMap;

use futures::StreamExt;
use serde_json::Value;

use crate::error::AppResult;
use crate::steam::models::AppMeta;

/// How many `appdetails` requests may be in flight at once. Steam tolerates a
/// modest burst, but we stay conservative to avoid tripping rate limiting.
const MAX_CONCURRENT_REQUESTS: usize = 8;

/// Fetch normalized metadata for a set of app ids.
///
/// Steam's bulk form (`appids=1,2,3`) is **no longer accepted** — it returns
/// HTTP 400 — so each app id is requested individually with bounded
/// concurrency. Individual failures are skipped rather than failing the batch,
/// so callers still receive whatever the store answered.
pub async fn fetch_app_details(
    client: &reqwest::Client,
    app_ids: &[u32],
    cc: &str,
    lang: &str,
) -> AppResult<BTreeMap<u32, AppMeta>> {
    let mut out = BTreeMap::new();
    if app_ids.is_empty() {
        return Ok(out);
    }

    let results: Vec<Option<(u32, AppMeta)>> = futures::stream::iter(app_ids.iter().copied())
        .map(|app_id| fetch_one(client, app_id, cc, lang))
        .buffer_unordered(MAX_CONCURRENT_REQUESTS)
        .collect()
        .await;

    for (app_id, meta) in results.into_iter().flatten() {
        out.insert(app_id, meta);
    }
    Ok(out)
}

/// Fetch one app's store details.
///
/// Returns `None` when the store has no data for the id (not a store app,
/// delisted) or the request fails — callers treat those as "unavailable".
async fn fetch_one(
    client: &reqwest::Client,
    app_id: u32,
    cc: &str,
    lang: &str,
) -> Option<(u32, AppMeta)> {
    let url =
        format!("https://store.steampowered.com/api/appdetails?appids={app_id}&cc={cc}&l={lang}");
    let response = client.get(&url).send().await.ok()?;
    if !response.status().is_success() {
        return None;
    }
    let value: Value = response.json().await.ok()?;
    let entry = value.get(app_id.to_string())?;
    if !entry
        .get("success")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return None;
    }
    let data = entry.get("data")?;
    Some((app_id, parse_details(app_id, data)))
}

fn parse_details(app_id: u32, data: &Value) -> AppMeta {
    let release_year = data
        .get("release_date")
        .and_then(|r| r.get("date"))
        .and_then(Value::as_str)
        .and_then(|date| {
            date.rsplit(' ')
                .find_map(|token| token.trim_end_matches(',').parse::<i32>().ok())
        });

    AppMeta {
        app_id,
        name: string(data.get("name")),
        developers: string_array(data.get("developers")),
        publishers: string_array(data.get("publishers")),
        genres: object_names(data.get("genres")),
        categories: object_names(data.get("categories")),
        release_date: data
            .get("release_date")
            .and_then(|r| r.get("date"))
            .and_then(Value::as_str)
            .map(str::to_string),
        release_year,
        short_description: string(data.get("short_description")),
        header_image: string(data.get("header_image")),
        capsule_image: string(data.get("capsule_image")),
        // The store response carries a Metacritic critic score (0–100) when one
        // exists. There is no keyless user-review percentage in this endpoint,
        // so the score bucket uses whichever score is available.
        user_score: data
            .get("metacritic")
            .and_then(|m| m.get("score"))
            .and_then(Value::as_f64),
        screenshots: data
            .get("screenshots")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|s| {
                        s.get("path_full")
                            .or_else(|| s.get("path_thumbnail"))
                            .and_then(Value::as_str)
                            .map(str::to_string)
                    })
                    .collect()
            })
            .unwrap_or_default(),
        movies: data
            .get("movies")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|m| {
                        m.get("mp4")
                            .and_then(|mp4| mp4.get("max").or_else(|| mp4.get("480")))
                            .or_else(|| {
                                m.get("webm")
                                    .and_then(|webm| webm.get("max").or_else(|| webm.get("480")))
                            })
                            .and_then(Value::as_str)
                            .map(str::to_string)
                    })
                    .collect()
            })
            .unwrap_or_default(),
        ..Default::default()
    }
}

/// Fetch Steam community tags for a set of apps.
///
/// Uses Steam's own JSON endpoint (`actions/ajaxgetstoretags`), which returns
/// `{"tags":[{"tagid":..,"name":"…"}]}` — the same first-party API host as
/// `appdetails`, so no HTML scraping is involved. Apps with no tags are
/// omitted. Individual failures are skipped rather than failing the batch.
pub async fn fetch_tags(
    client: &reqwest::Client,
    app_ids: &[u32],
    cc: &str,
    lang: &str,
) -> BTreeMap<u32, Vec<String>> {
    let mut out = BTreeMap::new();
    for app_id in app_ids {
        let url = format!(
            "https://store.steampowered.com/actions/ajaxgetstoretags?appid={app_id}&cc={cc}&l={lang}"
        );
        let Ok(response) = client.get(&url).send().await else {
            continue;
        };
        let Ok(value) = response.json::<Value>().await else {
            continue;
        };
        let tags: Vec<String> = value
            .get("tags")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(|tag| tag.get("name").and_then(Value::as_str).map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        if !tags.is_empty() {
            out.insert(*app_id, tags);
        }
    }
    out
}

fn string(value: Option<&Value>) -> Option<String> {
    value.and_then(Value::as_str).map(str::to_string)
}

fn string_array(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn object_names(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.get("description").and_then(Value::as_str))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}
