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
        extensions: ".iso .ISO .ciso .CISO .gcm .GCM .gcz .GCZ .rvz .RVZ .wbfs .WBFS .m3u .M3U",
        launch_hint: "dolphin-emu -b -e %ROM%",
    },
    SystemFolderSpec {
        id: "wii",
        full_name: "Nintendo Wii",
        extensions: ".iso .ISO .wbfs .WBFS .rvz .RVZ .ciso .CISO .gcz .GCZ .m3u .M3U",
        launch_hint: "dolphin-emu -b -e %ROM%",
    },
    SystemFolderSpec {
        id: "wiiu",
        full_name: "Nintendo Wii U",
        extensions: ".wua .WUA .wud .WUD .wux .WUX .rpx .RPX .m3u .M3U",
        launch_hint: "cemu -f -g %ROM%",
    },
    SystemFolderSpec {
        id: "switch",
        full_name: "Nintendo Switch",
        extensions: ".nsp .NSP .xci .XCI .nro .NRO",
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
        extensions: ".3ds .3DS .3dsx .3DSX .cia .CIA .cci .CCI .cxi .CXI",
        launch_hint: "citra-qt %ROM%",
    },
    SystemFolderSpec {
        id: "psx",
        full_name: "Sony PlayStation 1",
        extensions: ".chd .CHD .cue .CUE .iso .ISO .pbp .PBP .bin .BIN .m3u .M3U",
        launch_hint: "duckstation-qt -batch %ROM%",
    },
    SystemFolderSpec {
        id: "ps2",
        full_name: "Sony PlayStation 2",
        extensions: ".iso .ISO .chd .CHD .cso .CSO .bin .BIN .gz .GZ .m3u .M3U",
        launch_hint: "pcsx2-qt -nogui -fullscreen %ROM%",
    },
    SystemFolderSpec {
        id: "ps3",
        full_name: "Sony PlayStation 3",
        extensions: ".iso .ISO .bin .BIN .m3u .M3U",
        launch_hint: "rpcs3 --no-gui %ROM%",
    },
    SystemFolderSpec {
        id: "psp",
        full_name: "Sony PlayStation Portable",
        extensions: ".iso .ISO .cso .CSO .pbp .PBP .chd .CHD",
        launch_hint: "PPSSPPWindows64 %ROM%",
    },
    SystemFolderSpec {
        id: "psvita",
        full_name: "Sony PlayStation Vita",
        extensions: ".vpk .VPK .zip .ZIP",
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
        extensions: ".chd .CHD .cue .CUE .iso .ISO .bin .BIN .m3u .M3U",
        launch_hint: "%EMULATOR_RETROARCH% -L %CORE_RETROARCH%/mednafen_saturn_libretro.so %ROM%",
    },
    SystemFolderSpec {
        id: "dreamcast",
        full_name: "Sega Dreamcast",
        extensions: ".cdi .CDI .gdi .GDI .chd .CHD .iso .ISO .m3u .M3U",
        launch_hint: "%EMULATOR_RETROARCH% -L %CORE_RETROARCH%/flycast_libretro.so %ROM%",
    },
    SystemFolderSpec {
        id: "segacd",
        full_name: "Sega CD",
        extensions: ".chd .CHD .cue .CUE .iso .ISO .bin .BIN .m3u .M3U",
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

fn invert_article(title: &str) -> String {
    let s = title.trim();

    // Check with separators: " - ", " – ", " — ", ": "
    for sep in [" - ", " – ", " — ", ": "] {
        for (article_suffix, article_prefix) in [
            (", The", "The "),
            (", the", "The "),
            (", A", "A "),
            (", a", "A "),
            (", An", "An "),
            (", an", "An "),
        ] {
            let pattern = format!("{article_suffix}{sep}");
            if let Some(idx) = s.find(&pattern) {
                let prefix = &s[..idx];
                let suffix = &s[idx + pattern.len()..];
                return format!("{article_prefix}{prefix}{sep}{suffix}");
            }
        }
    }

    // Check trailing suffix
    if let Some(stripped) = s.strip_suffix(", The").or_else(|| s.strip_suffix(", the")) {
        return format!("The {}", stripped.trim());
    }
    if let Some(stripped) = s.strip_suffix(", A").or_else(|| s.strip_suffix(", a")) {
        return format!("A {}", stripped.trim());
    }
    if let Some(stripped) = s.strip_suffix(", An").or_else(|| s.strip_suffix(", an")) {
        return format!("An {}", stripped.trim());
    }

    s.to_string()
}

/// Normalizes a raw ROM file or folder name into a clean, human-readable display title.
///
/// Strips dump/region tags like `(USA)`, `(Europe)`, `[!]`, `(Rev 1)`, `(Beta)`,
/// preserves multi-disc markers like `(Disc 1)` as clean suffixes, inverts trailing "The"
/// (`"Legend of Zelda, The"` -> `"The Legend of Zelda"`), replaces underscores with spaces,
/// and strips file extensions.
pub fn normalize_rom_title(filename: &str) -> String {
    let mut title = filename.trim();

    // 1. Strip known compound extensions (e.g., .nkit.gcz, .nkit.iso, .nkit.ciso, .nkit.rvz)
    let nkit_suffixes = [
        ".nkit.gcz", ".nkit.iso", ".nkit.ciso", ".nkit.rvz", ".nkit.wbfs", ".nkit.zip", ".nkit.7z",
        ".NKIT.GCZ", ".NKIT.ISO", ".NKIT.CISO", ".NKIT.RVZ", ".NKIT.WBFS", ".NKIT.ZIP", ".NKIT.7Z"
    ];
    for suffix in &nkit_suffixes {
        if title.ends_with(suffix) {
            title = &title[..title.len() - suffix.len()];
            break;
        }
    }

    // 2. Strip single file extension if present
    if let Some(pos) = title.rfind('.') {
        if pos > 0 {
            let ext = &title[pos..];
            if ext.len() <= 5 && !ext.contains(' ') {
                title = &title[..pos];
            }
        }
    }

    // 3. Strip trailing .nkit if left behind
    if title.to_lowercase().ends_with(".nkit") {
        title = &title[..title.len() - 5];
    }

    // 4. Replace underscores with spaces
    let clean = title.replace('_', " ");

    // 5. Extract and preserve multi-disc tag if present
    let mut disc_tag: Option<String> = None;
    let disc_patterns = ["(Disc 1)", "(Disc 2)", "(Disc 3)", "(Disc 4)", "(Disk 1)", "(Disk 2)", "(CD 1)", "(CD 2)"];
    for pattern in &disc_patterns {
        if clean.to_lowercase().contains(&pattern.to_lowercase()) {
            disc_tag = Some((*pattern).to_string());
            break;
        }
    }

    // 6. Strip goodtools / no-intro / NKit / dump parenthesized tags & bracketed game IDs (e.g. [GM4E01])
    let mut result = String::with_capacity(clean.len());
    let mut in_paren = false;
    let mut in_bracket = false;
    let mut paren_buf = String::new();

    for ch in clean.chars() {
        if ch == '(' {
            in_paren = true;
            paren_buf.clear();
        } else if ch == ')' && in_paren {
            in_paren = false;
            let tag = paren_buf.trim();
            // If it's a disc tag or part number, retain it if we didn't extract it already
            if (tag.to_lowercase().starts_with("disc ") || tag.to_lowercase().starts_with("disk ")) && disc_tag.is_none() {
                disc_tag = Some(format!("({tag})"));
            }
            paren_buf.clear();
        } else if ch == '[' {
            in_bracket = true;
        } else if ch == ']' {
            in_bracket = false;
        } else if in_paren {
            paren_buf.push(ch);
        } else if !in_bracket {
            result.push(ch);
        }
    }

    // 7. Invert trailing or separator-prefixed articles
    let inverted = invert_article(result.trim());

    // 8. Append preserved disc tag if present
    let mut s = inverted;
    if let Some(ref dt) = disc_tag {
        s.push_str(&format!(" {dt}"));
    }

    // 9. Collapse multiple consecutive spaces and trim
    let words: Vec<&str> = s.split_whitespace().collect();
    words.join(" ")
}

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

/// Scans the user-selected ROM directory for games across all scaffolded console systems.
/// Normalizes titles when `normalize` is enabled.
pub fn scan_emulation_roms(roms_dir: &str, normalize: bool) -> AppResult<Vec<PlatformGame>> {
    let base_path = PathBuf::from(roms_dir.trim());
    if !base_path.is_dir() {
        return Ok(Vec::new());
    }

    let mut games = Vec::new();

    for spec in SYSTEM_FOLDER_SPECS {
        let system_dir = base_path.join(spec.id);
        if !system_dir.is_dir() {
            continue;
        }

        // Build list of valid lower-cased extensions for this system
        let valid_extensions: Vec<String> = spec
            .extensions
            .split_whitespace()
            .map(|ext| ext.trim_start_matches('.').to_lowercase())
            .collect();

        if let Ok(entries) = fs::read_dir(&system_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    // Skip system text files, hidden files, and save states
                    if file_name.starts_with('.')
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
    }

    // Sort games primarily by platform/console, then alphabetically by title
    games.sort_by(|a, b| {
        let console_a = a.console.as_deref().unwrap_or("");
        let console_b = b.console.as_deref().unwrap_or("");
        match console_a.cmp(console_b) {
            std::cmp::Ordering::Equal => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            other => other,
        }
    });

    Ok(games)
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
            "The Legend of Zelda - Ocarina of Time"
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
