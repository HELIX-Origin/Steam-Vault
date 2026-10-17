//! Library storage management: inspect, register, and relocate Steam library
//! folders and the games inside them.
//!
//! Steam spreads installed games across one or more *library folders*. Each
//! library folder is a directory containing a `steamapps` subdirectory, and the
//! set of libraries is recorded in `libraryfolders.vdf`:
//!
//! * the primary copy at `<root>/steamapps/libraryfolders.vdf`;
//! * a legacy copy at `<root>/config/libraryfolders.vdf`;
//! * a per-library copy at `<library>/steamapps/libraryfolders.vdf`.
//!
//! Because Steam rewrites every copy it finds on startup, moving content
//! between drives **must** update all of them — otherwise Steam "loses" the
//! games and re-downloads or overwrites the change. [`write_library_folders`]
//! handles that, backing each file up first via [`crate::backup`].
//!
//! Every mutating entry point requires Steam to be closed; the command layer
//! enforces that via [`crate::steam::process::is_steam_running`].

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::error::{AppError, AppResult};
use crate::platforms::Platform;
use crate::steam::models::SteamLibrary;
use crate::steam::{paths, vdf};

/// `StateFlags` bit indicating a fully installed app.
const STATE_FLAG_FULLY_INSTALLED: u32 = 4;

/// Extra headroom required on the destination drive before a move starts.
const FREE_SPACE_SLACK: u64 = 512 * 1024 * 1024;

/// Files at or above this size are copied with the OS fast path (coarser
/// progress) instead of a streamed, byte-counted copy.
const FAST_COPY_THRESHOLD: u64 = 32 * 1024 * 1024;

/// Emit progress at most once per this many bytes, to avoid flooding the IPC.
const PROGRESS_INTERVAL: u64 = 16 * 1024 * 1024;

// ---------------------------------------------------------------------------
// Types surfaced to the frontend
// ---------------------------------------------------------------------------

/// One registered library folder with its size and drive capacity.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryEntry {
    pub index: u32,
    pub path: String,
    pub label: Option<String>,
    /// Whether the folder currently exists on disk.
    pub exists: bool,
    /// True for the library that hosts the Steam installation itself.
    pub is_steam_root: bool,
    /// Installed games recorded in this library.
    pub game_count: usize,
    /// Sum of the games' `SizeOnDisk` values.
    pub total_bytes: u64,
    pub disk_total: Option<u64>,
    pub disk_free: Option<u64>,
}

/// An installed Steam app or external game shortcut and the library folder that holds it.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GamePlacement {
    pub app_id: u32,
    pub name: String,
    pub library_path: String,
    pub install_dir: Option<String>,
    pub size_on_disk: u64,
    pub installed: bool,
    /// Whether the `common/<installdir>` or game directory folder is actually present.
    pub folder_exists: bool,
    pub platform: Option<Platform>,
}

/// A mounted disk, for the drive overview.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskInfo {
    pub name: String,
    pub mount: String,
    pub total: u64,
    pub free: u64,
    pub removable: bool,
}

/// Everything the Storage view needs in one round-trip.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageReport {
    pub root: String,
    pub libraries: Vec<LibraryEntry>,
    pub games: Vec<GamePlacement>,
    pub disks: Vec<DiskInfo>,
}

/// A dry-run summary of what a move would do.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MovePreview {
    pub from: String,
    pub to: String,
    pub bytes: u64,
    pub target_free: Option<u64>,
    /// Whether the destination has room for the content plus [`FREE_SPACE_SLACK`].
    pub fits: bool,
    /// True when the move is on one volume and will therefore be instant.
    pub same_volume: bool,
    pub warnings: Vec<String>,
}

/// The result of a completed move.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveOutcome {
    pub from: String,
    pub to: String,
    pub bytes: u64,
    pub files: usize,
    pub same_volume: bool,
    /// `libraryfolders.vdf` files that were updated.
    pub vdf_updated: Vec<String>,
}

// ---------------------------------------------------------------------------
// Path & volume helpers
// ---------------------------------------------------------------------------

/// Normalize a path for comparison: lowercase on Windows, trim trailing
/// separators, and unify `/` vs `\`.
pub fn normalize_path(path: &str) -> String {
    let mut value = path.trim().trim_matches('"').to_string();
    if cfg!(windows) {
        value = value.replace('/', "\\");
    }
    while value.len() > 3 && (value.ends_with('\\') || value.ends_with('/')) {
        value.pop();
    }
    if cfg!(windows) {
        value = value.to_lowercase();
    }
    value
}

fn paths_equal(a: &str, b: &str) -> bool {
    normalize_path(a) == normalize_path(b)
}

