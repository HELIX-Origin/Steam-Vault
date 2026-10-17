//! EmulationStation and ES-DE ROMs directory scaffolding, title normalization, and scanner.
//!
//! Conforms strictly to the EmulationStation Desktop Edition (ES-DE) folder hierarchy,
//! `systeminfo.txt` file format, and directory standard referenced in:
//! https://github.com/retrogamecorps/ES-DE-Directories

use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use super::{Platform, PlatformGame};

/// Standardized EmulationStation system folder specification.
#[derive(Debug, Clone, Copy)]
pub struct SystemFolderSpec {
    pub id: &'static str,
    pub full_name: &'static str,
    pub extensions: &'static str,
    pub launch_hint: &'static str,
}

/// Comprehensive list of standard ES-DE system directories and metadata.
pub const SYSTEM_FOLDER_SPECS: &[SystemFolderSpec] = &[
    SystemFolderSpec {
        id: "nes",
        full_name: "Nintendo Entertainment System",
        extensions: ".nes .NES .bin .BIN .fds .FDS .unf .UNF .zip .ZIP .7z .7Z",
        launch_hint: "%EMULATOR_RETROARCH% -L %CORE_RETROARCH%/mesen_libretro.so %ROM%",
    },
    SystemFolderSpec {
        id: "snes",
        full_name: "Super Nintendo Entertainment System",
        extensions: ".smc .SMC .sfc .SFC .fig .FIG .swc .SWC .bs .BS .zip .ZIP .7z .7Z",
        launch_hint: "%EMULATOR_RETROARCH% -L %CORE_RETROARCH%/snes9x_libretro.so %ROM%",
    },
    SystemFolderSpec {
        id: "n64",
        full_name: "Nintendo 64",
        extensions: ".z64 .Z64 .n64 .N64 .v64 .V64 .ndd .NDD .zip .ZIP .7z .7Z",
        launch_hint: "%EMULATOR_RETROARCH% -L %CORE_RETROARCH%/mupen64plus_next_libretro.so %ROM%",
    },
    SystemFolderSpec {
        id: "gc",
        full_name: "Nintendo GameCube",
        extensions: ".iso .ISO .ciso .CISO .gcm .GCM .gcz .GCZ .rvz .RVZ .wbfs .WBFS .m3u .M3U .zip .ZIP .7z .7Z",
        launch_hint: "dolphin-emu -b -e %ROM%",
    },
    SystemFolderSpec {
        id: "wii",
        full_name: "Nintendo Wii",
        extensions: ".iso .ISO .wbfs .WBFS .rvz .RVZ .ciso .CISO .gcz .GCZ .m3u .M3U .zip .ZIP .7z .7Z",
        launch_hint: "dolphin-emu -b -e %ROM%",
    },
    SystemFolderSpec {
        id: "wiiu",
        full_name: "Nintendo Wii U",
        extensions: ".wua .WUA .wud .WUD .wux .WUX .rpx .RPX .m3u .M3U .zip .ZIP .7z .7Z",
        launch_hint: "cemu -f -g %ROM%",
    },
    SystemFolderSpec {
        id: "switch",
        full_name: "Nintendo Switch",
        extensions: ".nsp .NSP .xci .XCI .nro .NRO .zip .ZIP .7z .7Z",
        launch_hint: "ryujinx %ROM%",
    },
    SystemFolderSpec {
        id: "gb",
        full_name: "Nintendo Game Boy",
        extensions: ".gb .GB .dmg .DMG .zip .ZIP .7z .7Z",
        launch_hint: "%EMULATOR_RETROARCH% -L %CORE_RETROARCH%/gambatte_libretro.so %ROM%",
    },
    SystemFolderSpec {
        id: "gba",
        full_name: "Nintendo Game Boy Advance",
        extensions: ".gba .GBA .bin .BIN .zip .ZIP .7z .7Z",
        launch_hint: "%EMULATOR_RETROARCH% -L %CORE_RETROARCH%/mgba_libretro.so %ROM%",
    },
    SystemFolderSpec {
        id: "gbc",
        full_name: "Nintendo Game Boy Color",
        extensions: ".gbc .GBC .zip .ZIP .7z .7Z",
        launch_hint: "%EMULATOR_RETROARCH% -L %CORE_RETROARCH%/gambatte_libretro.so %ROM%",
    },
    SystemFolderSpec {
        id: "nds",
        full_name: "Nintendo DS",
        extensions: ".nds .NDS .bin .BIN .zip .ZIP .7z .7Z",
        launch_hint: "melonds %ROM%",
    },
    SystemFolderSpec {
        id: "n3ds",
        full_name: "Nintendo 3DS",
        extensions: ".3ds .3DS .3dsx .3DSX .cia .CIA .cci .CCI .cxi .CXI .zip .ZIP .7z .7Z",
        launch_hint: "citra-qt %ROM%",
    },
    SystemFolderSpec {
        id: "psx",
        full_name: "Sony PlayStation 1",
        extensions: ".chd .CHD .cue .CUE .iso .ISO .pbp .PBP .bin .BIN .m3u .M3U .zip .ZIP .7z .7Z",
        launch_hint: "duckstation-qt -batch %ROM%",
    },
    SystemFolderSpec {
        id: "ps2",
        full_name: "Sony PlayStation 2",
        extensions: ".iso .ISO .chd .CHD .cso .CSO .bin .BIN .gz .GZ .m3u .M3U .zip .ZIP .7z .7Z",
        launch_hint: "pcsx2-qt -nogui -fullscreen %ROM%",
    },
    SystemFolderSpec {
        id: "ps3",
        full_name: "Sony PlayStation 3",
        extensions: ".iso .ISO .bin .BIN .m3u .M3U .zip .ZIP .7z .7Z",
        launch_hint: "rpcs3 --no-gui %ROM%",
    },
    SystemFolderSpec {
        id: "psp",
        full_name: "Sony PlayStation Portable",
        extensions: ".iso .ISO .cso .CSO .pbp .PBP .chd .CHD .zip .ZIP .7z .7Z",
        launch_hint: "PPSSPPWindows64 %ROM%",
    },
    SystemFolderSpec {
        id: "psvita",
        full_name: "Sony PlayStation Vita",
        extensions: ".vpk .VPK .zip .ZIP .7z .7Z",
        launch_hint: "Vita3K -r %ROM%",
    },
    SystemFolderSpec {
        id: "megadrive",
        full_name: "Sega Mega Drive",
        extensions: ".md .MD .smd .SMD .gen .GEN .bin .BIN .zip .ZIP .7z .7Z",
        launch_hint: "%EMULATOR_RETROARCH% -L %CORE_RETROARCH%/genesis_plus_gx_libretro.so %ROM%",
    },
    SystemFolderSpec {
        id: "genesis",
        full_name: "Sega Genesis",
        extensions: ".gen .GEN .bin .BIN .md .MD .smd .SMD .zip .ZIP .7z .7Z",
        launch_hint: "%EMULATOR_RETROARCH% -L %CORE_RETROARCH%/genesis_plus_gx_libretro.so %ROM%",
    },
    SystemFolderSpec {
        id: "mastersystem",
        full_name: "Sega Master System",
        extensions: ".sms .SMS .bin .BIN .zip .ZIP .7z .7Z",
        launch_hint: "%EMULATOR_RETROARCH% -L %CORE_RETROARCH%/genesis_plus_gx_libretro.so %ROM%",
    },
    SystemFolderSpec {
        id: "gamegear",
        full_name: "Sega Game Gear",
        extensions: ".gg .GG .bin .BIN .zip .ZIP .7z .7Z",
        launch_hint: "%EMULATOR_RETROARCH% -L %CORE_RETROARCH%/genesis_plus_gx_libretro.so %ROM%",
    },
    SystemFolderSpec {
        id: "saturn",
        full_name: "Sega Saturn",
        extensions: ".chd .CHD .cue .CUE .iso .ISO .bin .BIN .m3u .M3U .zip .ZIP .7z .7Z",
        launch_hint: "%EMULATOR_RETROARCH% -L %CORE_RETROARCH%/mednafen_saturn_libretro.so %ROM%",
    },
    SystemFolderSpec {
        id: "dreamcast",
        full_name: "Sega Dreamcast",
        extensions: ".cdi .CDI .gdi .GDI .chd .CHD .iso .ISO .m3u .M3U .zip .ZIP .7z .7Z",
        launch_hint: "%EMULATOR_RETROARCH% -L %CORE_RETROARCH%/flycast_libretro.so %ROM%",
    },
    SystemFolderSpec {
        id: "segacd",
        full_name: "Sega CD",
        extensions: ".chd .CHD .cue .CUE .iso .ISO .bin .BIN .m3u .M3U .zip .ZIP .7z .7Z",
        launch_hint: "%EMULATOR_RETROARCH% -L %CORE_RETROARCH%/genesis_plus_gx_libretro.so %ROM%",
    },
    SystemFolderSpec {
        id: "sega32x",
        full_name: "Sega 32X",
        extensions: ".32x .32X .bin .BIN .zip .ZIP .7z .7Z",
        launch_hint: "%EMULATOR_RETROARCH% -L %CORE_RETROARCH%/picodrive_libretro.so %ROM%",
    },
    SystemFolderSpec {
        id: "arcade",
        full_name: "Arcade Games",
        extensions: ".zip .ZIP .7z .7Z",
        launch_hint: "%EMULATOR_RETROARCH% -L %CORE_RETROARCH%/fbneo_libretro.so %ROM%",
    },
    SystemFolderSpec {
        id: "mame",
        full_name: "MAME Arcade",
        extensions: ".zip .ZIP .7z .7Z .chd .CHD",
        launch_hint: "%EMULATOR_RETROARCH% -L %CORE_RETROARCH%/mame_libretro.so %ROM%",
    },
    SystemFolderSpec {
        id: "neogeo",
        full_name: "SNK Neo Geo AES/MVS",
        extensions: ".zip .ZIP .7z .7Z",
        launch_hint: "%EMULATOR_RETROARCH% -L %CORE_RETROARCH%/fbneo_libretro.so %ROM%",
    },
    SystemFolderSpec {
        id: "ngp",
        full_name: "SNK Neo Geo Pocket",
        extensions: ".ngp .NGP .zip .ZIP .7z .7Z",
        launch_hint: "%EMULATOR_RETROARCH% -L %CORE_RETROARCH%/mednafen_ngp_libretro.so %ROM%",
    },
    SystemFolderSpec {
        id: "ngpc",
        full_name: "SNK Neo Geo Pocket Color",
        extensions: ".ngc .NGC .ngp .NGP .zip .ZIP .7z .7Z",
        launch_hint: "%EMULATOR_RETROARCH% -L %CORE_RETROARCH%/mednafen_ngp_libretro.so %ROM%",
    },
    SystemFolderSpec {
        id: "atari2600",
        full_name: "Atari 2600",
        extensions: ".a26 .A26 .bin .BIN .zip .ZIP .7z .7Z",
        launch_hint: "%EMULATOR_RETROARCH% -L %CORE_RETROARCH%/stella_libretro.so %ROM%",
    },
    SystemFolderSpec {
        id: "atari5200",
        full_name: "Atari 5200",
        extensions: ".a52 .A52 .bin .BIN .zip .ZIP .7z .7Z",
        launch_hint: "%EMULATOR_RETROARCH% -L %CORE_RETROARCH%/atari800_libretro.so %ROM%",
    },
    SystemFolderSpec {
        id: "atari7800",
        full_name: "Atari 7800",
        extensions: ".a78 .A78 .bin .BIN .zip .ZIP .7z .7Z",
        launch_hint: "%EMULATOR_RETROARCH% -L %CORE_RETROARCH%/prosystem_libretro.so %ROM%",
    },
    SystemFolderSpec {
        id: "atarijaguar",
        full_name: "Atari Jaguar",
        extensions: ".j64 .J64 .jag .JAG .zip .ZIP .7z .7Z",
        launch_hint: "%EMULATOR_RETROARCH% -L %CORE_RETROARCH%/virtualjaguar_libretro.so %ROM%",
    },
    SystemFolderSpec {
        id: "atarilynx",
        full_name: "Atari Lynx",
        extensions: ".lnx .LNX .zip .ZIP .7z .7Z",
        launch_hint: "%EMULATOR_RETROARCH% -L %CORE_RETROARCH%/handy_libretro.so %ROM%",
    },
    SystemFolderSpec {
        id: "atarist",
        full_name: "Atari ST",
        extensions: ".st .ST .msa .MSA .ipf .IPF .zip .ZIP .7z .7Z",
        launch_hint: "%EMULATOR_RETROARCH% -L %CORE_RETROARCH%/hatari_libretro.so %ROM%",
    },
    SystemFolderSpec {
        id: "pcengine",
        full_name: "NEC PC Engine / TurboGrafx-16",
        extensions: ".pce .PCE .bin .BIN .zip .ZIP .7z .7Z",
        launch_hint: "%EMULATOR_RETROARCH% -L %CORE_RETROARCH%/mednafen_pce_fast_libretro.so %ROM%",
    },
    SystemFolderSpec {
        id: "pcenginecd",
        full_name: "NEC PC Engine CD / TurboGrafx-CD",
        extensions: ".chd .CHD .cue .CUE .iso .ISO .bin .BIN .m3u .M3U",
        launch_hint: "%EMULATOR_RETROARCH% -L %CORE_RETROARCH%/mednafen_pce_fast_libretro.so %ROM%",
    },
    SystemFolderSpec {
        id: "wonderswan",
        full_name: "Bandai WonderSwan",
        extensions: ".ws .WS .zip .ZIP .7z .7Z",
        launch_hint: "%EMULATOR_RETROARCH% -L %CORE_RETROARCH%/mednafen_wswan_libretro.so %ROM%",
    },
    SystemFolderSpec {
        id: "wonderswancolor",
        full_name: "Bandai WonderSwan Color",
        extensions: ".wsc .WSC .zip .ZIP .7z .7Z",
        launch_hint: "%EMULATOR_RETROARCH% -L %CORE_RETROARCH%/mednafen_wswan_libretro.so %ROM%",
    },
    SystemFolderSpec {
        id: "colecovision",
        full_name: "ColecoVision",
        extensions: ".col .COL .bin .BIN .rom .ROM .zip .ZIP .7z .7Z",
        launch_hint: "%EMULATOR_RETROARCH% -L %CORE_RETROARCH%/gearcoleco_libretro.so %ROM%",
    },
    SystemFolderSpec {
        id: "intellivision",
        full_name: "Intellivision",
        extensions: ".int .INT .bin .BIN .rom .ROM .zip .ZIP .7z .7Z",
        launch_hint: "%EMULATOR_RETROARCH% -L %CORE_RETROARCH%/freeintv_libretro.so %ROM%",
    },
    SystemFolderSpec {
        id: "virtualboy",
        full_name: "Nintendo Virtual Boy",
        extensions: ".vb .VB .bin .BIN .zip .ZIP .7z .7Z",
        launch_hint: "%EMULATOR_RETROARCH% -L %CORE_RETROARCH%/mednafen_vb_libretro.so %ROM%",
    },
    SystemFolderSpec {
        id: "c64",
        full_name: "Commodore 64",
        extensions: ".d64 .D64 .t64 .T64 .prg .PRG .crt .CRT .zip .ZIP .7z .7Z",
        launch_hint: "%EMULATOR_RETROARCH% -L %CORE_RETROARCH%/vice_x64sc_libretro.so %ROM%",
    },
    SystemFolderSpec {
        id: "amiga",
        full_name: "Commodore Amiga",
        extensions: ".adf .ADF .ipf .IPF .lha .LHA .hdf .HDF .zip .ZIP .7z .7Z",
        launch_hint: "%EMULATOR_RETROARCH% -L %CORE_RETROARCH%/puae_libretro.so %ROM%",
    },
    SystemFolderSpec {
        id: "dos",
        full_name: "MS-DOS PC Games",
        extensions: ".exe .EXE .com .COM .bat .BAT .conf .CONF .iso .ISO",
        launch_hint: "dosbox-staging %ROM%",
    },
    SystemFolderSpec {
        id: "scummvm",
        full_name: "ScummVM Adventures",
        extensions: ".scummvm .SCUMMVM",
        launch_hint: "scummvm %ROM%",
    },
    SystemFolderSpec {
        id: "3do",
        full_name: "3DO Interactive Multiplayer",
        extensions: ".iso .ISO .chd .CHD .cue .CUE .bin .BIN",
        launch_hint: "%EMULATOR_RETROARCH% -L %CORE_RETROARCH%/opera_libretro.so %ROM%",
    },
];

