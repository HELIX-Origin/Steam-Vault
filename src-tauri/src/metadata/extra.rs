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

/// Clean wikitext formatting (markup, templates, references, links).
pub fn clean_wikitext(raw: &str) -> String {
    let mut s = raw.to_string();
    while let Some(start) = s.find("<!--") {
        if let Some(end) = s[start..].find("-->") {
            s.replace_range(start..start + end + 3, "");
        } else {
            break;
        }
    }
    while let Some(start) = s.to_lowercase().find("<ref") {
        if let Some(end) = s[start..].find('>') {
            let tag = &s[start..start + end + 1];
            if tag.ends_with("/>") {
                s.replace_range(start..start + end + 1, "");
            } else if let Some(close_ref) = s[start..].to_lowercase().find("</ref>") {
                s.replace_range(start..start + close_ref + 6, "");
            } else {
                s.replace_range(start..start + end + 1, "");
            }
        } else {
            break;
        }
    }
    while let Some(start) = s.find("{{") {
        if let Some(end) = s[start..].find("}}") {
            s.replace_range(start..start + end + 2, " ");
        } else {
            break;
        }
    }
    while let Some(start) = s.find("[[") {
        if let Some(end) = s[start..].find("]]") {
            let inner = &s[start + 2..start + end];
            let display = if let Some(pipe) = inner.rfind('|') {
                &inner[pipe + 1..]
            } else {
                inner
            };
            let display_str = display.to_string();
            s.replace_range(start..start + end + 2, &display_str);
        } else {
            break;
        }
    }
    s = s.replace("'''", "").replace("''", "");
    s.trim().to_string()
}

pub fn get_infobox_field(wikitext: &str, name: &str) -> Option<String> {
    let mut collecting = false;
    let mut accumulated = String::new();

    for line in wikitext.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('|') {
            if collecting {
                let cleaned = clean_wikitext(&accumulated);
                if !cleaned.is_empty() {
                    return Some(cleaned);
                }
                collecting = false;
                accumulated.clear();
            }

            if let Some(rest) = trimmed.strip_prefix('|') {
                if let Some((key, val)) = rest.split_once('=') {
                    if key.trim().eq_ignore_ascii_case(name) {
                        collecting = true;
                        accumulated.push_str(val);
                        accumulated.push('\n');
                    }
                }
            }
        } else if trimmed.starts_with("}}") {
            if collecting {
                let cleaned = clean_wikitext(&accumulated);
                if !cleaned.is_empty() {
                    return Some(cleaned);
                }
                break;
            }
        } else if collecting {
            accumulated.push_str(line);
            accumulated.push('\n');
        }
    }

    if collecting {
        let cleaned = clean_wikitext(&accumulated);
        if !cleaned.is_empty() {
            return Some(cleaned);
        }
    }

    None
}

pub fn split_wiki_items(raw: &str) -> Vec<String> {
    raw.split([',', '\n', '/', ';', '•'])
        .map(|s| s.trim().trim_start_matches('*').trim().to_string())
        .filter(|s| !s.is_empty() && s.len() > 1)
        .collect()
}

pub fn parse_wiki_platforms(plat_str: &str) -> Vec<String> {
    let lp = plat_str.to_lowercase();
    let mut platforms = Vec::new();
    if lp.contains("gamecube") || lp.contains("game cube") {
        platforms.push("GameCube".to_string());
    }
    if lp.contains("wii u") {
        platforms.push("Wii U".to_string());
    } else if lp.contains("wii") {
        platforms.push("Wii".to_string());
    }
    if lp.contains("nintendo switch") || lp.contains("switch") {
        platforms.push("Nintendo Switch".to_string());
    }
    if lp.contains("nintendo 64") || lp.contains("n64") {
        platforms.push("Nintendo 64".to_string());
    }
    if lp.contains("super nintendo") || lp.contains("snes") || lp.contains("super famicom") {
        platforms.push("Super Nintendo".to_string());
    }
    if lp.contains("nintendo entertainment system") || lp.contains("nes") || lp.contains("famicom")
    {
        platforms.push("NES".to_string());
    }
    if lp.contains("game boy advance") || lp.contains("gba") {
        platforms.push("Game Boy Advance".to_string());
    }
    if lp.contains("game boy color") || lp.contains("gbc") {
        platforms.push("Game Boy Color".to_string());
    }
    if (lp.contains("game boy") || lp.contains("gb"))
        && !platforms
            .iter()
            .any(|p| p == "Game Boy Advance" || p == "Game Boy Color")
    {
        platforms.push("Game Boy".to_string());
    }
    if lp.contains("nintendo ds") || lp.contains("nds") {
        platforms.push("Nintendo DS".to_string());
    }
    if lp.contains("nintendo 3ds") || lp.contains("3ds") {
        platforms.push("Nintendo 3DS".to_string());
    }
    if lp.contains("playstation 2") || lp.contains("ps2") {
        platforms.push("PlayStation 2".to_string());
    }
    if lp.contains("playstation 3") || lp.contains("ps3") {
        platforms.push("PlayStation 3".to_string());
    }
    if lp.contains("playstation 4") || lp.contains("ps4") {
        platforms.push("PlayStation 4".to_string());
    }
    if lp.contains("playstation 5") || lp.contains("ps5") {
        platforms.push("PlayStation 5".to_string());
    }
    if lp.contains("playstation portable") || lp.contains("psp") {
        platforms.push("PlayStation Portable".to_string());
    }
    if lp.contains("playstation vita") || lp.contains("ps vita") || lp.contains("psvita") {
        platforms.push("PlayStation Vita".to_string());
    }
    if (lp.contains("playstation") || lp.contains("ps1") || lp.contains("psx"))
        && !platforms.iter().any(|p| p.starts_with("PlayStation "))
    {
        platforms.push("PlayStation".to_string());
    }
    if lp.contains("genesis") || lp.contains("mega drive") {
        platforms.push("Sega Genesis".to_string());
    }
    if lp.contains("saturn") {
        platforms.push("Sega Saturn".to_string());
    }
    if lp.contains("dreamcast") {
        platforms.push("Dreamcast".to_string());
    }
    if lp.contains("game gear") {
        platforms.push("Sega Game Gear".to_string());
    }
    if lp.contains("master system") {
        platforms.push("Sega Master System".to_string());
    }
    platforms
}

