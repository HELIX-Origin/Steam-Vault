//! Universal Metadata & Artwork Provider Engine (`metadata-universal`).
//!
//! Modeled after Playnite's `OnDemandMetadataProvider` and GOG Galaxy 2.0:
//! automatically resolves and merges rich metadata and artwork for any game
//! (PC, console/emulated ROM, itch.io, standalone) across keyless and optional
//! authenticated providers in Rust via `reqwest` (zero WebView CORS failures):
//! 1. Local Engine & Manifest Inspector (`metadata::local`)
//! 2. Libretro Thumbnails CDN (`thumbnails.libretro.com` for console ROMs)
//! 3. Steam Store Search & `appdetails` API
//! 4. GOG Catalog v1 API (`catalog.gog.com/v1/catalog`)
//! 5. Wikipedia + Wikidata MediaWiki API (`P178` developer, `P123` publisher, `P136` genre, `P577` release date)
//! 6. PCGamingWiki Cargo API (`Infobox_game`)

use std::collections::HashSet;
use std::path::Path;
use std::time::Duration;

use serde_json::Value;

use crate::plugins::title_normalizer::{canonical_title_key, normalize_game_title};
use crate::steam::models::{AppMeta, Game, GameKind};

/// Resolve and enrich metadata for any game automatically across all providers.
pub async fn resolve_game_metadata(
    client: &reqwest::Client,
    game: &Game,
    country_code: &str,
    language: &str,
) -> AppMeta {
    let clean_title = {
        let n = normalize_game_title(&game.name);
        if n.is_empty() {
            game.name.clone()
        } else {
            n
        }
    };

    let mut meta = AppMeta {
        app_id: game.app_id,
        name: Some(clean_title.clone()),
        ..Default::default()
    };
    let mut enhanced_sources: Vec<String> = Vec::new();

    let is_emulated =
        game.platform == crate::platforms::Platform::EmulationStation || game.console.is_some();

    // 1. Primary Steam Store lookup if this is a native Steam game
    if game.kind == GameKind::Steam && game.app_id != 0 {
        if let Ok(map) =
            super::steam_store::fetch_app_details(client, &[game.app_id], country_code, language)
                .await
        {
            if let Some(steam_meta) = map.get(&game.app_id) {
                merge_into(&mut meta, steam_meta);
                enhanced_sources.push("Steam Store".to_string());
            }
        }
    }

    // 2. Local directory manifest & engine inspection (itch.io receipt, RPG Maker, Ren'Py, Unity, Godot, PE)
    let install_dir = game
        .install_dir
        .as_deref()
        .map(Path::new)
        .or_else(|| game.exe.as_deref().and_then(|e| Path::new(e).parent()));
    let exe_path = game.exe.as_deref().map(Path::new);
    if let Some(local_meta) = super::local::inspect_local_game(install_dir, exe_path) {
        merge_into(&mut meta, &local_meta);
        enhanced_sources.push("Local Manifest".to_string());
    }

    // 3. Libretro Thumbnails CDN for console / emulated ROMs
    if is_emulated {
        if let Some(ref console_name) = game.console {
            if !meta.categories.contains(console_name) {
                meta.categories.push(console_name.clone());
            }
            if let Some(libretro_meta) =
                fetch_libretro_artwork(client, &clean_title, console_name).await
            {
                merge_into(&mut meta, &libretro_meta);
                enhanced_sources.push("Libretro Thumbnails".to_string());
            }
        }
    }

    // 4. Steam Store Search fallback (for PC / standalone / cross-platform games, not exclusive console ROMs)
    if !is_emulated && needs_enrichment(&meta) {
        if let Ok(Some(steam_id)) =
            super::steam_search::find_steam_app_id(client, &clean_title, country_code, language)
                .await
        {
            if let Ok(map) =
                super::steam_store::fetch_app_details(client, &[steam_id], country_code, language)
                    .await
            {
                if let Some(store_meta) = map.get(&steam_id) {
                    let store_name = store_meta.name.as_deref().unwrap_or("");
                    if titles_match(&clean_title, store_name)
                        && super::developers::validate_metadata_candidate(
                            &clean_title,
                            store_name,
                            &store_meta.developers,
                            &store_meta.publishers,
                        )
                    {
                        merge_into(&mut meta, store_meta);
                        enhanced_sources.push("Steam Store".to_string());
                    }
                }
            }
        }
    }

    // 5. GOG Catalog v1 API (keyless structured JSON for PC & cross-platform games)
    if !is_emulated && needs_enrichment(&meta) {
        if let Some(gog_meta) = fetch_gog_catalog_metadata(client, &clean_title).await {
            merge_into(&mut meta, &gog_meta);
            enhanced_sources.push("GOG Catalog".to_string());
        }
    }

    // 6. Wikipedia + Wikidata MediaWiki API (universal coverage for console exclusives like Zelda, PC games, & indies)
    if needs_enrichment(&meta) {
        if let Some(wiki_meta) =
            fetch_wikipedia_wikidata_metadata(client, &clean_title, game.console.as_deref()).await
        {
            merge_into(&mut meta, &wiki_meta);
            enhanced_sources.push("Wikipedia & Wikidata".to_string());
        }
    }

    // 7. PCGamingWiki Cargo API fallback if credits or genres are still missing
    if !is_emulated
        && (meta.developers.is_empty() || meta.publishers.is_empty() || meta.genres.is_empty())
    {
        if let Some(pcgw_meta) = fetch_pcgamingwiki_metadata(client, &clean_title).await {
            merge_into(&mut meta, &pcgw_meta);
            enhanced_sources.push("PCGamingWiki".to_string());
        }
    }

    // 8. Hand-rolled developer and game metadata fallback (per-developer authoritative data)
    if meta.developers.is_empty()
        || meta.publishers.is_empty()
        || meta.genres.is_empty()
        || meta.capsule_image.is_none()
        || meta.short_description.is_none()
    {
        if let Some(backup_meta) = super::developers::backup_metadata_for_title(&clean_title) {
            merge_into(&mut meta, &backup_meta);
            enhanced_sources.push("Developer Hand-Rolled Database".to_string());
        }
    }

    // Ensure hero_image falls back to header_image or first screenshot if still empty
    if meta.hero_image.is_none() {
        meta.hero_image = meta
            .screenshots
            .first()
            .cloned()
            .or_else(|| meta.header_image.clone())
            .or_else(|| meta.capsule_image.clone());
    }
    if meta.capsule_image.is_none() {
        meta.capsule_image = meta.header_image.clone();
    }

    if !enhanced_sources.is_empty() {
        meta.enhanced_by = enhanced_sources;
    }
    meta.fetched_at = Some(chrono::Utc::now().timestamp());
    meta
}