/// Result report from scaffolding ROM directories.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScaffoldReport {
    pub roms_dir: String,
    pub created_folders_count: usize,
    pub total_systems_count: usize,
    pub scaffolded_systems: Vec<String>,
    pub message: String,
}

pub use crate::plugins::title_normalizer::{
    canonical_title_key, normalize_game_title, normalize_rom_title,
};


/// Scaffolds EmulationStation standardized system directories and necessary `.txt` files
/// inside the user-selected ROM directory.
///
/// Follows the `retrogamecorps/ES-DE-Directories` architecture:
/// - In each system folder: creates `systeminfo.txt` with system name, full name, supported extensions, and launch commands.
/// - At the root of the ROMs folder: creates `systems.txt` and `README.txt`.
pub fn scaffold_emulation_folders(roms_dir: &str) -> AppResult<ScaffoldReport> {
    let base_path = PathBuf::from(roms_dir.trim());
    if base_path.as_os_str().is_empty() {
        return Err(AppError::Invalid("ROM directory path cannot be empty.".into()));
    }

    fs::create_dir_all(&base_path)?;

    let mut created_count = 0;
    let mut scaffolded = Vec::with_capacity(SYSTEM_FOLDER_SPECS.len());

    // 1. Scaffold system subfolders and systeminfo.txt files
    for spec in SYSTEM_FOLDER_SPECS {
        let system_dir = base_path.join(spec.id);
        if !system_dir.exists() {
            fs::create_dir_all(&system_dir)?;
            created_count += 1;
        }

        let info_file = system_dir.join("systeminfo.txt");
        if !info_file.exists() {
            let content = format!(
                "System name:\r\n{}\r\n\r\n\
                 Full system name:\r\n{}\r\n\r\n\
                 Supported file extensions:\r\n{}\r\n\r\n\
                 Launch command:\r\n{}\r\n\r\n\
                 Platform (for scraping):\r\n{}\r\n\r\n\
                 Theme folder:\r\n{}\r\n",
                spec.id, spec.full_name, spec.extensions, spec.launch_hint, spec.id, spec.id
            );
            let _ = fs::write(&info_file, content);
        }

        scaffolded.push(spec.id.to_string());
    }

    // 2. Scaffold root systems.txt reference index
    let systems_txt = base_path.join("systems.txt");
    let mut systems_index = String::from(
        "EmulationStation & ES-DE System Directories Index\r\n\
         =================================================\r\n\r\n\
         Folder Name      | Full Console Platform Name\r\n\
         -----------------+------------------------------------------------------------\r\n"
    );
    for spec in SYSTEM_FOLDER_SPECS {
        systems_index.push_str(&format!("{:<16} | {}\r\n", spec.id, spec.full_name));
    }
    let _ = fs::write(&systems_txt, systems_index);

    // 3. Scaffold root README.txt guide
    let readme_txt = base_path.join("README.txt");
    let readme_content = format!(
        "Universal Game Launcher & EmulationStation ROMs Directory\r\n\
         ========================================================\r\n\r\n\
         This folder has been scaffolded conforming strictly to the EmulationStation\r\n\
         Desktop Edition (ES-DE) standard (ref: https://github.com/retrogamecorps/ES-DE-Directories).\r\n\r\n\
         HOW TO USE:\r\n\
         1. Place your ROM files directly into their respective console subfolders (e.g. 'nes', 'snes', 'n64', 'psx').\r\n\
         2. Each subfolder contains a 'systeminfo.txt' file listing all supported file extensions for that system.\r\n\
         3. BIOS files should generally be placed in the 'bios/' subfolder or directly within the console's emulator directory.\r\n\
         4. Universal Game Launcher will automatically scan this folder, normalize dump tags (e.g. '(USA)', '[!]'),\r\n\
            and present your retro titles cleanly in your library.\r\n\r\n\
         Total scaffolded systems: {}\r\n",
        SYSTEM_FOLDER_SPECS.len()
    );
    let _ = fs::write(&readme_txt, readme_content);

    Ok(ScaffoldReport {
        roms_dir: base_path.display().to_string(),
        created_folders_count: created_count,
        total_systems_count: SYSTEM_FOLDER_SPECS.len(),
        scaffolded_systems: scaffolded,
        message: format!(
            "Successfully verified and scaffolded {} EmulationStation system directories in '{}'.",
            SYSTEM_FOLDER_SPECS.len(),
            base_path.display()
        ),
    })
}