/// The volume prefix for the deepest existing ancestor of `path` (`C:\`, `/`).
fn volume_root(path: &Path) -> Option<PathBuf> {
    let mut current = path.to_path_buf();
    loop {
        if current.exists() {
            break;
        }
        current = current.parent()?.to_path_buf();
    }
    current.components().next().and_then(|part| {
        let value = part.as_os_str().to_string_lossy().to_lowercase();
        if value.is_empty() {
            None
        } else {
            Some(PathBuf::from(value))
        }
    })
}

/// Whether two paths live on the same volume (so a move is a cheap rename).
pub fn same_volume(a: &Path, b: &Path) -> bool {
    match (volume_root(a), volume_root(b)) {
        (Some(left), Some(right)) => left == right,
        _ => false,
    }
}

/// Reject a destination that would nest inside the source or vice versa.
fn is_inside(child: &Path, parent: &Path) -> bool {
    let child = normalize_path(&child.display().to_string());
    let parent = normalize_path(&parent.display().to_string());
    let parent = parent.trim_end_matches('\\');
    child.starts_with(&format!("{parent}\\"))
}

// ---------------------------------------------------------------------------
// Disks
// ---------------------------------------------------------------------------

fn is_cloud_disk(name: &str, mount: &str) -> bool {
    let n = name.to_lowercase();
    let m = mount.to_lowercase();
    n.contains("google drive")
        || n.contains("onedrive")
        || n.contains("dropbox")
        || n.contains("icloud")
        || n.contains("proton drive")
        || n.contains("pcloud")
        || n.contains("box sync")
        || n.contains("nextcloud")
        || n.contains("owncloud")
        || n.contains("mega")
        || m.contains("google drive")
        || m.contains("onedrive")
        || m.contains("dropbox")
        || m.contains("icloud")
}

fn disks_snapshot() -> Vec<DiskInfo> {
    let disks = sysinfo::Disks::new_with_refreshed_list();
    let mut out: Vec<DiskInfo> = disks
        .iter()
        .filter(|d| !is_cloud_disk(&d.name().to_string_lossy(), &d.mount_point().to_string_lossy()))
        .map(|disk| DiskInfo {
            name: disk.name().to_string_lossy().to_string(),
            mount: disk.mount_point().display().to_string(),
            total: disk.total_space(),
            free: disk.available_space(),
            removable: disk.is_removable(),
        })
        .collect();
    out.sort_by(|a, b| a.mount.cmp(&b.mount));
    out
}

/// Free / total space for the volume that holds `path` (longest mount match).
fn disk_space_for(path: &Path, disks: &[DiskInfo]) -> Option<(u64, u64)> {
    let target = normalize_path(&path.display().to_string());
    let mut best: Option<(&DiskInfo, usize)> = None;
    for disk in disks {
        let mount = normalize_path(&disk.mount);
        if mount.is_empty() {
            continue;
        }
        let matches = target == mount
            || target.starts_with(&mount) && (mount.ends_with('\\') || mount.ends_with('/'))
            || target.starts_with(&format!("{mount}\\"))
            || target.starts_with(&format!("{mount}/"));
        if !matches {
            continue;
        }
        let len = mount.len();
        if best.map(|(_, best_len)| len > best_len).unwrap_or(true) {
            best = Some((disk, len));
        }
    }
    best.map(|(disk, _)| (disk.total, disk.free))
}

// ---------------------------------------------------------------------------
// Manifests
// ---------------------------------------------------------------------------

/// The subset of `appmanifest_*.acf` that storage management needs.
#[derive(Debug, Clone)]
struct Manifest {
    app_id: u32,
    name: String,
    install_dir: String,
    size_on_disk: u64,
    state_flags: u32,
}

fn appmanifest_id(file_name: &str) -> Option<u32> {
    file_name
        .strip_prefix("appmanifest_")?
        .strip_suffix(".acf")?
        .parse()
        .ok()
}

fn read_manifest(path: &Path) -> Option<Manifest> {
    let app_id = appmanifest_id(&path.file_name()?.to_string_lossy())?;
    let text = std::fs::read_to_string(path).ok()?;
    let doc = vdf::parse(&text).ok()?;
    let state = doc.get("AppState")?;
    Some(Manifest {
        app_id,
        name: state.get_str("name").unwrap_or_default().to_string(),
        install_dir: state.get_str("installdir").unwrap_or_default().to_string(),
        size_on_disk: state.get_u64("SizeOnDisk").unwrap_or(0),
        state_flags: state.get_u32("StateFlags").unwrap_or(0),
    })
}

