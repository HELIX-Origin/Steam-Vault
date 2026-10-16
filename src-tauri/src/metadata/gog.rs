//! GOG Store metadata provider (keyless, official GOG Storefront and Galaxy API).

use serde_json::Value;
use crate::error::AppResult;
use crate::steam::models::AppMeta;

/// Look up a game on GOG Catalog API and return normalized [`AppMeta`].
pub async fn gog_lookup(client: &reqwest::Client, name: &str) -> AppResult<Option<AppMeta>> {
    let clean = crate::metadata::vndb::clean_search_term(name);
    if clean.is_empty() {
        return Ok(None);
    }

    let url = "https://catalog.gog.com/v1/catalog";
    let resp = client
        .get(url)
        .query(&[
            ("query", clean.as_str()),
            ("limit", "1"),
            ("order", "desc:score"),
        ])
        .send()
        .await?;

    if !resp.status().is_success() {
        return Ok(None);
    }

    let data: Value = resp.json().await?;
    let product = data
        .get("products")
        .and_then(Value::as_array)
        .and_then(|arr| arr.first());

    let Some(prod) = product else {
        return Ok(None);
    };

    let title = prod.get("title").and_then(Value::as_str).map(String::from);
    let vertical_cover = prod.get("coverVertical").and_then(Value::as_str).map(String::from);
    let horizontal_cover = prod.get("coverHorizontal").and_then(Value::as_str).map(String::from);
    let galaxy_bg = prod.get("galaxyBackgroundImage").and_then(Value::as_str).map(String::from);
    let logo = prod.get("logo").and_then(Value::as_str).map(String::from);

    let developers = prod
        .get("developers")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(Value::as_str)
                .map(String::from)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let publishers = prod
        .get("publishers")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(Value::as_str)
                .map(String::from)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let genres = prod
        .get("genres")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|g| g.get("name").and_then(Value::as_str))
                .map(String::from)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let tags = prod
        .get("tags")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|t| t.get("name").and_then(Value::as_str))
                .map(String::from)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let release_date = prod
        .get("releaseDate")
        .and_then(Value::as_str)
        .map(String::from);

    let release_year = release_date.as_deref().and_then(|d| {
        d.split('-').next().and_then(|y| y.parse::<i32>().ok())
    });

    let mut screenshots = Vec::new();
    if let Some(screens) = prod.get("screenshots").and_then(Value::as_array) {
        for s in screens.iter().take(6) {
            if let Some(src) = s.as_str() {
                screenshots.push(src.to_string());
            }
        }
    }

    let capsule = vertical_cover.clone().or_else(|| horizontal_cover.clone());
    let header = horizontal_cover.clone().or_else(|| vertical_cover.clone());

    Ok(Some(AppMeta {
        app_id: 0,
        name: title,
        developers,
        publishers,
        genres,
        tags,
        release_date,
        release_year,
        header_image: header,
        capsule_image: capsule,
        hero_image: galaxy_bg,
        logo_image: logo,
        screenshots,
        fetched_at: Some(chrono::Utc::now().timestamp()),
        ..Default::default()
    }))
}
