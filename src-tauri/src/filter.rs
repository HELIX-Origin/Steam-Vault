//! Advanced library filtering (Depressurizer "Allowed / Required / Excluded").
//!
//! - `allowed`: game must be in **at least one** listed collection (OR).
//! - `required`: game must be in **all** listed collections (AND).
//! - `excluded`: game must be in **none** of the listed collections (NOT).
//! - `search`: case-insensitive substring match on the game name.
//!
//! Empty groups are ignored.

use serde::{Deserialize, Serialize};

use crate::steam::models::{Game, GameKind};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Filter {
    #[serde(default)]
    pub allowed: Vec<String>,
    #[serde(default)]
    pub required: Vec<String>,
    #[serde(default)]
    pub excluded: Vec<String>,
    pub search: Option<String>,
    pub kind: Option<GameKind>,
}

impl Filter {
    pub fn is_empty(&self) -> bool {
        self.allowed.is_empty()
            && self.required.is_empty()
            && self.excluded.is_empty()
            && self.search.as_deref().unwrap_or("").is_empty()
            && self.kind.is_none()
    }
}

/// Test whether a game satisfies a filter.
pub fn matches(filter: &Filter, game: &Game) -> bool {
    if let Some(kind) = filter.kind {
        if game.kind != kind {
            return false;
        }
    }

    if let Some(search) = filter.search.as_deref() {
        if !search.is_empty() && !game.name.to_lowercase().contains(&search.to_lowercase()) {
            return false;
        }
    }

    let has = |name: &String| {
        game.collections
            .iter()
            .any(|c| c.eq_ignore_ascii_case(name))
    };

    if !filter.allowed.is_empty() && !filter.allowed.iter().any(has) {
        return false;
    }
    if !filter.required.is_empty() && !filter.required.iter().all(has) {
        return false;
    }
    if !filter.excluded.is_empty() && filter.excluded.iter().any(has) {
        return false;
    }
    true
}