/// Every manifest in a library's `steamapps` folder.
fn manifests_in(library: &str) -> Vec<Manifest> {
    let steamapps = Path::new(library).join("steamapps");
    let Ok(entries) = std::fs::read_dir(&steamapps) else {
        return Vec::new();
    };
    let mut out: Vec<Manifest> = entries
        .flatten()
        .filter(|entry| entry.path().is_file())
        .filter_map(|entry| read_manifest(&entry.path()))
        .collect();
    out.sort_by_key(|a| a.name.to_lowercase());
    out
}

fn is_installed(manifest: &Manifest) -> bool {
    manifest.state_flags & STATE_FLAG_FULLY_INSTALLED != 0
}

fn common_dir(library: &str, install_dir: &str) -> PathBuf {
    Path::new(library)
        .join("steamapps")
        .join("common")
        .join(install_dir)
}

// ---------------------------------------------------------------------------
// Reporting
// ---------------------------------------------------------------------------

/// Recursively compute the size of a directory on disk up to `max_depth`.
fn dir_size(path: &Path, max_depth: usize) -> u64 {
    if !path.is_dir() {
        if path.is_file() {
            return path.metadata().map(|m| m.len()).unwrap_or(0);
        }
        return 0;
    }
    let mut total = 0u64;
    let mut stack = vec![(path.to_path_buf(), 0usize)];
    let mut visited_dirs = 0usize;

    while let Some((dir, depth)) = stack.pop() {
        if visited_dirs > 30_000 || depth > max_depth {
            continue;
        }
        visited_dirs += 1;
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    total = total.saturating_add(meta.len());
                } else if meta.is_dir() && !meta.file_type().is_symlink() && depth < max_depth {
                    stack.push((p, depth + 1));
                }
            }
        }
    }
    total
}

/// Build the full storage report for a Steam root, including both Steam games
/// and non-Steam game shortcuts with their disk footprints and platforms.
pub fn report(root: &Path, _account: Option<&str>) -> AppResult<StorageReport> {
    let root_str = root.display().to_string();
    let disks = disks_snapshot();
    let libraries = paths::read_library_folders(root);

    let mut entries = Vec::new();
    let mut games = Vec::new();

    // 1. Steam games from registered library folders
    for library in &libraries {
        let manifests = manifests_in(&library.path);
        let total_bytes: u64 = manifests.iter().map(|m| m.size_on_disk).sum();
        let (disk_total, disk_free) = match disk_space_for(Path::new(&library.path), &disks) {
            Some((total, free)) => (Some(total), Some(free)),
            None => (None, None),
        };

        for manifest in &manifests {
            let folder = common_dir(&library.path, &manifest.install_dir);
            games.push(GamePlacement {
                app_id: manifest.app_id,
                name: manifest.name.clone(),
                library_path: library.path.clone(),
                install_dir: Some(manifest.install_dir.clone()),
                size_on_disk: manifest.size_on_disk,
                installed: is_installed(manifest),
                folder_exists: !manifest.install_dir.is_empty() && folder.is_dir(),
                platform: Some(Platform::Steam),
            });
        }

        entries.push(LibraryEntry {
            index: library.index,
            path: library.path.clone(),
            label: library.label.clone(),
            exists: Path::new(&library.path).is_dir(),
            is_steam_root: paths_equal(&library.path, &root_str),
            game_count: manifests.len(),
            total_bytes,
            disk_total,
            disk_free,
        });
    }

    // Steam storage only tracks Steam games; non-Steam/external games are tracked by platform providers

    games.sort_by_key(|a| std::cmp::Reverse(a.size_on_disk));

    Ok(StorageReport {
        root: root_str,
        libraries: entries,
        games,
        disks,
    })
}

// ---------------------------------------------------------------------------
// libraryfolders.vdf read / write
// ---------------------------------------------------------------------------

/// Load the primary `libraryfolders.vdf` as a mutable document.
fn load_library_folders(root: &Path) -> AppResult<vdf::Vdf> {
    let file = paths::library_folders_file(root).ok_or_else(|| {
        AppError::NotFound("No libraryfolders.vdf found for this Steam installation.".into())
    })?;
    let text = std::fs::read_to_string(&file)?;
    let doc = vdf::parse(&text)?;
    if doc.get("libraryfolders").is_none() {
        return Err(AppError::Vdf(
            "libraryfolders.vdf has no 'libraryfolders' section".into(),
        ));
    }
    Ok(doc)
}

fn libraries_map_mut(doc: &mut vdf::Vdf) -> AppResult<&mut BTreeMap<String, vdf::Vdf>> {
    doc.get_mut("libraryfolders")
        .and_then(vdf::Vdf::as_object_mut)
        .ok_or_else(|| AppError::Vdf("libraryfolders section is not an object".into()))
}