/// Fetch metadata for a console, retro console, or PC game from Wikipedia video game infobox.
pub async fn wikipedia_game_lookup(
    client: &reqwest::Client,
    title: &str,
) -> AppResult<Option<AppMeta>> {
    let clean = crate::metadata::vndb::clean_search_term(title);
    if clean.is_empty() {
        return Ok(None);
    }

    let mut page_title = None;
    let mut wikitext = None;

    // 1. Try direct page parse with redirects
    if let Ok(res) = client
        .get("https://en.wikipedia.org/w/api.php")
        .query(&[
            ("action", "parse"),
            ("format", "json"),
            ("page", clean.as_str()),
            ("prop", "wikitext"),
            ("section", "0"),
            ("redirects", "1"),
        ])
        .send()
        .await
    {
        if let Ok(val) = res.json::<Value>().await {
            if let Some(text) = val.pointer("/parse/wikitext/*").and_then(Value::as_str) {
                wikitext = Some(text.to_string());
                page_title = val
                    .pointer("/parse/title")
                    .and_then(Value::as_str)
                    .map(str::to_string);
            }
        }
    }

    // 2. If direct parse didn't find an infobox or page, search Wikipedia
    if wikitext.is_none()
        || wikitext
            .as_deref()
            .is_some_and(|w| !w.to_lowercase().contains("infobox video game"))
    {
        let search_query = format!("{clean} video game");
        if let Ok(s_res) = client
            .get("https://en.wikipedia.org/w/api.php")
            .query(&[
                ("action", "query"),
                ("format", "json"),
                ("list", "search"),
                ("srsearch", search_query.as_str()),
                ("srlimit", "1"),
            ])
            .send()
            .await
        {
            if let Ok(s_val) = s_res.json::<Value>().await {
                if let Some(target_title) = s_val
                    .pointer("/query/search/0/title")
                    .and_then(Value::as_str)
                {
                    if let Ok(p_res) = client
                        .get("https://en.wikipedia.org/w/api.php")
                        .query(&[
                            ("action", "parse"),
                            ("format", "json"),
                            ("page", target_title),
                            ("prop", "wikitext"),
                            ("section", "0"),
                            ("redirects", "1"),
                        ])
                        .send()
                        .await
                    {
                        if let Ok(p_val) = p_res.json::<Value>().await {
                            if let Some(text) =
                                p_val.pointer("/parse/wikitext/*").and_then(Value::as_str)
                            {
                                wikitext = Some(text.to_string());
                                page_title = Some(target_title.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    let Some(wiki) = wikitext else {
        return Ok(None);
    };

    let dev_raw = get_infobox_field(&wiki, "developer");
    let pub_raw = get_infobox_field(&wiki, "publisher");
    let genre_raw = get_infobox_field(&wiki, "genre");
    let plat_raw = get_infobox_field(&wiki, "platforms");
    let released_raw = get_infobox_field(&wiki, "released");

    let developers = dev_raw.as_deref().map(split_wiki_items).unwrap_or_default();
    let publishers = pub_raw.as_deref().map(split_wiki_items).unwrap_or_default();
    let genres = genre_raw
        .as_deref()
        .map(split_wiki_items)
        .unwrap_or_default();
    let console_platforms = plat_raw
        .as_deref()
        .map(parse_wiki_platforms)
        .unwrap_or_default();

    let release_year = released_raw.as_deref().and_then(|r| {
        r.split(|c: char| !c.is_ascii_digit())
            .find(|token| token.len() == 4)
            .and_then(|y| y.parse::<i32>().ok())
    });

    let mut tags = Vec::new();
    for p in &console_platforms {
        tags.push(format!("Platform: {p}"));
    }

    let summary = if let Some(ref t) = page_title {
        wikipedia_summary(client, t).await.unwrap_or(None)
    } else {
        None
    };

    Ok(Some(AppMeta {
        app_id: 0,
        name: page_title.or(Some(clean)),
        developers,
        publishers,
        genres,
        categories: console_platforms,
        tags,
        release_date: released_raw,
        release_year,
        short_description: summary,
        fetched_at: Some(chrono::Utc::now().timestamp()),
        ..Default::default()
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_wikitext_infobox() {
        let wikitext = r#"
            {{Infobox video game
            | title = Super Smash Bros. Melee
            | developer = [[HAL Laboratory]]
            | publisher = [[Nintendo]]
            | director = Masahiro Sakurai
            | genre = [[Fighting game|Fighting]]
            | platforms = [[GameCube]]
            | released = November 21, 2001
            }}
        "#;
        assert_eq!(
            get_infobox_field(wikitext, "developer").as_deref(),
            Some("HAL Laboratory")
        );
        assert_eq!(
            get_infobox_field(wikitext, "publisher").as_deref(),
            Some("Nintendo")
        );
        assert_eq!(
            get_infobox_field(wikitext, "genre").as_deref(),
            Some("Fighting")
        );
        let plats = parse_wiki_platforms(&get_infobox_field(wikitext, "platforms").unwrap());
        assert_eq!(plats, vec!["GameCube"]);
    }
}