/// Structured representation of a ROM index file (`rom.ini`) stored alongside an extracted ROM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RomIni {
    pub title: String,
    pub file: String,
    pub archive: Option<String>,
    pub system: String,
    pub console: String,
    pub save_dir: String,
}

/// Sanitizes a title string so it can be safely used as a directory name on Windows/macOS/Linux.
fn sanitize_folder_name(name: &str) -> String {
    let invalid = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    let cleaned: String = name
        .chars()
        .map(|c| if invalid.contains(&c) { '-' } else { c })
        .collect();
    cleaned.trim().trim_end_matches('.').trim().to_string()
}

/// Reads and parses `rom.ini` from a ROM's dedicated subfolder.
pub fn read_rom_ini(folder: &std::path::Path) -> Option<RomIni> {
    let ini_path = folder.join("rom.ini");
    let content = fs::read_to_string(ini_path).ok()?;
    let mut title = String::new();
    let mut file = String::new();
    let mut archive = None;
    let mut system = String::new();
    let mut console = String::new();
    let mut save_dir = "saves".to_string();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('[') || trimmed.starts_with(';') || trimmed.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = trimmed.split_once('=') {
            let key = k.trim().to_lowercase();
            let val = v.trim().to_string();
            match key.as_str() {
                "title" => title = val,
                "file" => file = val,
                "archive" => {
                    if !val.is_empty() {
                        archive = Some(val);
                    }
                }
                "system" => system = val,
                "console" => console = val,
                "save_dir" => save_dir = val,
                _ => {}
            }
        }
    }

    if title.is_empty() || file.is_empty() {
        return None;
    }

    let clean_title = normalize_rom_title(&title);

    Some(RomIni {
        title: if clean_title.is_empty() { title } else { clean_title },
        file,
        archive,
        system,
        console,
        save_dir,
    })
}