/// Every on-disk `libraryfolders.vdf` Steam maintains for this installation.
///
/// Steam rewrites each copy it discovers, so all of them must agree on the new
/// paths or a move is undone the next time Steam starts.
fn library_folders_targets(root: &Path, libraries: &[SteamLibrary]) -> Vec<PathBuf> {
    let mut targets = vec![root.join("steamapps").join("libraryfolders.vdf")];

    let legacy = root.join("config").join("libraryfolders.vdf");
    if legacy.is_file() {
        targets.push(legacy);
    }

    for library in libraries {
        let path = library.path.trim();
        if path.is_empty() {
            continue;
        }
        let candidate = Path::new(path).join("steamapps").join("libraryfolders.vdf");
        if !targets.iter().any(|existing| {
            paths_equal(
                &existing.display().to_string(),
                &candidate.display().to_string(),
            )
        }) {
            targets.push(candidate);
        }
    }
    targets
}

/// Write an updated `libraryfolders.vdf` to every copy Steam keeps.
fn write_library_folders(
    root: &Path,
    doc: &vdf::Vdf,
    libraries: &[SteamLibrary],
) -> AppResult<Vec<String>> {
    let text = vdf::to_string(doc);
    let mut written = Vec::new();
    for target in library_folders_targets(root, libraries) {
        crate::backup::backup_file(&target)?;
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&target, &text)?;
        written.push(target.display().to_string());
    }
    Ok(written)
}

/// Rebuild the read model from a document after it has been mutated.
fn libraries_from_doc(doc: &vdf::Vdf) -> Vec<SteamLibrary> {
    let Some(map) = doc.get("libraryfolders").and_then(vdf::Vdf::as_object) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (key, value) in map {
        let Ok(index) = key.parse::<u32>() else {
            continue;
        };
        let path = value.get_str("path").unwrap_or_default().to_string();
        let label = value.get_str("label").map(str::to_string);
        let mut app_ids = Vec::new();
        if let Some(vdf::Vdf::Object(apps)) = value.get("apps") {
            app_ids.extend(apps.keys().filter_map(|id| id.parse::<u32>().ok()));
        }
        app_ids.sort_unstable();
        out.push(SteamLibrary {
            index,
            path,
            label,
            app_ids,
        });
    }
    out.sort_by_key(|library| library.index);
    out
}

// ---------------------------------------------------------------------------
// File operations
// ---------------------------------------------------------------------------

/// Emits throttled `storage-progress` events during a long copy.
struct Progress<'a> {
    app: Option<&'a tauri::AppHandle>,
    label: String,
    done: u64,
    total: u64,
    last_emit: u64,
    files: usize,
}

impl<'a> Progress<'a> {
    fn new(app: Option<&'a tauri::AppHandle>, label: impl Into<String>, total: u64) -> Self {
        Progress {
            app,
            label: label.into(),
            done: 0,
            total,
            last_emit: 0,
            files: 0,
        }
    }

    fn add(&mut self, bytes: u64) {
        self.done += bytes;
        if self.done.saturating_sub(self.last_emit) >= PROGRESS_INTERVAL {
            self.emit();
        }
    }

    fn emit(&mut self) {
        self.last_emit = self.done;
        let Some(app) = self.app else { return };
        use tauri::Emitter;
        let _ = app.emit(
            "storage-progress",
            serde_json::json!({
                "label": self.label,
                "done": self.done,
                "total": self.total,
                "files": self.files,
            }),
        );
    }

    fn finish(&mut self) {
        self.done = self.total.max(self.done);
        self.emit();
    }
}

fn copy_file(src: &Path, dst: &Path, progress: &mut Progress<'_>) -> AppResult<()> {
    use std::io::{Read, Write};

    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let len = std::fs::metadata(src).map(|meta| meta.len()).unwrap_or(0);

    if len >= FAST_COPY_THRESHOLD {
        std::fs::copy(src, dst)?;
        progress.add(len);
        progress.files += 1;
        return Ok(());
    }

    let mut reader = std::io::BufReader::with_capacity(1 << 20, std::fs::File::open(src)?);
    let mut writer = std::io::BufWriter::with_capacity(1 << 20, std::fs::File::create(dst)?);
    let mut buffer = vec![0u8; 1 << 20];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        writer.write_all(&buffer[..read])?;
        progress.add(read as u64);
    }
    writer.flush()?;
    progress.files += 1;

    // Preserve permission bits (notably the executable bit on Linux).
    if let Ok(meta) = std::fs::metadata(src) {
        let _ = std::fs::set_permissions(dst, meta.permissions());
    }
    Ok(())
}

