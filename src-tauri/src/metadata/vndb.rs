//! Keyless Visual Novel Database (VNDB) metadata provider.
//!
//! VNDB is the authoritative database for visual novels, Ren'Py titles,
//! narrative indie games, and anime-style PC titles.
//! Endpoint: `https://api.vndb.org/kana/vn` (POST JSON, keyless, public).

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::AppResult;
use crate::steam::models::AppMeta;

const VNDB_API_URL: &str = "https://api.vndb.org/kana/vn";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VndbResult {
    pub id: String,
    pub title: String,
    pub alttitle: Option<String>,
    pub released: Option<String>,
    #[serde(default)]
    pub developers: Vec<VndbDeveloper>,
    pub image: Option<VndbImage>,
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<VndbTag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VndbDeveloper {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VndbImage {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VndbTag {
    pub name: String,
}

/// Search VNDB for a game title and map the closest match into [`AppMeta`].
pub async fn vndb_lookup(client: &reqwest::Client, title: &str) -> AppResult<Option<AppMeta>> {
    let clean = clean_search_term(title);
    if clean.is_empty() {
        return Ok(None);
    }

    let payload = json!({
        "filters": ["search", "=", clean],
        "fields": "title, alttitle, released, developers.name, image.url, description, tags.name"
    });

    let response = client
        .post(VNDB_API_URL)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?;

    if !response.status().is_success() {
        return Ok(None);
    }

    let body: Value = response.json().await?;
    let results = body.get("results").and_then(Value::as_array);
    let Some(results) = results else {
        return Ok(None);
    };

    let Some(first) = results.first() else {
        return Ok(None);
    };

    let item: VndbResult = match serde_json::from_value(first.clone()) {
        Ok(item) => item,
        Err(_) => return Ok(None),
    };

    Ok(Some(vndb_to_app_meta(item)))
}

/// Map a VNDB API response into standard [`AppMeta`].
pub fn vndb_to_app_meta(item: VndbResult) -> AppMeta {
    let release_year = item
        .released
        .as_deref()
        .and_then(|r| r.split('-').next())
        .and_then(|y| y.parse::<i32>().ok());

    let developers: Vec<String> = item
        .developers
        .into_iter()
        .map(|d| d.name.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    // In VN publishing, the original developer is frequently the publisher as well
    let publishers = developers.clone();

    let mut genres = vec!["Visual Novel".to_string()];
    let tags: Vec<String> = item
        .tags
        .into_iter()
        .map(|t| t.name.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // Include prominent genre tags
    for tag in &tags {
        let lower = tag.to_lowercase();
        if (lower.contains("adventure")
            || lower.contains("romance")
            || lower.contains("mystery")
            || lower.contains("horror")
            || lower.contains("sci-fi")
            || lower.contains("fantasy")
            || lower.contains("comedy")
            || lower.contains("drama"))
            && !genres.contains(tag)
        {
            genres.push(tag.clone());
        }
    }

    let image_url = item.image.map(|img| img.url);

    AppMeta {
        app_id: 0,
        name: Some(item.title),
        developers,
        publishers,
        genres,
        categories: vec!["Single-player".to_string()],
        tags,
        release_date: item.released,
        release_year,
        short_description: item.description,
        header_image: image_url.clone(),
        capsule_image: image_url,
        user_score: None,
        fetched_at: Some(chrono::Utc::now().timestamp()),
    }
}

/// Strip common file suffixes and release metadata tags for clean searching.
pub fn clean_search_term(name: &str) -> String {
    let mut term = name.trim();
    // Strip file extensions if present
    for ext in &[".exe", ".iso", ".zip", ".7z", ".rar", ".rpy", ".sh"] {
        if let Some(stripped) = term.strip_suffix(ext) {
            term = stripped;
        }
    }
    // Remove release bracket tags e.g. [v1.0], (Windows), [En]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleans_search_term_correctly() {
        assert_eq!(
            clean_search_term("Katawa Shoujo (Windows) [v1.3.1].exe"),
            "Katawa Shoujo"
        );
        assert_eq!(
            clean_search_term("Doki_Doki_Literature_Club"),
            "Doki Doki Literature Club"
        );
        assert_eq!(
            clean_search_term("Fate/Stay Night [Realta Nua]"),
            "Fate/Stay Night"
        );
    }

    #[test]
    fn maps_vndb_result_to_app_meta() {
        let result = VndbResult {
            id: "v949".into(),
            title: "Katawa Shoujo".into(),
            alttitle: None,
            released: Some("2012-01-04".into()),
            developers: vec![VndbDeveloper {
                name: "Four Leaf Studios".into(),
            }],
            image: Some(VndbImage {
                url: "https://t.vndb.org/cv/99/12399.jpg".into(),
            }),
            description: Some("A romantic visual novel set in Yamaku High School.".into()),
            tags: vec![
                VndbTag {
                    name: "Romance".into(),
                },
                VndbTag {
                    name: "Drama".into(),
                },
            ],
        };
        let meta = vndb_to_app_meta(result);
        assert_eq!(meta.name.as_deref(), Some("Katawa Shoujo"));
        assert_eq!(meta.developers, vec!["Four Leaf Studios"]);
        assert_eq!(meta.publishers, vec!["Four Leaf Studios"]);
        assert_eq!(meta.release_year, Some(2012));
        assert!(meta.genres.contains(&"Visual Novel".to_string()));
        assert!(meta.genres.contains(&"Romance".to_string()));
        assert!(meta.genres.contains(&"Drama".to_string()));
    }
}
