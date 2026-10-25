//! Elite-specific tweaks: the DLL list in Elite.launchconfig and the LobbyStats settings file.
use std::fs;
use std::path::Path;

pub const LAUNCHCONFIG: &str = "Elite.launchconfig";

/// Add or remove `dll` in every `*.launchconfig` of the game folder (Elite.launchconfig, Elite_Test.launchconfig of the
/// public test, …). Returns the names of the files that changed.
/// Set `key=value` in `[section]` of a simple ini file (created if missing). Keeps everything else as is.
pub fn set_ini_value(path: &Path, section: &str, key: &str, value: &str) -> std::io::Result<()> {
    if let Some(d) = path.parent() { fs::create_dir_all(d)?; }
    let text = fs::read_to_string(path).unwrap_or_default();
    let crlf = text.contains("\r\n") || text.is_empty();
    let mut out: Vec<String> = Vec::new();
    let (mut in_sec, mut done, mut seen_sec) = (false, false, false);
    for l in text.lines() {
        let t = l.trim();
        if t.starts_with('[') && t.ends_with(']') {
            if in_sec && !done { out.push(format!("{key}={value}")); done = true; }
            in_sec = t[1..t.len() - 1].trim().eq_ignore_ascii_case(section);
            seen_sec |= in_sec;
        } else if in_sec && !done {
            if let Some((k, _)) = t.split_once('=') {
                if k.trim().eq_ignore_ascii_case(key) { out.push(format!("{key}={value}")); done = true; continue; }
            }
        }
        out.push(l.to_string());
    }
    if !done {
        if !seen_sec { if !out.is_empty() { out.push(String::new()); } out.push(format!("[{section}]")); }
        else { /* section was the last one */ }
        out.push(format!("{key}={value}"));
    }
    let nl = if crlf { "\r\n" } else { "\n" };
    fs::write(path, out.join(nl) + nl)
}

pub fn set_injected_all(game_dir: &Path, dll: &str, enable: bool) -> std::io::Result<Vec<String>> {
    let mut changed = Vec::new();
    for e in fs::read_dir(game_dir)?.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        if !name.to_ascii_lowercase().ends_with(".launchconfig") || !e.path().is_file() { continue; }
        if set_injected_file(&e.path(), dll, enable)? { changed.push(name); }
    }
    Ok(changed)
}

/// Add or remove `dll` in the `InjectedFiles=` line of Elite.launchconfig. Returns true if the file changed.
pub fn set_injected(game_dir: &Path, dll: &str, enable: bool) -> std::io::Result<bool> {
    set_injected_file(&game_dir.join(LAUNCHCONFIG), dll, enable)
}

fn set_injected_file(path: &Path, dll: &str, enable: bool) -> std::io::Result<bool> {
    let path = path.to_path_buf();
    let text = fs::read_to_string(&path)?;
    let crlf = text.contains("\r\n");
    let mut changed = false;
    let mut found = false;
    let lines: Vec<String> = text
        .lines()
        .map(|l| {
            let Some(rest) = l.strip_prefix("InjectedFiles=") else { return l.to_string() };
            found = true;
            let mut items: Vec<String> = rest.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect();
            let has = items.iter().any(|x| x.eq_ignore_ascii_case(dll));
            if enable && !has {
                items.push(dll.to_string());
                changed = true;
            } else if !enable && has {
                items.retain(|x| !x.eq_ignore_ascii_case(dll));
                changed = true;
            }
            format!("InjectedFiles={}", items.join(", "))
        })
        .collect();
    let mut lines = lines;
    if !found && enable {
        lines.push(format!("InjectedFiles={dll}"));
        changed = true;
    }
    if changed {
        let nl = if crlf { "\r\n" } else { "\n" };
        let mut out = lines.join(nl);
        if text.ends_with('\n') {
            out.push_str(nl);
        }
        fs::write(&path, out)?;
    }
    Ok(changed)
}

/// LobbyStats.ini: keep the player's own lines, drop everything left over from the old host filter
/// (the [Guard], [UI] and [Chat] sections and the "!filter" hint). Creates a minimal file if there is none.
pub fn clean_lobbystats_ini(game_dir: &Path) -> std::io::Result<()> {
    let dir = game_dir.join("Elite_DLLs");
    let path = dir.join("LobbyStats.ini");
    let old = fs::read_to_string(&path).unwrap_or_default();
    let mut out = String::new();
    let mut skip = false;
    for line in old.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            let sec = t.to_ascii_lowercase();
            skip = sec == "[guard]" || sec == "[ui]" || sec == "[chat]";
        }
        if skip || t.contains("!filter") { continue; }
        out.push_str(line);
        out.push_str("\r\n");
    }
    if !out.to_ascii_lowercase().contains("[lobbystats]") {
        out = format!("[LobbyStats]\r\nShowRatingInName=1\r\n{out}");
    }
    if out != old {
        fs::create_dir_all(&dir)?;
        fs::write(&path, out)?;
    }
    Ok(())
}