fn copy_recursive(src: &Path, dst: &Path, progress: &mut Progress<'_>) -> AppResult<()> {
    let meta = std::fs::symlink_metadata(src)?;
    if meta.is_dir() {
        std::fs::create_dir_all(dst)?;
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            copy_recursive(&entry.path(), &dst.join(entry.file_name()), progress)?;
        }
        Ok(())
    } else {
        copy_file(src, dst, progress)
    }
}

/// Move a file or directory, preferring an instant same-volume rename.
fn move_path(src: &Path, dst: &Path, progress: &mut Progress<'_>) -> AppResult<bool> {
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if std::fs::rename(src, dst).is_ok() {
        progress.files += 1;
        return Ok(true);
    }
    copy_recursive(src, dst, progress)?;
    std::fs::remove_dir_all(src)
        .or_else(|_| std::fs::remove_file(src))
        .map_err(AppError::Io)?;
    Ok(false)
}

// ---------------------------------------------------------------------------
// Validation helpers
// ---------------------------------------------------------------------------

fn find_library(root: &Path, path: &str) -> AppResult<Vec<SteamLibrary>> {
    let libraries = paths::read_library_folders(root);
    if !libraries
        .iter()
        .any(|library| paths_equal(&library.path, path))
    {
        return Err(AppError::Invalid(format!(
            "'{path}' is not a registered Steam library folder."
        )));
    }
    Ok(libraries)
}

fn target_free_space(to: &Path) -> Option<u64> {
    let disks = disks_snapshot();
    disk_space_for(to, &disks).map(|(_, free)| free)
}

// ---------------------------------------------------------------------------
// Operations
// ---------------------------------------------------------------------------

/// Pre-flight for moving a whole library folder to a new location.
pub fn preview_move_library(root: &Path, from: &str, to: &str) -> AppResult<MovePreview> {
    find_library(root, from)?;
    let source = PathBuf::from(from);
    let target = PathBuf::from(to);

    let mut warnings = Vec::new();
    if paths_equal(&root.display().to_string(), from) {
        return Err(AppError::Invalid(
            "The Steam installation's own folder cannot be moved; move Steam itself instead."
                .into(),
        ));
    }
    if !source.is_dir() {
        return Err(AppError::NotFound(format!(
            "'{from}' does not exist on disk."
        )));
    }
    if target.exists()
        && std::fs::read_dir(&target)
            .map(|mut dir| dir.next().is_some())
            .unwrap_or(false)
    {
        return Err(AppError::Invalid(format!(
            "'{to}' already exists and is not empty."
        )));
    }
    if is_inside(&target, &source) || is_inside(&source, &target) {
        return Err(AppError::Invalid(
            "The destination cannot be inside the source (or vice versa).".into(),
        ));
    }

    let bytes: u64 = manifests_in(from).iter().map(|m| m.size_on_disk).sum();
    let free = target_free_space(&target);
    let fits = free
        .map(|free| free >= bytes + FREE_SPACE_SLACK)
        .unwrap_or(false);
    if !fits {
        if let Some(free) = free {
            warnings.push(format!(
                "Destination has {} free; about {} is required.",
                human_bytes(free),
                human_bytes(bytes + FREE_SPACE_SLACK)
            ));
        }
    }

    Ok(MovePreview {
        from: from.to_string(),
        to: to.to_string(),
        bytes,
        target_free: free,
        fits,
        same_volume: same_volume(&source, &target),
        warnings,
    })
}

/// Move an entire library folder to a new path and update every
/// `libraryfolders.vdf` copy.
pub fn move_library(
    root: &Path,
    from: &str,
    to: &str,
    app: Option<&tauri::AppHandle>,
) -> AppResult<MoveOutcome> {
    let preview = preview_move_library(root, from, to)?;
    if !preview.fits {
        return Err(AppError::Invalid(
            preview
                .warnings
                .first()
                .cloned()
                .unwrap_or_else(|| "Not enough free space at the destination.".into()),
        ));
    }

    let source = PathBuf::from(from);
    let target = PathBuf::from(to);

    let mut progress = Progress::new(app, format!("Moving {from} → {to}"), preview.bytes);

    std::fs::create_dir_all(&target)?;

    // Move the contents (normally a single `steamapps` folder) into the target.
    for entry in std::fs::read_dir(&source)? {
        let entry = entry?;
        let destination = target.join(entry.file_name());
        if destination.exists() {
            return Err(AppError::Invalid(format!(
                "Destination already contains '{}'.",
                entry.file_name().to_string_lossy()
            )));
        }
        move_path(&entry.path(), &destination, &mut progress)?;
    }

    // Remove the now-empty source folder; ignore a non-empty failure.
    let _ = std::fs::remove_dir(&source);
    progress.finish();

    // Point the registry at the new path.
    let mut doc = load_library_folders(root)?;
    {
        let map = libraries_map_mut(&mut doc)?;
        let mut found = false;
        for value in map.values_mut() {
            if paths_equal(value.get_str("path").unwrap_or_default(), from) {
                if let vdf::Vdf::Object(fields) = value {
                    fields.insert("path".into(), vdf::Vdf::String(to.to_string()));
                }
                found = true;
            }
        }
        if !found {
            return Err(AppError::NotFound(format!(
                "'{from}' was not present in libraryfolders.vdf."
            )));
        }
    }
    let libraries = libraries_from_doc(&doc);
    let vdf_updated = write_library_folders(root, &doc, &libraries)?;

    Ok(MoveOutcome {
        from: from.to_string(),
        to: to.to_string(),
        bytes: preview.bytes,
        files: progress.files,
        same_volume: preview.same_volume,
        vdf_updated,
    })
}

