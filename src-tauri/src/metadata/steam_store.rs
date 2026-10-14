//! Keyless Steam store metadata.
//!
//! Endpoints:
//! - `https://store.steampowered.com/api/appdetails?appids=<csv>` — genres,
//!   developers, publishers, feature-flag categories, release date,
//!   header/capsule images, and the Metacritic critic score.
//! - `https://store.steampowered.com/actions/ajaxgetstoretags?appid=<id>` —
//!   Steam community tags.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::error::AppResult;
use crate::steam::models::AppMeta;

/// Fetch normalized metadata for up to ~100 appids in one request.
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

    let ids = app_ids
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let url =
        format!("https://store.steampowered.com/api/appdetails?appids={ids}&cc={cc}&l={lang}");
    let response = client.get(&url).send().await?;
    let value: Value = response.json().await?;

    if let Some(map) = value.as_object() {
        for (key, entry) in map {
            let Ok(app_id) = key.parse::<u32>() else {
                continue;
            };
            if !entry.get("success").and_then(Value::as_bool).unwrap_or(false) {
                continue;
            }
            let Some(data) = entry.get("data") else {
                continue;
            };
            out.insert(app_id, parse_details(app_id, data));
        }
    }
    Ok(out)
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