/// Writes `rom.ini` and creates the isolated `saves/` folder inside the ROM's dedicated directory.
pub fn write_rom_ini(folder: &std::path::Path, ini: &RomIni) {
    let saves_path = folder.join(&ini.save_dir);
    let _ = fs::create_dir_all(&saves_path);

    let ini_content = format!(
        "[ROM]\r\n\
         title={}\r\n\
         file={}\r\n\
         archive={}\r\n\
         system={}\r\n\
         console={}\r\n\
         save_dir={}\r\n",
        ini.title,
        ini.file,
        ini.archive.as_deref().unwrap_or(""),
        ini.system,
        ini.console,
        ini.save_dir
    );
    let _ = fs::write(folder.join("rom.ini"), ini_content);
}

/// Extracts a `.zip` or `.7z` ROM archive into `target_dir`, flattens any nested single-folder wrapper,
/// and returns the filename of the primary extracted ROM file.
fn extract_rom_archive_to_folder(
    archive_path: &std::path::Path,
    target_dir: &std::path::Path,
    valid_rom_extensions: &[String],
) -> Option<PathBuf> {
    let _ = fs::create_dir_all(target_dir);

    let archive_str = archive_path.to_str()?;
    let target_str = target_dir.to_str()?;

    // Use system `tar -xf` (available natively on Windows 10/11, macOS, and Linux)
    let mut cmd = std::process::Command::new("tar");
    cmd.args(["-xf", archive_str, "-C", target_str]);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    let status = cmd.status().ok();
    let tar_ok = status.map(|s| s.success()).unwrap_or(false);

    if !tar_ok {
        // Fallback to PowerShell Expand-Archive on Windows for .zip
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            let mut ps = std::process::Command::new("powershell");
            ps.args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                &format!(
                    "Expand-Archive -LiteralPath '{}' -DestinationPath '{}' -Force",
                    archive_str.replace('\'', "''"),
                    target_str.replace('\'', "''")
                ),
            ]);
            ps.creation_flags(0x08000000);
            let _ = ps.status();
        }
    }

    // Find the extracted ROM file inside target_dir (up to depth 2)
    let found = find_rom_in_dir(target_dir, valid_rom_extensions, 0)?;

    // If the extracted ROM was inside a nested child directory of target_dir, move it up to target_dir
    if let Some(parent) = found.parent() {
        if parent != target_dir {
            if let Some(fname) = found.file_name() {
                let dest = target_dir.join(fname);
                if fs::rename(&found, &dest).is_ok() {
                    let _ = fs::remove_dir_all(parent);
                    return Some(dest);
                }
            }
        }
    }

    Some(found)
}

