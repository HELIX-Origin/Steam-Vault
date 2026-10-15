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

impl AutoCatKind {
    /// Whether this kind needs per-app store metadata to produce values.
    ///
    /// The other kinds read straight off the [`Game`] record (platform, installed
    /// state, playtime, name), so they work without any network access.
    pub fn needs_metadata(self) -> bool {
        matches!(
            self,
            AutoCatKind::Genre
                | AutoCatKind::Developer
                | AutoCatKind::Publisher
                | AutoCatKind::Flag
                | AutoCatKind::ReleaseYear
                | AutoCatKind::UserScore
        )
    }

    /// Whether this kind needs Steam community tags (a separate endpoint).
    pub fn needs_tags(self) -> bool {
        matches!(self, AutoCatKind::Tag)
    }
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

/// Split multi-entity strings (e.g. comma/semicolon/slash separated developers
/// or publishers) into individual distinct names, preserving legal suffixes.
pub fn split_entities(raw: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for item in raw {
        // Protect legal company suffixes from comma splitting
        let protected = item
            .replace("Co., Ltd.", "Co._Ltd_")
            .replace("Co.,Ltd.", "Co._Ltd_")
            .replace("co., ltd.", "co._ltd_")
            .replace("Corp., Ltd.", "Corp._Ltd_");

        for part in protected.split(&[';', ','][..]) {
            for sub in part.split(" / ") {
                let restored = sub
                    .replace("Co._Ltd_", "Co., Ltd.")
                    .replace("co._ltd_", "co., ltd.")
                    .replace("Corp._Ltd_", "Corp., Ltd.")
                    .trim()
                    .to_string();
                if !restored.is_empty() && !out.contains(&restored) {
                    out.push(restored);
                }
            }
        }
    }
    out
}

/// Compute the category names an [`AutoCat`] assigns to a game.
pub fn apply(autocat: &AutoCat, game: &Game, meta: &AppMeta) -> Vec<String> {
    if !autocat.enabled {
        return Vec::new();
    }

    let values: Vec<String> = match autocat.kind {
        AutoCatKind::Genre => split_entities(&meta.genres),
        AutoCatKind::Developer => {
            let devs = split_entities(&meta.developers);
            if devs.is_empty() {
                let pubs = split_entities(&meta.publishers);
                if !pubs.is_empty() {
                    pubs
                } else if game.platform != crate::platforms::Platform::Steam
                    && game.platform != crate::platforms::Platform::Other
                {
                    vec![game.platform.label().to_string()]
                } else {
                    Vec::new()
                }
            } else {
                devs
            }
        }
        AutoCatKind::Publisher => {
            let pubs = split_entities(&meta.publishers);
            if pubs.is_empty() {
                let devs = split_entities(&meta.developers);
                if !devs.is_empty() {
                    devs
                } else if game.platform != crate::platforms::Platform::Steam
                    && game.platform != crate::platforms::Platform::Other
                {
                    vec![game.platform.label().to_string()]
                } else {
                    Vec::new()
                }
            } else {
                pubs
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platforms::Platform;
    use crate::steam::models::{AppMeta, Game, GameKind};

    #[test]
    fn splits_multi_entity_strings_preserving_company_suffixes() {
        let raw = vec![
            "Square Enix, PlatinumGames".to_string(),
            "Spike Chunsoft Co., Ltd.".to_string(),
            "Valve / Hidden Path Entertainment".to_string(),
            "Kojima Productions; Konami".to_string(),
        ];
        let split = split_entities(&raw);
        assert_eq!(
            split,
            vec![
                "Square Enix",
                "PlatinumGames",
                "Spike Chunsoft Co., Ltd.",
                "Valve",
                "Hidden Path Entertainment",
                "Kojima Productions",
                "Konami"
            ]
        );
    }

    #[test]
    fn falls_back_to_platform_publisher_when_metadata_absent() {
        let autocat = AutoCat {
            id: "pub".into(),
            name: "Publisher".into(),
            kind: AutoCatKind::Publisher,
            enabled: true,
            prefix: None,
            max_categories: None,
            remove_existing: false,
            include: Vec::new(),
            exclude: Vec::new(),
        };
        let game = Game {
            app_id: 12345,
            name: "Dead Space".into(),
            kind: GameKind::Shortcut,
            platform: Platform::Ea,
            ..Default::default()
        };
        let meta = AppMeta::default();
        let cats = apply(&autocat, &game, &meta);
        assert_eq!(cats, vec!["EA"]);
    }
}