fn needs_enrichment(meta: &AppMeta) -> bool {
    meta.short_description.is_none()
        || meta.developers.is_empty()
        || meta.capsule_image.is_none()
        || meta.hero_image.is_none()
        || meta.movies.is_empty()
}

fn contains_non_game_media_type(title_lower: &str) -> bool {
    const KEYWORDS: &[&str] = &[
        "soundtrack",
        "ost",
        "original soundtrack",
        "album",
        "score",
        "discography",
        "music of",
        "artbook",
        "art book",
        "dlc",
        "expansion pack",
        "season pass",
        "strategy guide",
        "walkthrough",
        "bonus disc",
        "trailer",
        "teaser",
    ];
    let lower = title_lower.to_ascii_lowercase();
    for kw in KEYWORDS {
        if lower.contains(kw) {
            return true;
        }
    }
    false
}

fn titles_match(target: &str, candidate: &str) -> bool {
    let t_lower = target.to_ascii_lowercase();
    let c_lower = candidate.to_ascii_lowercase();

    // If candidate has soundtrack/album/artbook/DLC keywords but target does not, reject
    if contains_non_game_media_type(&c_lower) && !contains_non_game_media_type(&t_lower) {
        return false;
    }

    let k1 = canonical_title_key(target);
    let k2 = canonical_title_key(candidate);
    if k1.is_empty() || k2.is_empty() {
        return false;
    }

    if k1 == k2 {
        return true;
    }

    // Check if one contains the other, but reject if the diff contains numbers/numerals (e.g. Galaxy vs Galaxy 2)
    if k1.contains(&k2) || k2.contains(&k1) {
        let (longer, shorter) = if k1.len() > k2.len() {
            (&k1, &k2)
        } else {
            (&k2, &k1)
        };
        let remainder = longer.replace(shorter, "");

        // If remainder contains digits or roman numerals like "2", "3", "iv", "ii", "iii", they are different sequels!
        let has_sequel_marker = remainder.chars().any(|c| c.is_ascii_digit())
            || remainder.starts_with("ii")
            || remainder.starts_with("iii")
            || remainder.starts_with("iv")
            || remainder.starts_with("v")
            || remainder.starts_with("vi");
        if has_sequel_marker {
            return false;
        }

        let benign = [
            "edition",
            "definitive",
            "remastered",
            "goty",
            "complete",
            "hd",
            "deluxe",
            "director",
        ];
        if benign.iter().any(|b| remainder.contains(b)) || remainder.len() <= 12 {
            return true;
        }
    }

    false
}