/// Pre-flight for moving a single installed game to another library.
pub fn preview_move_game(root: &Path, app_id: u32, target: &str) -> AppResult<MovePreview> {
    let libraries = paths::read_library_folders(root);
    let placement = locate_game(&libraries, app_id)?;
    let target_library = libraries
        .iter()
        .find(|library| paths_equal(&library.path, target))
        .ok_or_else(|| {
            AppError::Invalid(format!("'{target}' is not a registered library folder."))
        })?;

    if paths_equal(&placement.library_path, &target_library.path) {
        return Err(AppError::Invalid(
            "The game is already in that library folder.".into(),
        ));
    }

    let mut warnings = Vec::new();
    let free = target_free_space(Path::new(&target_library.path));
    let fits = free
        .map(|free| free >= placement.size_on_disk + FREE_SPACE_SLACK)
        .unwrap_or(false);
    if !fits {
        warnings.push("Not enough free space at the destination library.".into());
    }

    Ok(MovePreview {
        from: placement.library_path.clone(),
        to: target_library.path.clone(),
        bytes: placement.size_on_disk,
        target_free: free,
        fits,
        same_volume: same_volume(
            Path::new(&placement.library_path),
            Path::new(&target_library.path),
        ),
        warnings,
    })
}

struct Placement {
    app_id: u32,
    name: String,
    library_path: String,
    install_dir: String,
    size_on_disk: u64,
    acf: PathBuf,
}

fn locate_game(libraries: &[SteamLibrary], app_id: u32) -> AppResult<Placement> {
    for library in libraries {
        let acf = Path::new(&library.path)
            .join("steamapps")
            .join(format!("appmanifest_{app_id}.acf"));
        if let Some(manifest) = read_manifest(&acf) {
            let folder = common_dir(&library.path, &manifest.install_dir);
            if manifest.install_dir.is_empty() || !folder.is_dir() {
                return Err(AppError::NotFound(format!(
                    "'{}' has no installed folder to move (incomplete install?).",
                    manifest.name
                )));
            }
            return Ok(Placement {
                app_id,
                name: manifest.name,
                library_path: library.path.clone(),
                install_dir: manifest.install_dir,
                size_on_disk: manifest.size_on_disk,
                acf,
            });
        }
    }
    Err(AppError::NotFound(format!(
        "App {app_id} is not installed in any library folder."
    )))
}