/// Recursively finds the primary non-archive ROM file inside `dir` up to depth 2.
fn find_rom_in_dir(
    dir: &std::path::Path,
    valid_extensions: &[String],
    depth: usize,
) -> Option<PathBuf> {
    if depth > 2 || !dir.is_dir() {
        return None;
    }

    let archive_exts = ["zip", "7z", "rar"];
    let mut subdirs = Vec::new();
    let mut fallback_archive = None;

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !name.eq_ignore_ascii_case("saves") && !name.starts_with('.') {
                    subdirs.push(path);
                }
            } else if path.is_file() {
                let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if fname.starts_with('.')
                    || fname.eq_ignore_ascii_case("rom.ini")
                    || fname.eq_ignore_ascii_case("systeminfo.txt")
                    || fname.eq_ignore_ascii_case("noload.txt")
                    || fname.ends_with(".sav")
                    || fname.ends_with(".srm")
                    || fname.ends_with(".state")
                {
                    continue;
                }
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let lower = ext.to_lowercase();
                    if valid_extensions.contains(&lower) {
                        if archive_exts.contains(&lower.as_str()) {
                            fallback_archive = Some(path);
                        } else {
                            return Some(path);
                        }
                    }
                }
            }
        }
    }

    for sub in subdirs {
        if let Some(found) = find_rom_in_dir(&sub, valid_extensions, depth + 1) {
            return Some(found);
        }
    }

    fallback_archive
}