/// Merge non-empty fields from `source` into `target` without overwriting existing populated values.
pub fn merge_into(target: &mut AppMeta, source: &AppMeta) {
    if target.name.is_none() && source.name.is_some() {
        target.name = source.name.clone();
    }
    if target.short_description.is_none() && source.short_description.is_some() {
        target.short_description = source.short_description.clone();
    }
    if target.release_date.is_none() && source.release_date.is_some() {
        target.release_date = source.release_date.clone();
    }
    if target.release_year.is_none() && source.release_year.is_some() {
        target.release_year = source.release_year;
    }
    if target.user_score.is_none() && source.user_score.is_some() {
        target.user_score = source.user_score;
    }
    if target.header_image.is_none() && source.header_image.is_some() {
        target.header_image = source.header_image.clone();
    }
    if target.capsule_image.is_none() && source.capsule_image.is_some() {
        target.capsule_image = source.capsule_image.clone();
    }
    if target.hero_image.is_none() && source.hero_image.is_some() {
        target.hero_image = source.hero_image.clone();
    }
    if target.logo_image.is_none() && source.logo_image.is_some() {
        target.logo_image = source.logo_image.clone();
    }

    for dev in &source.developers {
        if !dev.trim().is_empty() && !target.developers.contains(dev) {
            target.developers.push(dev.clone());
        }
    }
    for pub_name in &source.publishers {
        if !pub_name.trim().is_empty() && !target.publishers.contains(pub_name) {
            target.publishers.push(pub_name.clone());
        }
    }
    for genre in &source.genres {
        if !genre.trim().is_empty() && !target.genres.contains(genre) {
            target.genres.push(genre.clone());
        }
    }
    for cat in &source.categories {
        if !cat.trim().is_empty() && !target.categories.contains(cat) {
            target.categories.push(cat.clone());
        }
    }
    for tag in &source.tags {
        if !tag.trim().is_empty() && !target.tags.contains(tag) {
            target.tags.push(tag.clone());
        }
    }
    for ss in &source.screenshots {
        if !ss.trim().is_empty() && !target.screenshots.contains(ss) && target.screenshots.len() < 12
        {
            target.screenshots.push(ss.clone());
        }
    }
    for mv in &source.movies {
        if !mv.trim().is_empty() && !target.movies.contains(mv) {
            target.movies.push(mv.clone());
        }
    }
    for st in &source.soundtracks {
        if !st.trim().is_empty() && !target.soundtracks.contains(st) {
            target.soundtracks.push(st.clone());
        }
    }
}

// ---------------------------------------------------------------------------
// Libretro Thumbnails CDN (Keyless Retro / Console Boxart, Title, Snap, Logo)
// ---------------------------------------------------------------------------