/// Move one installed game to another library and update the `apps` maps in
/// every `libraryfolders.vdf` copy.
pub fn move_game(
    root: &Path,
    app_id: u32,
    target: &str,
    app: Option<&tauri::AppHandle>,
) -> AppResult<MoveOutcome> {
    let libraries = paths::read_library_folders(root);
    let preview = preview_move_game(root, app_id, target)?;
    if !preview.fits {
        return Err(AppError::Invalid(
            preview
                .warnings
                .first()
                .cloned()
                .unwrap_or_else(|| "Not enough free space at the destination.".into()),
        ));
    }

    let placement = locate_game(&libraries, app_id)?;
    let target_library = libraries
        .iter()
        .find(|library| paths_equal(&library.path, target))
        .ok_or_else(|| {
            AppError::Invalid(format!("'{target}' is not a registered library folder."))
        })?;

    let source_library = placement.library_path.clone();
    let target_path = target_library.path.clone();

    let mut progress = Progress::new(
        app,
        format!("Moving {} → {target_path}", placement.name),
        placement.size_on_disk,
    );

    // 1. The game content.
    let source_dir = common_dir(&source_library, &placement.install_dir);
    let target_dir = common_dir(&target_path, &placement.install_dir);
    if target_dir.exists() {
        return Err(AppError::Invalid(format!(
            "'{}' already exists in the destination library.",
            placement.install_dir
        )));
    }
    move_path(&source_dir, &target_dir, &mut progress)?;

    // 2. The manifest.
    let target_acf = Path::new(&target_path)
        .join("steamapps")
        .join(format!("appmanifest_{app_id}.acf"));
    if let Some(parent) = target_acf.parent() {
        std::fs::create_dir_all(parent)?;
    }
    move_path(&placement.acf, &target_acf, &mut progress)?;

    // 3. Any Steam Workshop manifest for the app.
    let workshop_name = format!("appworkshop_{app_id}.acf");
    let source_workshop = Path::new(&source_library)
        .join("steamapps")
        .join("workshop")
        .join(&workshop_name);
    if source_workshop.is_file() {
        let target_workshop = Path::new(&target_path)
            .join("steamapps")
            .join("workshop")
            .join(&workshop_name);
        move_path(&source_workshop, &target_workshop, &mut progress)?;
    }

    progress.finish();

    // 4. Move the app id between the `apps` maps.
    let mut doc = load_library_folders(root)?;
    {
        let map = libraries_map_mut(&mut doc)?;
        let mut source_entry = None;
        let mut target_entry = None;
        for (key, value) in map.iter_mut() {
            let path = value.get_str("path").unwrap_or_default().to_string();
            if paths_equal(&path, &source_library) {
                source_entry = Some(key.clone());
            } else if paths_equal(&path, &target_path) {
                target_entry = Some(key.clone());
            }
        }
        let (Some(source_key), Some(target_key)) = (source_entry, target_entry) else {
            return Err(AppError::NotFound(
                "Could not find the source and destination library entries.".into(),
            ));
        };
        let size = placement.size_on_disk.to_string();
        if let Some(vdf::Vdf::Object(fields)) = map.get_mut(&source_key) {
            if let Some(vdf::Vdf::Object(apps)) = fields.get_mut("apps") {
                apps.remove(&app_id.to_string());
            }
        }
        if let Some(value) = map.get_mut(&target_key) {
            // Ensure the 'apps' child is an object.
            if value.get("apps").is_none() {
                value.insert("apps", vdf::Vdf::object());
            }
            if let Some(vdf::Vdf::Object(apps)) = value.get_mut("apps") {
                apps.insert(app_id.to_string(), vdf::Vdf::String(size));
            }
        }
    }
    let updated_libraries = libraries_from_doc(&doc);
    let vdf_updated = write_library_folders(root, &doc, &updated_libraries)?;

    Ok(MoveOutcome {
        from: source_library,
        to: target_path,
        bytes: placement.size_on_disk,
        files: progress.files,
        same_volume: preview.same_volume,
        vdf_updated,
    })
}

/// Register a new library folder (creating its `steamapps` directory).
pub fn add_library_folder(root: &Path, path: &str) -> AppResult<Vec<String>> {
    let trimmed = path.trim().trim_matches('"');
    if trimmed.is_empty() {
        return Err(AppError::Invalid("A folder path is required.".into()));
    }
    let folder = PathBuf::from(trimmed);
    let libraries = paths::read_library_folders(root);
    if libraries
        .iter()
        .any(|library| paths_equal(&library.path, trimmed))
    {
        return Err(AppError::Invalid(format!(
            "'{trimmed}' is already a registered library."
        )));
    }
    if is_inside(&folder, root) {
        return Err(AppError::Invalid(
            "A library folder cannot live inside the Steam installation folder.".into(),
        ));
    }
    if folder.exists()
        && std::fs::read_dir(&folder)
            .map(|mut dir| dir.next().is_some())
            .unwrap_or(false)
        && !folder.join("steamapps").is_dir()
    {
        return Err(AppError::Invalid(format!(
            "'{trimmed}' already exists and is not empty."
        )));
    }

    std::fs::create_dir_all(folder.join("steamapps"))?;

    let mut doc = load_library_folders(root)?;
    let next_index = libraries
        .iter()
        .map(|library| library.index)
        .max()
        .map(|max| max + 1)
        .unwrap_or(0);

    let total = disk_space_for(&folder, &disks_snapshot())
        .map(|(total, _free)| total)
        .unwrap_or(0);
    {
        let map = libraries_map_mut(&mut doc)?;
        let mut entry = vdf::Vdf::object();
        entry.insert_str("path", trimmed);
        entry.insert_str("label", "");
        entry.insert_str("totalsize", total.to_string());
        entry.insert("apps", vdf::Vdf::object());
        map.insert(next_index.to_string(), entry);
    }
    let updated = libraries_from_doc(&doc);
    write_library_folders(root, &doc, &updated)
}

