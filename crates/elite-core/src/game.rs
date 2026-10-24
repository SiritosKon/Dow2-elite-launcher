//! Finding Dawn of War II: Retribution and Steam on Windows, Linux and macOS.
use std::path::{Path, PathBuf};

pub fn is_game_dir(dir: &Path, marker: &str) -> bool {
    dir.join(marker).is_file()
}

/// Steam installation folders that exist on this machine.
pub fn steam_roots() -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = Vec::new();
    #[cfg(windows)]
    {
        use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
        use winreg::RegKey;
        if let Ok(k) = RegKey::predef(HKEY_CURRENT_USER).open_subkey("Software\\Valve\\Steam") {
            if let Ok(p) = k.get_value::<String, _>("SteamPath") {
                v.push(PathBuf::from(p.replace('/', "\\")));
            }
        }
        for key in ["SOFTWARE\\WOW6432Node\\Valve\\Steam", "SOFTWARE\\Valve\\Steam"] {
            if let Ok(k) = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(key) {
                if let Ok(p) = k.get_value::<String, _>("InstallPath") {
                    v.push(PathBuf::from(p));
                }
            }
        }
        v.push(PathBuf::from("C:\\Program Files (x86)\\Steam"));
    }
    if let Some(home) = dirs::home_dir() {
        #[cfg(target_os = "linux")]
        for p in [".steam/steam", ".steam/root", ".local/share/Steam", ".var/app/com.valvesoftware.Steam/.local/share/Steam"] {
            v.push(home.join(p));
        }
        #[cfg(target_os = "macos")]
        v.push(home.join("Library/Application Support/Steam"));
        let _ = &home;
    }
    let mut out: Vec<PathBuf> = Vec::new();
    for p in v {
        if p.join("steamapps").is_dir() {
            let c = p.canonicalize().unwrap_or(p);
            if !out.contains(&c) {
                out.push(c);
            }
        }
    }
    out
}

/// All Steam library folders (the main one + those listed in libraryfolders.vdf).
pub fn steam_libraries() -> Vec<PathBuf> {
    let mut libs = Vec::new();
    for root in steam_roots() {
        if !libs.contains(&root) {
            libs.push(root.clone());
        }
        let vdf = root.join("steamapps").join("libraryfolders.vdf");
        if let Ok(t) = std::fs::read_to_string(vdf) {
            for line in t.lines() {
                let l = line.trim();
                if let Some(rest) = l.strip_prefix("\"path\"") {
                    let p = rest.trim().trim_matches('"').replace("\\\\", "\\");
                    let pb = PathBuf::from(p);
                    if pb.join("steamapps").is_dir() && !libs.contains(&pb) {
                        libs.push(pb);
                    }
                }
            }
        }
    }
    libs
}

pub fn find_game(folder: &str, marker: &str) -> Option<PathBuf> {
    steam_libraries()
        .into_iter()
        .map(|l| l.join("steamapps").join("common").join(folder))
        .find(|d| is_game_dir(d, marker))
}

pub fn steam_installed() -> bool {
    !steam_roots().is_empty()
}