fn map_console_to_libretro_playlist(console: &str) -> Option<&'static str> {
    let lower = console.trim().to_ascii_lowercase();
    if lower.contains("gamecube") || lower == "gc" || lower == "ngc" {
        Some("Nintendo - GameCube")
    } else if lower.contains("wii u") || lower == "wiiu" {
        Some("Nintendo - Wii U")
    } else if lower.contains("wii") {
        Some("Nintendo - Wii")
    } else if lower.contains("switch") {
        Some("Nintendo - Switch")
    } else if lower.contains("nintendo 64") || lower == "n64" {
        Some("Nintendo - Nintendo 64")
    } else if lower.contains("super nintendo") || lower == "snes" {
        Some("Nintendo - Super Nintendo Entertainment System")
    } else if lower.contains("nintendo entertainment system") || lower == "nes" {
        Some("Nintendo - Nintendo Entertainment System")
    } else if lower.contains("game boy advance") || lower == "gba" {
        Some("Nintendo - Game Boy Advance")
    } else if lower.contains("game boy color") || lower == "gbc" {
        Some("Nintendo - Game Boy Color")
    } else if lower.contains("game boy") || lower == "gb" {
        Some("Nintendo - Game Boy")
    } else if lower.contains("3ds") {
        Some("Nintendo - Nintendo 3DS")
    } else if lower.contains("nintendo ds") || lower == "nds" {
        Some("Nintendo - Nintendo DS")
    } else if lower.contains("playstation 2") || lower == "ps2" {
        Some("Sony - PlayStation 2")
    } else if lower.contains("playstation 3") || lower == "ps3" {
        Some("Sony - PlayStation 3")
    } else if lower.contains("playstation portable") || lower == "psp" {
        Some("Sony - PlayStation Portable")
    } else if lower.contains("vita") {
        Some("Sony - PlayStation Vita")
    } else if lower.contains("playstation") || lower == "psx" || lower == "ps1" {
        Some("Sony - PlayStation")
    } else if lower.contains("dreamcast") {
        Some("Sega - Dreamcast")
    } else if lower.contains("saturn") {
        Some("Sega - Saturn")
    } else if lower.contains("genesis") || lower.contains("mega drive") || lower == "megadrive" {
        Some("Sega - Mega Drive - Genesis")
    } else if lower.contains("neo geo") || lower == "neogeo" {
        Some("SNK - Neo Geo")
    } else if lower.contains("arcade") || lower.contains("mame") {
        Some("MAME")
    } else {
        None
    }
}

/// Sanitize a title for Libretro Thumbnails (`&*/:`"\<>?\|` -> `_`).
fn sanitize_libretro_name(title: &str) -> String {
    let mut s = String::with_capacity(title.len());
    let mut chars = title.chars().peekable();
    while let Some(c) = chars.next() {
        if c == ':' && chars.peek() == Some(&' ') {
            // Libretro uses " - " for ": " subtitle separators
            chars.next();
            s.push_str(" - ");
        } else if matches!(c, '&' | '*' | '/' | ':' | '`' | '"' | '<' | '>' | '?' | '\\' | '|') {
            s.push('_');
        } else {
            s.push(c);
        }
    }
    s
}

fn libretro_title_candidates(clean_title: &str) -> Vec<String> {
    let mut base_names = Vec::new();
    let sanitized = sanitize_libretro_name(clean_title);
    base_names.push(sanitized.clone());

    // Also try inverted article form if title starts with "The " (e.g. "The Legend of Zelda - The Wind Waker" vs "Legend of Zelda, The - The Wind Waker")
    if let Some(rest) = sanitized.strip_prefix("The ") {
        if let Some((main, sub)) = rest.split_once(" - ") {
            base_names.push(format!("{main}, The - {sub}"));
        } else {
            base_names.push(format!("{rest}, The"));
        }
    }

    let regions = [" (USA)", " (Europe)", " (World)", " (USA, Europe)", ""];
    let mut candidates = Vec::new();
    for base in &base_names {
        for reg in regions {
            candidates.push(format!("{base}{reg}"));
        }
    }
    candidates
}

