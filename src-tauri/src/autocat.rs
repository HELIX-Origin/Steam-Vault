//! AutoCats: Depressurizer-style automatic category generation.
//!
//! An [`AutoCat`] turns one property of a game or its [`AppMeta`] (genre,
//! developer, publisher, Steam feature flag, tag, release year, user score,
//! source platform, installed state, playtime, or name) into category names to
//! assign. Rules are declarative and stored in settings.
//!
//! When a user runs "Create collections" they pick one kind — the generated
//! collection names are the values of that kind (a game's genre becomes the
//! "Action" collection, its developer becomes the "Valve" collection, and so
//! on).

use serde::{Deserialize, Serialize};

use crate::steam::models::{AppMeta, Game};

/// The field an [`AutoCat`] derives categories from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AutoCatKind {
    Genre,
    Developer,
    Publisher,
    Flag,
    Tag,
    ReleaseYear,
    UserScore,
    /// Source storefront (works for imported non-Steam games too).
    Platform,
    /// Installed vs. not installed.
    InstalledState,
    /// Playtime buckets (Unplayed, <2h, 2–10h, 10–50h, 50h+).
    Playtime,
    /// First letter of the name (A–Z, or `#`).
    Name,
}

/// A single automatic categorization rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoCat {
    pub id: String,
    pub name: String,
    pub kind: AutoCatKind,
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Text prepended to each generated category name.
    pub prefix: Option<String>,
    /// Maximum categories this rule may assign per game.
    pub max_categories: Option<usize>,
    /// Whether to record the underlying values as generated (for re-runs).
    #[serde(default)]
    pub remove_existing: bool,
    /// Only values in this list are used (empty = all).
    #[serde(default)]
    pub include: Vec<String>,
    /// Values in this list are always skipped.
    #[serde(default)]
    pub exclude: Vec<String>,
}

fn default_true() -> bool {
    true
}

/// Compute the category names an [`AutoCat`] assigns to a game.
pub fn apply(autocat: &AutoCat, game: &Game, meta: &AppMeta) -> Vec<String> {
    if !autocat.enabled {
        return Vec::new();
    }

    let values: Vec<String> = match autocat.kind {
        AutoCatKind::Genre => meta.genres.clone(),
        AutoCatKind::Developer => meta.developers.clone(),
        AutoCatKind::Publisher => meta.publishers.clone(),
        AutoCatKind::Flag => meta.categories.clone(),
        AutoCatKind::Tag => meta.tags.clone(),
        AutoCatKind::ReleaseYear => match meta.release_year {
            Some(year) => vec![(year / 10 * 10).to_string(), year.to_string()],
            None => Vec::new(),
        },
        AutoCatKind::UserScore => match meta.user_score {
            Some(score) => vec![score_bucket(score)],
            None => Vec::new(),
        },
        AutoCatKind::Platform => vec![game.platform.label().to_string()],
        AutoCatKind::InstalledState => vec![if game.installed {
            "Installed".to_string()
        } else {
            "Not installed".to_string()
        }],
        AutoCatKind::Playtime => vec![playtime_bucket(game.playtime_minutes)],
        AutoCatKind::Name => vec![first_letter(&game.name)],
    };

    let include: Vec<String> = autocat.include.iter().map(|s| s.to_lowercase()).collect();
    let exclude: Vec<String> = autocat.exclude.iter().map(|s| s.to_lowercase()).collect();

    let mut out: Vec<String> = Vec::new();
    for value in values {
        let lower = value.to_lowercase();
        if !include.is_empty() && !include.contains(&lower) {
            continue;
        }
        if exclude.contains(&lower) {
            continue;
        }
        let name = match &autocat.prefix {
            Some(prefix) if !prefix.is_empty() => format!("{prefix}{value}"),
            _ => value.clone(),
        };
        if !out.contains(&name) {
            out.push(name);
        }
    }

    if let Some(max) = autocat.max_categories {
        out.truncate(max);
    }
    out
}

fn score_bucket(score: f64) -> String {
    if score >= 90.0 {
        "Score: 90+".into()
    } else if score >= 80.0 {
        "Score: 80-89".into()
    } else if score >= 70.0 {
        "Score: 70-79".into()
    } else if score >= 60.0 {
        "Score: 60-69".into()
    } else {
        "Score: <60".into()
    }
}

fn playtime_bucket(minutes: Option<u64>) -> String {
    match minutes.unwrap_or(0) {
        0 => "Playtime: Unplayed".into(),
        m if m < 120 => "Playtime: <2h".into(),
        m if m < 600 => "Playtime: 2-10h".into(),
        m if m < 3000 => "Playtime: 10-50h".into(),
        _ => "Playtime: 50h+".into(),
    }
}

fn first_letter(name: &str) -> String {
    match name.trim().chars().next() {
        Some(ch) if ch.is_ascii_alphabetic() => ch.to_ascii_uppercase().to_string(),
        Some(_) | None => "#".into(),
    }
}
