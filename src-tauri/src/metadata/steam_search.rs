//! Keyless Steam Store search provider.
//!
//! Maps game titles to official Steam app IDs via Steam's public store search
//! API (`https://store.steampowered.com/api/storesearch/?term=...`). This allows
//! imported non-Steam shortcuts, indie games, and ROMs that also exist on Steam
//! to receive official Steam store metadata, tags, and artwork.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::AppResult;

const STEAM_SEARCH_URL: &str = "https://store.steampowered.com/api/storesearch/";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteamSearchResult {
    pub id: u32,
    pub name: String,
    pub tiny_image: Option<String>,
}

/// Strip common file suffixes and release metadata tags for clean searching.
pub fn clean_search_term(name: &str) -> String {
    let mut term = name.trim();
    for ext in &[".exe", ".iso", ".zip", ".7z", ".rar", ".rpy", ".sh"] {
        if let Some(stripped) = term.strip_suffix(ext) {
            term = stripped;
        }
    }
    let mut result = String::new();
    let mut in_paren = false;
    let mut in_bracket = false;
    for ch in term.chars() {
        match ch {
            '(' => in_paren = true,
            ')' => in_paren = false,
            '[' => in_bracket = true,
            ']' => in_bracket = false,
            _ => {
                if !in_paren && !in_bracket {
                    result.push(ch);
                }
            }
        }
    }
    let cleaned = result.trim().replace(['_', '-'], " ");
    cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Search the Steam Store for a game title and return matching games.
pub async fn search_steam_store(
    client: &reqwest::Client,
    title: &str,
    cc: &str,
    lang: &str,
) -> AppResult<Vec<SteamSearchResult>> {
    let clean = clean_search_term(title);
    if clean.is_empty() {
        return Ok(Vec::new());
    }

    let response = client
        .get(STEAM_SEARCH_URL)
        .query(&[("term", clean.as_str()), ("cc", cc), ("l", lang)])
        .send()
        .await?;

    if !response.status().is_success() {
        return Ok(Vec::new());
    }

    let body: Value = response.json().await?;
    let items = body.get("items").and_then(Value::as_array);
    let Some(items) = items else {
        return Ok(Vec::new());
    };

    let mut results = Vec::new();
    for item in items {
        if let (Some(id), Some(name)) = (
            item.get("id").and_then(Value::as_u64).map(|v| v as u32),
            item.get("name").and_then(Value::as_str),
        ) {
            results.push(SteamSearchResult {
                id,
                name: name.to_string(),
                tiny_image: item
                    .get("tiny_image")
                    .and_then(Value::as_str)
                    .map(str::to_string),
            });
        }
    }

    Ok(results)
}

/// Look up the first / best matching Steam app id for a game title.
pub async fn find_steam_app_id(
    client: &reqwest::Client,
    title: &str,
    cc: &str,
    lang: &str,
) -> AppResult<Option<u32>> {
    let results = search_steam_store(client, title, cc, lang).await?;
    let clean_lower = clean_search_term(title).to_lowercase();
    // Prefer exact name match, otherwise first result
    let best = results
        .iter()
        .find(|res| res.name.trim().to_lowercase() == clean_lower)
        .or_else(|| results.first());

    Ok(best.map(|res| res.id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_steam_search_json() {
        let raw = r#"{
            "total": 1,
            "items": [
                {
                    "id": 620,
                    "name": "Portal 2",
                    "tiny_image": "https://cdn.akamai.steamstatic.com/steam/apps/620/capsule_sm_120.jpg"
                }
            ]
        }"#;
        let body: Value = serde_json::from_str(raw).unwrap();
        let items = body.get("items").and_then(Value::as_array).unwrap();
        let item = &items[0];
        let id = item.get("id").and_then(Value::as_u64).unwrap() as u32;
        let name = item.get("name").and_then(Value::as_str).unwrap();
        assert_eq!(id, 620);
        assert_eq!(name, "Portal 2");
    }
}