async fn fetch_libretro_artwork(
    client: &reqwest::Client,
    clean_title: &str,
    console: &str,
) -> Option<AppMeta> {
    let playlist = map_console_to_libretro_playlist(console)?;
    let encoded_playlist = urlencoding::encode(playlist);
    let candidates = libretro_title_candidates(clean_title);

    // Find which candidate filename exists in Named_Boxarts
    let mut matched_candidate: Option<String> = None;
    let mut boxart_url: Option<String> = None;

    for cand in &candidates {
        let encoded_file = urlencoding::encode(cand);
        let url = format!(
            "https://thumbnails.libretro.com/{encoded_playlist}/Named_Boxarts/{encoded_file}.png"
        );
        if let Ok(resp) = client
            .head(&url)
            .timeout(Duration::from_millis(2500))
            .send()
            .await
        {
            if resp.status().is_success() {
                boxart_url = Some(url);
                matched_candidate = Some(cand.clone());
                break;
            }
        }
    }

    let cand = matched_candidate?;
    let encoded_file = urlencoding::encode(&cand);
    let snap_url = format!(
        "https://thumbnails.libretro.com/{encoded_playlist}/Named_Snaps/{encoded_file}.png"
    );
    let title_url = format!(
        "https://thumbnails.libretro.com/{encoded_playlist}/Named_Titles/{encoded_file}.png"
    );
    let logo_url = format!(
        "https://thumbnails.libretro.com/{encoded_playlist}/Named_Logos/{encoded_file}.png"
    );

    let snap_ok = client
        .head(&snap_url)
        .timeout(Duration::from_millis(2000))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false);
    let title_ok = client
        .head(&title_url)
        .timeout(Duration::from_millis(2000))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false);
    let logo_ok = client
        .head(&logo_url)
        .timeout(Duration::from_millis(2000))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false);

    let mut screenshots = Vec::new();
    if snap_ok {
        screenshots.push(snap_url.clone());
    }
    if title_ok {
        screenshots.push(title_url.clone());
    }

    let hero_image = if snap_ok {
        Some(snap_url)
    } else if title_ok {
        Some(title_url)
    } else {
        boxart_url.clone()
    };

    Some(AppMeta {
        header_image: boxart_url.clone(),
        capsule_image: boxart_url,
        hero_image,
        logo_image: if logo_ok { Some(logo_url) } else { None },
        screenshots,
        ..Default::default()
    })
}

// ---------------------------------------------------------------------------
// GOG Catalog v1 API (Keyless)
// ---------------------------------------------------------------------------

async fn fetch_gog_catalog_metadata(
    client: &reqwest::Client,
    clean_title: &str,
) -> Option<AppMeta> {
    let url = format!(
        "https://catalog.gog.com/v1/catalog?limit=5&order=desc:score&productType=in:game,pack&query=like:{}",
        urlencoding::encode(clean_title)
    );
    let resp = client
        .get(&url)
        .timeout(Duration::from_secs(4))
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let json: Value = resp.json().await.ok()?;
    let products = json.get("products")?.as_array()?;

    let prod = products.iter().find(|p| {
        let t = match p.get("title").and_then(Value::as_str) {
            Some(title) => title,
            None => return false,
        };
        let devs: Vec<String> = p
            .get("developers")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().or_else(|| v.get("name").and_then(Value::as_str)))
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        let pubs: Vec<String> = p
            .get("publishers")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().or_else(|| v.get("name").and_then(Value::as_str)))
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        titles_match(clean_title, t) && super::developers::validate_metadata_candidate(clean_title, t, &devs, &pubs)
    })?;

    let developers: Vec<String> = prod
        .get("developers")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().or_else(|| v.get("name").and_then(Value::as_str)))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();

    let publishers: Vec<String> = prod
        .get("publishers")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().or_else(|| v.get("name").and_then(Value::as_str)))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();

    let genres: Vec<String> = prod
        .get("genres")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.get("name").and_then(Value::as_str))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();

    let tags: Vec<String> = prod
        .get("tags")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.get("name").and_then(Value::as_str))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();

    let release_date = prod
        .get("releaseDate")
        .and_then(Value::as_str)
        .map(|s| s.replace('.', "-"));
    let release_year = release_date
        .as_deref()
        .and_then(|s| s.split('-').next())
        .and_then(|s| s.parse::<i32>().ok());

    let user_score = prod
        .get("reviewsRating")
        .and_then(Value::as_u64)
        .map(|r| (r as u32).min(100));

    let capsule_image = prod
        .get("coverVertical")
        .and_then(Value::as_str)
        .map(str::to_string);
    let hero_image = prod
        .get("coverHorizontal")
        .and_then(Value::as_str)
        .map(str::to_string);

    let screenshots: Vec<String> = prod
        .get("screenshots")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(Value::as_str)
                .take(10)
                .map(|s| s.replace("_{formatter}", "_ggvgl_2x"))
                .collect()
        })
        .unwrap_or_default();

    Some(AppMeta {
        name: prod
            .get("title")
            .and_then(Value::as_str)
            .map(str::to_string),
        developers,
        publishers,
        genres,
        tags,
        release_date,
        release_year,
        user_score: user_score.map(f64::from),
        header_image: capsule_image.clone(),
        capsule_image,
        hero_image,
        screenshots,
        ..Default::default()
    })
}