/// Unregister an empty library folder (the files are left untouched).
pub fn remove_library_folder(root: &Path, path: &str) -> AppResult<Vec<String>> {
    find_library(root, path)?;
    if paths_equal(path, &root.display().to_string()) {
        return Err(AppError::Invalid(
            "The Steam installation's own library cannot be removed.".into(),
        ));
    }
    let manifests = manifests_in(path);
    if !manifests.is_empty() {
        return Err(AppError::Invalid(format!(
            "'{path}' still holds {} installed {}; move them first.",
            manifests.len(),
            if manifests.len() == 1 {
                "game"
            } else {
                "games"
            }
        )));
    }

    let mut doc = load_library_folders(root)?;
    {
        let map = libraries_map_mut(&mut doc)?;
        let key = map
            .iter()
            .find(|(_key, value)| paths_equal(value.get_str("path").unwrap_or_default(), path))
            .map(|(key, _value)| key.clone());
        let Some(key) = key else {
            return Err(AppError::NotFound(format!(
                "'{path}' was not present in libraryfolders.vdf."
            )));
        };
        map.remove(&key);
    }
    let updated = libraries_from_doc(&doc);
    write_library_folders(root, &doc, &updated)
}

/// Human-readable byte size, for preview warnings.
fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0usize;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.1} {}", UNITS[unit])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_windows_paths() {
        if cfg!(windows) {
            assert_eq!(normalize_path("D:\\Steam/"), normalize_path("d:\\steam"));
            assert!(paths_equal(
                "D:/Games\\SteamLibrary\\",
                "d:\\games\\steamlibrary"
            ));
        }
    }

    #[test]
    fn detects_nested_paths() {
        let parent = Path::new("D:\\Games");
        let child = Path::new("D:\\Games\\SteamLibrary");
        assert!(is_inside(child, parent));
        assert!(!is_inside(parent, child));
    }

    #[test]
    fn moves_app_between_apps_maps() {
        let input = r#"
        "libraryfolders"
        {
            "0"
            {
                "path"    "C:\\Program Files (x86)\\Steam"
                "apps"
                {
                    "440"   "12345"
                }
            }
            "1"
            {
                "path"    "D:\\SteamLibrary"
                "apps"
                {
                }
            }
        }
        "#;
        let mut doc = vdf::parse(input).unwrap();
        {
            let map = libraries_map_mut(&mut doc).unwrap();
            // Simulate the move of 440 from key "0" to key "1".
            if let Some(vdf::Vdf::Object(fields)) = map.get_mut("0") {
                if let Some(vdf::Vdf::Object(apps)) = fields.get_mut("apps") {
                    apps.remove("440");
                }
            }
            if let Some(value) = map.get_mut("1") {
                if value.get("apps").is_none() {
                    value.insert("apps", vdf::Vdf::object());
                }
                if let Some(vdf::Vdf::Object(apps)) = value.get_mut("apps") {
                    apps.insert("440".into(), vdf::Vdf::String("999".into()));
                }
            }
        }
        let libraries = libraries_from_doc(&doc);
        assert_eq!(libraries.len(), 2);
        assert!(libraries[0].app_ids.is_empty());
        assert_eq!(libraries[1].app_ids, vec![440]);
    }

    #[test]
    fn updates_library_path() {
        let input = r#"
        "libraryfolders"
        {
            "1"
            {
                "path"    "D:\\SteamLibrary"
                "label"   ""
                "apps"    { }
            }
        }
        "#;
        let mut doc = vdf::parse(input).unwrap();
        {
            let map = libraries_map_mut(&mut doc).unwrap();
            for value in map.values_mut() {
                if paths_equal(
                    value.get_str("path").unwrap_or_default(),
                    "D:\\SteamLibrary",
                ) {
                    if let vdf::Vdf::Object(fields) = value {
                        fields.insert("path".into(), vdf::Vdf::String("E:\\SteamLibrary".into()));
                    }
                }
            }
        }
        let libraries = libraries_from_doc(&doc);
        assert_eq!(libraries[0].path, "E:\\SteamLibrary");
    }

    #[test]
    fn round_trips_after_mutation() {
        let input =
            "\"libraryfolders\"\n{\n\t\"0\"\n\t{\n\t\t\"path\"\t\t\"C:\\\\Steam\"\n\t}\n}\n";
        let doc = vdf::parse(input).unwrap();
        let text = vdf::to_string(&doc);
        let reparsed = vdf::parse(&text).unwrap();
        assert_eq!(libraries_from_doc(&doc), libraries_from_doc(&reparsed));
    }
}