/// Auto-detects ROM root directories across all mounted drives and user directories.
pub fn detect_rom_directories() -> Vec<String> {
    let mut found = Vec::new();
    let candidates = [
        "Games\\ES-DE\\ROMs",
        "ES-DE\\ROMs",
        "Games\\ROMs",
        "ROMs",
        "Emulation\\ROMs",
        "Emulators\\ROMs",
    ];

    for drive in crate::plugins::engine::drive_roots() {
        for rel in &candidates {
            let path = drive.join(rel);
            if path.is_dir() {
                let s = path.display().to_string();
                if !found.contains(&s) {
                    found.push(s);
                }
            }
        }
    }

    if let Some(home) = dirs::home_dir() {
        for rel in &["ROMs", "ES-DE\\ROMs", "Games\\ES-DE\\ROMs", "Emulation\\ROMs"] {
            let path = home.join(rel);
            if path.is_dir() {
                let s = path.display().to_string();
                if !found.contains(&s) {
                    found.push(s);
                }
            }
        }
    }

    found
}

/// Scans a single ROM root directory (and auto-extracts archives + generates `rom.ini` indices).
fn scan_single_rom_root(base_path: &std::path::Path, normalize: bool) -> Vec<PlatformGame> {
    let mut games = Vec::new();
    if !base_path.is_dir() {
        return games;
    }

    let archive_exts = ["zip", "7z", "rar"];

    for spec in SYSTEM_FOLDER_SPECS {
        let system_dir = base_path.join(spec.id);
        if !system_dir.is_dir() {
            continue;
        }

        let valid_extensions: Vec<String> = spec
            .extensions
            .split_whitespace()
            .map(|ext| ext.trim_start_matches('.').to_lowercase())
            .collect();

        // PASS 1: Check for archives (.zip, .7z, .rar) in `system_dir` and auto-extract them
        // into their own dedicated game folder with a `rom.ini` and `saves/` directory.
        if let Ok(entries) = fs::read_dir(&system_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();

                if !archive_exts.contains(&ext.as_str()) {
                    continue;
                }

                let normalized_name = normalize_rom_title(file_name);
                let folder_name = sanitize_folder_name(if normalized_name.is_empty() {
                    path.file_stem().and_then(|s| s.to_str()).unwrap_or("Game")
                } else {
                    &normalized_name
                });

                let game_folder = system_dir.join(&folder_name);

                // Also check if a folder matching raw archive stem already exists
                let raw_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                let raw_folder = system_dir.join(raw_stem);
                let target_folder = if raw_folder.is_dir() && raw_folder != system_dir {
                    raw_folder
                } else {
                    game_folder
                };

                // Check if target_folder already has a valid rom.ini or extracted ROM
                let already_extracted = if let Some(ini) = read_rom_ini(&target_folder) {
                    target_folder.join(&ini.file).is_file()
                } else {
                    find_rom_in_dir(&target_folder, &valid_extensions, 0).is_some()
                };

                if !already_extracted {
                    if let Some(extracted_rom) =
                        extract_rom_archive_to_folder(&path, &target_folder, &valid_extensions)
                    {
                        let rom_fname = extracted_rom
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("")
                            .to_string();
                        let display_title = if normalize {
                            normalize_rom_title(&rom_fname)
                        } else {
                            normalized_name.clone()
                        };
                        let ini = RomIni {
                            title: display_title,
                            file: rom_fname,
                            archive: Some(file_name.to_string()),
                            system: spec.id.to_string(),
                            console: spec.full_name.to_string(),
                            save_dir: "saves".to_string(),
                        };
                        write_rom_ini(&target_folder, &ini);
                    }
                }
            }
        }

        // PASS 2: Scan all dedicated game folders inside `system_dir` using `rom.ini` (or creating it if missing)
        let mut seen_titles = std::collections::HashSet::new();

        if let Ok(entries) = fs::read_dir(&system_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if dir_name.starts_with('.') || dir_name.eq_ignore_ascii_case("saves") || dir_name.eq_ignore_ascii_case("media") {
                    continue;
                }

                // 1. Check if `rom.ini` exists
                let resolved_ini = if let Some(ini) = read_rom_ini(&path) {
                    if path.join(&ini.file).is_file() {
                        Some((ini.clone(), path.join(&ini.file)))
                    } else {
                        None
                    }
                } else {
                    None
                };

                let (ini, rom_file_path) = if let Some(pair) = resolved_ini {
                    pair
                } else if let Some(found_rom) = find_rom_in_dir(&path, &valid_extensions, 0) {
                    let rom_fname = found_rom
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();
                    let display_title = if normalize {
                        normalize_rom_title(&rom_fname)
                    } else {
                        dir_name.to_string()
                    };
                    // Relative path from `path` to `found_rom`
                    let rel_file = found_rom
                        .strip_prefix(&path)
                        .unwrap_or(&found_rom)
                        .display()
                        .to_string();
                    let new_ini = RomIni {
                        title: display_title,
                        file: rel_file,
                        archive: None,
                        system: spec.id.to_string(),
                        console: spec.full_name.to_string(),
                        save_dir: "saves".to_string(),
                    };
                    write_rom_ini(&path, &new_ini);
                    (new_ini, found_rom)
                } else {
                    continue;
                };

                let size_on_disk = fs::metadata(&rom_file_path).map(|m| m.len()).ok();
                seen_titles.insert(canonical_title_key(&ini.title));

                games.push(PlatformGame {
                    name: ini.title.clone(),
                    platform: Platform::EmulationStation,
                    platform_id: Some(format!("{}:{}", spec.id, ini.file)),
                    install_dir: Some(path.display().to_string()),
                    exe: Some(rom_file_path.display().to_string()),
                    size_on_disk,
                    launch_uri: None,
                    source: Some("EmulationStation".to_string()),
                    console: Some(spec.full_name.to_string()),
                    installed: true,
                    install_uri: None,
                    uninstall_uri: None,
                });
            }
        }

        // PASS 3: Also pick up any loose ROM files directly in `system_dir` whose titles weren't already added via a subfolder
        if let Ok(entries) = fs::read_dir(&system_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if file_name.starts_with('.')
                    || file_name.eq_ignore_ascii_case("rom.ini")
                    || file_name.eq_ignore_ascii_case("systeminfo.txt")
                    || file_name.eq_ignore_ascii_case("noload.txt")
                    || file_name.ends_with(".sav")
                    || file_name.ends_with(".srm")
                    || file_name.ends_with(".state")
                {
                    continue;
                }

                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let lower_ext = ext.to_lowercase();
                    if valid_extensions.contains(&lower_ext) {
                        let display_name = if normalize {
                            normalize_rom_title(file_name)
                        } else {
                            path.file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or(file_name)
                                .to_string()
                        };

                        let key = canonical_title_key(&display_name);
                        if seen_titles.contains(&key) {
                            continue;
                        }
                        seen_titles.insert(key);

                        let size_on_disk = entry.metadata().map(|m| m.len()).ok();
                        games.push(PlatformGame {
                            name: display_name,
                            platform: Platform::EmulationStation,
                            platform_id: Some(format!("{}:{}", spec.id, file_name)),
                            install_dir: Some(system_dir.display().to_string()),
                            exe: Some(path.display().to_string()),
                            size_on_disk,
                            launch_uri: None,
                            source: Some("EmulationStation".to_string()),
                            console: Some(spec.full_name.to_string()),
                            installed: true,
                            install_uri: None,
                            uninstall_uri: None,
                        });
                    }
                }
            }
        }
    }

    games
}