// ---------------------------------------------------------------------------
// Wikipedia + Wikidata MediaWiki API (Keyless Universal Encyclopedia)
// ---------------------------------------------------------------------------

async fn fetch_wikipedia_wikidata_metadata(
    client: &reqwest::Client,
    clean_title: &str,
    console: Option<&str>,
) -> Option<AppMeta> {
    let query_str = if let Some(c) = console {
        format!("\"{clean_title}\" {c} video game")
    } else {
        format!("\"{clean_title}\" video game")
    };

    let search_url = format!(
        "https://en.wikipedia.org/w/api.php?action=query&list=search&srsearch={}&srlimit=5&format=json",
        urlencoding::encode(&query_str)
    );
    let search_resp = client
        .get(&search_url)
        .timeout(Duration::from_secs(4))
        .send()
        .await
        .ok()?;
    let search_json: Value = search_resp.json().await.ok()?;
    let search_items = search_json
        .get("query")
        .and_then(|q| q.get("search"))
        .and_then(Value::as_array)?;

    // Pick the first search result whose title matches the game title (stripping " (video game)")
    let matched_page = search_items.iter().find(|item| {
        let raw_t = item.get("title").and_then(Value::as_str).unwrap_or("");
        let lower = raw_t.to_ascii_lowercase();
        if lower.contains("(soundtrack)")
            || lower.contains("(album)")
            || lower.contains("(film)")
            || lower.contains("(movie)")
            || lower.contains("(score)")
            || lower.contains("(discography)")
            || lower.contains("(tv series)")
            || lower.contains("(television series)")
            || lower.contains("(novel)")
            || lower.contains("(book)")
            || lower.contains("(manga)")
            || lower.contains("(anime)")
            || lower.contains("list of ")
            || lower.starts_with("music of ")
        {
            return false;
        }

        let stripped = raw_t
            .split('(')
            .next()
            .unwrap_or(raw_t)
            .trim();
        titles_match(clean_title, stripped)
    })?;

    let page_title = matched_page.get("title")?.as_str()?;

    let details_url = format!(
        "https://en.wikipedia.org/w/api.php?action=query&prop=extracts|pageimages|pageprops&exintro=1&explaintext=1&piprop=original&ppprop=wikibase_item&titles={}&redirects=1&format=json",
        urlencoding::encode(page_title)
    );
    let details_resp = client
        .get(&details_url)
        .timeout(Duration::from_secs(4))
        .send()
        .await
        .ok()?;
    let details_json: Value = details_resp.json().await.ok()?;
    let pages = details_json
        .get("query")
        .and_then(|q| q.get("pages"))
        .and_then(Value::as_object)?;
    let (_, page_obj) = pages.iter().next()?;

    let extract = page_obj
        .get("extract")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(summarize_extract);

    let is_soundtrack_extract = extract.as_deref().map_or(false, |ex| {
        let lower = ex.to_ascii_lowercase();
        lower.contains("is the soundtrack")
            || lower.contains("is the original soundtrack")
            || lower.contains("is an album")
            || lower.contains("soundtrack album")
            || (lower.contains("composed by") && lower.contains("video game"))
            || lower.contains("studio album")
            || lower.contains("score composed by")
    });
    if is_soundtrack_extract {
        return None;
    }

    let wiki_image = page_obj
        .get("original")
        .and_then(|o| o.get("source"))
        .and_then(Value::as_str)
        .filter(|u| !u.ends_with(".svg"))
        .map(str::to_string);

    let wikibase_id = page_obj
        .get("pageprops")
        .and_then(|pp| pp.get("wikibase_item"))
        .and_then(Value::as_str);

    let mut developers = Vec::new();
    let mut publishers = Vec::new();
    let mut genres = Vec::new();
    let mut release_date: Option<String> = None;
    let mut release_year: Option<i32> = None;

    if let Some(qid) = wikibase_id {
        if let Some((devs, pubs, gens, r_date, r_year)) =
            fetch_wikidata_claims(client, qid).await
        {
            developers = devs;
            publishers = pubs;
            genres = gens;
            release_date = r_date;
            release_year = r_year;
        }
    }

    Some(AppMeta {
        short_description: extract,
        developers,
        publishers,
        genres,
        release_date,
        release_year,
        header_image: wiki_image.clone(),
        capsule_image: wiki_image,
        ..Default::default()
    })
}

