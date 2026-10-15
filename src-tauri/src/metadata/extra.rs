//! Keyless supplemental providers: PCGamingWiki and Wikipedia.
//!
//! These add developer/publisher/engine/release data and descriptions without
//! any API key. They are best-effort fallbacks and never required.

use serde_json::Value;

use crate::error::AppResult;
use crate::steam::models::AppMeta;

/// Look up a game on PCGamingWiki and map the first match into [`AppMeta`].
pub async fn pcgamingwiki_lookup(
    client: &reqwest::Client,
    name: &str,
) -> AppResult<Option<AppMeta>> {
    let clean = crate::metadata::vndb::clean_search_term(name);
    if clean.is_empty() {
        return Ok(None);
    }
    // Escape single quotes for MediaWiki Cargo SQL where clause
    let escaped = clean.replace('\'', "''");
    let where_clause = format!("Infobox_game._pageName LIKE '%{escaped}%'");
    let response = client
        .get("https://www.pcgamingwiki.com/w/api.php")
        .query(&[
            ("action", "cargoquery"),
            ("format", "json"),
            ("tables", "Infobox_game"),
            (
                "fields",
                "Infobox_game._pageName=Page,Infobox_game.Developers,Infobox_game.Publishers,Infobox_game.Genres,Infobox_game.Engines,Infobox_game.Released",
            ),
            ("where", where_clause.as_str()),
            ("limit", "1"),
        ])
        .send()
        .await?;
    let value: Value = response.json().await?;

    let row = value
        .get("cargoquery")
        .and_then(Value::as_array)
        .and_then(|rows| rows.first())
        .and_then(|row| row.get("title"));
    let Some(row) = row else {
        return Ok(None);
    };

    let split = |key: &str| -> Vec<String> {
        row.get(key)
            .and_then(Value::as_str)
            .map(|text| {
                text.split(',')
                    .map(|part| part.trim().to_string())
                    .filter(|part| !part.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    };

    let released = row
        .get("Released")
        .and_then(Value::as_str)
        .map(str::to_string);
    let release_year = released.as_deref().and_then(|r| {
        // Find 4-digit year in string
        r.split(|c: char| !c.is_ascii_digit())
            .find(|token| token.len() == 4)
            .and_then(|y| y.parse::<i32>().ok())
    });

    let engines = split("Engines");
    let mut tags = Vec::new();
    for engine in engines {
        tags.push(format!("Engine: {engine}"));
    }

    Ok(Some(AppMeta {
        app_id: 0,
        name: row.get("Page").and_then(Value::as_str).map(str::to_string),
        developers: split("Developers"),
        publishers: split("Publishers"),
        genres: split("Genres"),
        tags,
        release_date: released,
        release_year,
        fetched_at: Some(chrono::Utc::now().timestamp()),
        ..Default::default()
    }))
}

/// Fetch the intro extract of a Wikipedia article.
pub async fn wikipedia_summary(client: &reqwest::Client, title: &str) -> AppResult<Option<String>> {
    let response = client
        .get("https://en.wikipedia.org/w/api.php")
        .query(&[
            ("action", "query"),
            ("format", "json"),
            ("prop", "extracts"),
            ("exintro", "1"),
            ("explaintext", "1"),
            ("redirects", "1"),
            ("titles", title),
        ])
        .send()
        .await?;
    let value: Value = response.json().await?;

    let extract = value
        .get("query")
        .and_then(|query| query.get("pages"))
        .and_then(Value::as_object)
        .and_then(|pages| pages.values().next())
        .and_then(|page| page.get("extract"))
        .and_then(Value::as_str)
        .map(str::to_string);

    Ok(extract)
}