/// Scans the user-selected ROM directory (and all auto-detected ROM directories across drives)
/// for games across all scaffolded console systems.
/// Automatically extracts archived ROMs into dedicated subfolders with `rom.ini` and `saves/`.
pub fn scan_emulation_roms(roms_dir: &str, normalize: bool) -> AppResult<Vec<PlatformGame>> {
    let mut roots_to_scan = Vec::new();
    let trimmed = roms_dir.trim();
    if !trimmed.is_empty() && PathBuf::from(trimmed).is_dir() {
        roots_to_scan.push(PathBuf::from(trimmed));
    }

    for detected in detect_rom_directories() {
        let p = PathBuf::from(&detected);
        if !roots_to_scan.contains(&p) {
            roots_to_scan.push(p);
        }
    }

    let mut all_games = Vec::new();
    let mut seen_exes = std::collections::HashSet::new();

    for root in roots_to_scan {
        for game in scan_single_rom_root(&root, normalize) {
            let key = game.exe.clone().unwrap_or_else(|| game.name.clone()).to_lowercase();
            if seen_exes.insert(key) {
                all_games.push(game);
            }
        }
    }

    // Sort games primarily by platform/console, then alphabetically by title
    all_games.sort_by(|a, b| {
        let console_a = a.console.as_deref().unwrap_or("");
        let console_b = b.console.as_deref().unwrap_or("");
        match console_a.cmp(console_b) {
            std::cmp::Ordering::Equal => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            other => other,
        }
    });

    Ok(all_games)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_rom_titles() {
        assert_eq!(
            normalize_rom_title("Super_Mario_World_(USA)_[!].sfc"),
            "Super Mario World"
        );
        assert_eq!(
            normalize_rom_title("Legend of Zelda, The - Ocarina of Time (USA) (Rev 2).z64"),
            "The Legend of Zelda: Ocarina of Time"
        );
        assert_eq!(
            normalize_rom_title("Metal Gear Solid (USA) (Disc 1).chd"),
            "Metal Gear Solid (Disc 1)"
        );
        assert_eq!(
            normalize_rom_title("Pokemon - Emerald Version (USA, Europe).gba"),
            "Pokemon - Emerald Version"
        );
        assert_eq!(
            normalize_rom_title("Sonic the Hedgehog (USA, Europe) (En).bin"),
            "Sonic the Hedgehog"
        );
    }
}