fn summarize_extract(text: &str) -> String {
    let first_para = text.lines().find(|l| l.trim().len() > 30).unwrap_or(text).trim();
    // Keep up to ~420 chars on sentence boundary
    if first_para.len() <= 420 {
        return first_para.to_string();
    }
    let slice = &first_para[..420];
    if let Some(dot) = slice.rfind(". ") {
        format!("{}.", &slice[..dot])
    } else {
        format!("{}…", slice.trim())
    }
}

async fn fetch_wikidata_claims(
    client: &reqwest::Client,
    qid: &str,
) -> Option<(
    Vec<String>,
    Vec<String>,
    Vec<String>,
    Option<String>,
    Option<i32>,
)> {
    let url = format!(
        "https://www.wikidata.org/w/api.php?action=wbgetentities&ids={}&props=claims&format=json",
        urlencoding::encode(qid)
    );
    let resp = client
        .get(&url)
        .timeout(Duration::from_secs(4))
        .send()
        .await
        .ok()?;
    let json: Value = resp.json().await.ok()?;
    let claims = json
        .get("entities")
        .and_then(|e| e.get(qid))
        .and_then(|ent| ent.get("claims"))?;

    let dev_qids = extract_entity_qids(claims, "P178", 3);
    let pub_qids = extract_entity_qids(claims, "P123", 3);
    let genre_qids = extract_entity_qids(claims, "P136", 4);

    // P577: publication date (+YYYY-MM-DDT00:00:00Z)
    let mut release_date: Option<String> = None;
    let mut release_year: Option<i32> = None;
    if let Some(p577) = claims.get("P577").and_then(Value::as_array) {
        for stmt in p577 {
            if let Some(time_str) = stmt
                .get("mainsnak")
                .and_then(|m| m.get("datavalue"))
                .and_then(|d| d.get("value"))
                .and_then(|v| v.get("time"))
                .and_then(Value::as_str)
            {
                let clean = time_str.trim_start_matches('+');
                if let Some(date_part) = clean.split('T').next() {
                    let year = date_part
                        .split('-')
                        .next()
                        .and_then(|y| y.parse::<i32>().ok());
                    if release_year.is_none() || year < release_year {
                        release_year = year;
                        release_date = Some(date_part.replace("-00", ""));
                    }
                }
            }
        }
    }

    // Batch resolve all Q-IDs to English labels in one request
    let mut all_qids: HashSet<String> = HashSet::new();
    for id in dev_qids.iter().chain(pub_qids.iter()).chain(genre_qids.iter()) {
        all_qids.insert(id.clone());
    }

    if all_qids.is_empty() {
        return Some((Vec::new(), Vec::new(), Vec::new(), release_date, release_year));
    }

    let ids_joined: Vec<String> = all_qids.into_iter().collect();
    let labels_url = format!(
        "https://www.wikidata.org/w/api.php?action=wbgetentities&ids={}&props=labels&languages=en&format=json",
        ids_joined.join("|")
    );
    let labels_resp = client
        .get(&labels_url)
        .timeout(Duration::from_secs(4))
        .send()
        .await
        .ok()?;
    let labels_json: Value = labels_resp.json().await.ok()?;
    let entities = labels_json.get("entities")?;

    let resolve_list = |qids: &[String]| -> Vec<String> {
        qids.iter()
            .filter_map(|id| {
                entities
                    .get(id)
                    .and_then(|e| e.get("labels"))
                    .and_then(|l| l.get("en"))
                    .and_then(|en| en.get("value"))
                    .and_then(Value::as_str)
                    .map(|s| {
                        // Capitalize first letter nicely for genres
                        let mut c = s.chars();
                        match c.next() {
                            None => String::new(),
                            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                        }
                    })
            })
            .collect()
    };

    Some((
        resolve_list(&dev_qids),
        resolve_list(&pub_qids),
        resolve_list(&genre_qids),
        release_date,
        release_year,
    ))
}

fn extract_entity_qids(claims: &Value, prop: &str, max: usize) -> Vec<String> {
    claims
        .get(prop)
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|stmt| {
                    stmt.get("mainsnak")
                        .and_then(|m| m.get("datavalue"))
                        .and_then(|d| d.get("value"))
                        .and_then(|v| v.get("id"))
                        .and_then(Value::as_str)
                        .map(str::to_string)
                })
                .take(max)
                .collect()
        })
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// PCGamingWiki Cargo API (Keyless)
// ---------------------------------------------------------------------------

async fn fetch_pcgamingwiki_metadata(
    client: &reqwest::Client,
    clean_title: &str,
) -> Option<AppMeta> {
    let escaped = clean_title.replace('"', "");
    let where_clause = format!("Infobox_game._pageName=\"{escaped}\"");
    let url = format!(
        "https://www.pcgamingwiki.com/w/api.php?action=cargoquery&tables=Infobox_game&fields=Infobox_game._pageName=Page,Infobox_game.Developers,Infobox_game.Publishers,Infobox_game.Released,Infobox_game.Genres&where={}&limit=1&format=json",
        urlencoding::encode(&where_clause)
    );
    let resp = client
        .get(&url)
        .timeout(Duration::from_secs(4))
        .send()
        .await
        .ok()?;
    let json: Value = resp.json().await.ok()?;
    let row = json
        .get("cargoquery")
        .and_then(Value::as_array)
        .and_then(|arr| arr.first())
        .and_then(|item| item.get("title"))?;

    let split_csv = |field: &str| -> Vec<String> {
        row.get(field)
            .and_then(Value::as_str)
            .map(|s| {
                s.split(',')
                    .map(|p| p.trim().to_string())
                    .filter(|p| !p.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    };

    let developers = split_csv("Developers");
    let publishers = split_csv("Publishers");
    let genres = split_csv("Genres");
    let page = row.get("Page").and_then(Value::as_str).unwrap_or(clean_title);

    if !titles_match(clean_title, page)
        || !super::developers::validate_metadata_candidate(clean_title, page, &developers, &publishers)
    {
        return None;
    }

    if developers.is_empty() && publishers.is_empty() && genres.is_empty() {
        return None;
    }

    Some(AppMeta {
        developers,
        publishers,
        genres,
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_soundtrack_rejection() {
        assert!(!titles_match(
            "FINAL FANTASY XIV - A Realm Reborn",
            "FINAL FANTASY XIV: A Realm Reborn Original Soundtrack"
        ));
        assert!(!titles_match(
            "FINAL FANTASY XIV",
            "FINAL FANTASY XIV Original Soundtrack"
        ));
        assert!(!titles_match(
            "Halo 3",
            "Halo 3 Original Soundtrack"
        ));
        assert!(!titles_match(
            "Persona 5",
            "Persona 5 Official Art Book"
        ));
        assert!(!titles_match(
            "Cyberpunk 2077",
            "Cyberpunk 2077: Phantom Liberty DLC"
        ));
    }

    #[test]
    fn test_sequel_rejection() {
        assert!(!titles_match("Super Mario Galaxy", "Super Mario Galaxy 2"));
        assert!(!titles_match("Half-Life", "Half-Life 2"));
    }

    #[test]
    fn test_benign_edition_matching() {
        assert!(titles_match(
            "The Witcher 3: Wild Hunt",
            "The Witcher 3: Wild Hunt - Complete Edition"
        ));
        assert!(titles_match("Relicts of Aeson", "Relicts of Aeson"));
        assert!(titles_match("My Futa Family", "My Futa Family"));
    }
}
