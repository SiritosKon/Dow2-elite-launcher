//! Elite Launcher — window around elite-core.
//!
//! Rules that shape this file:
//! * the game can always be started: a new version is only *offered*, never forced;
//! * everything the Elite team may want to change (versions, news, announcements, links,
//!   mirrors, launch commands) comes from the signed launcher.json, not from this code;
//! * the launcher works offline with the last verified config.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use elite_core::config::{version_newer, LauncherConfig};
use elite_core::{elite, game, net::Mirrors, update, LAUNCHER_VERSION};
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

/// Mirrors compiled into the launcher; launcher.json can add more. Order = preference.
/// Use hosts from different providers/domains so the launcher can start even if one is blocked.
/// Built-in mirrors, set at build time: `ELITE_MIRRORS="https://a/;https://b/" cargo build`.
/// Not stored in the repository; more mirrors can come later from the signed launcher.json.
const BUILTIN_MIRRORS: &str = match option_env!("ELITE_MIRRORS") { Some(v) => v, None => "" };
const DEFAULT_CACHE_MB: u64 = 6144;

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct Settings {
    game_dir: Option<PathBuf>,
    channel: Option<String>,
    /// "" = system language
    lang: String,
    extra_mirror: String,
    cache_mb: Option<u64>,
    dismissed: Vec<String>,
    /// Component id → ticked by the player (missing = component default).
    components: BTreeMap<String, bool>,
    /// "component.option" → ticked by the player (missing = option default).
    comp_options: BTreeMap<String, bool>,
    /// Channel the player switched away from — the quick "back to …" button.
    prev_channel: Option<String>,
    /// Mirror that worked last time — tried first (R2 is blocked in some countries, our server in others).
    last_mirror: String,
    /// Anonymous usage count (see usage_ping). None = on.
    usage_stats: Option<bool>,
    /// Random secret for the daily usage hash; never sent itself.
    ping_secret: String,
    /// Day number (UTC days since 1970) of the last successful usage ping.
    ping_day: u64,
}

struct App {
    settings: Mutex<Settings>,
    config: Mutex<Option<LauncherConfig>>,
    busy: AtomicBool,
    cancel: Mutex<Option<Arc<AtomicBool>>>,
    last_error: Mutex<String>,
    /// Version id → bytes a switch to it would download (0 = instant). Filled in the background.
    costs: Mutex<BTreeMap<String, u64>>,
}

/// Work out how much each version would download (manifests are fetched once and cached for good).
fn compute_costs(app: &AppHandle) {
    let st = app.state::<App>();
    let Some(cfg) = st.config.lock().unwrap().clone() else { return };
    let Some(dir) = game_dir(&st) else { return };
    let s = st.settings.lock().unwrap().clone();
    let m = mirrors(&s, Some(&cfg));
    let g = update::Game::new(&dir);
    let mut out = BTreeMap::new();
    for c in &cfg.channels {
        if let Ok(man) = update::load_manifest(&g, &m, &c.version) {
            out.insert(c.version.clone(), update::switch_cost(&g, &man));
        }
    }
    *st.costs.lock().unwrap() = out;
}

fn data_dir() -> PathBuf {
    let d = dirs::config_dir().unwrap_or_else(std::env::temp_dir).join("EliteLauncher");
    let _ = std::fs::create_dir_all(&d);
    d
}

fn load_settings() -> Settings {
    std::fs::read(data_dir().join("settings.json")).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default()
}

/// Remember the mirror that just worked (tried first next time).
fn remember_mirror(st: &App, worked: String) {
    if worked.is_empty() { return; }
    let mut s = st.settings.lock().unwrap();
    if s.last_mirror != worked { s.last_mirror = worked; save_settings(&s); }
}

fn save_settings(s: &Settings) {
    let _ = std::fs::write(data_dir().join("settings.json"), serde_json::to_vec_pretty(s).unwrap());
}

fn cached_config() -> Option<LauncherConfig> {
    let raw = std::fs::read(data_dir().join("launcher.json")).ok()?;
    let sig = std::fs::read_to_string(data_dir().join("launcher.json.sig")).ok()?;
    update::parse_signed_config(&raw, &sig)
}

fn os_name() -> &'static str {
    if cfg!(windows) {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "linux"
    }
}

fn mirrors(s: &Settings, cfg: Option<&LauncherConfig>) -> Mirrors {
    let mut v: Vec<String> = Vec::new();
    if !s.extra_mirror.trim().is_empty() {
        v.push(s.extra_mirror.trim().to_string());
    }
    // зеркала из подписанного launcher.json — первыми (так команда меняет порядок без нового лаунчера), потом встроенные
    if let Some(c) = cfg {
        v.extend(c.mirrors.iter().cloned());
    }
    v.extend(BUILTIN_MIRRORS.split(';').map(str::trim).filter(|x| !x.is_empty()).map(String::from));
    if !s.last_mirror.is_empty() && v.iter().any(|m| m == &s.last_mirror) {
        v.retain(|m| m != &s.last_mirror);
        v.insert(0, s.last_mirror.clone());
    }
    let mut seen = std::collections::HashSet::new();
    v.retain(|m| seen.insert(m.trim_end_matches('/').to_ascii_lowercase()));
    Mirrors::new(v)
}

fn game_dir(app: &App) -> Option<PathBuf> {
    let cfg = app.config.lock().unwrap().clone().unwrap_or_default();
    let mut s = app.settings.lock().unwrap();
    if let Some(d) = &s.game_dir {
        if game::is_game_dir(d, &cfg.game.marker) {
            return Some(d.clone());
        }
    }
    let found = game::find_game(&cfg.game.folder, &cfg.game.marker);
    if found.is_some() {
        s.game_dir = found.clone();
        save_settings(&s);
    }
    found
}

fn current_channel(app: &App, cfg: &LauncherConfig, installed: &update::State) -> String {
    let s = app.settings.lock().unwrap();
    let want = s.channel.clone().filter(|c| cfg.channels.iter().any(|x| &x.id == c));
    want.or_else(|| Some(installed.channel.clone()).filter(|c| cfg.channels.iter().any(|x| &x.id == c)))
        .or_else(|| cfg.channels.iter().find(|c| c.default).map(|c| c.id.clone()))
        .or_else(|| cfg.channels.first().map(|c| c.id.clone()))
        .unwrap_or_default()
}

/// Everything the window needs to draw itself.
#[tauri::command]
fn get_status(app: State<'_, App>) -> Value {
    let cfg = app.config.lock().unwrap().clone();
    let dir = game_dir(&app);
    let installed = dir.as_ref().map(|d| update::Game::new(d).load_state()).unwrap_or_default();
    let cfgd = cfg.clone().unwrap_or_default();
    let channel = current_channel(&app, &cfgd, &installed);
    let target = cfgd.channels.iter().find(|c| c.id == channel).cloned();
    let s = app.settings.lock().unwrap().clone();
    let launcher_update = cfgd.launcher_update.as_ref().filter(|u| version_newer(&u.version, LAUNCHER_VERSION)).map(|u| {
        json!({"version": u.version, "text": u.text, "url": u.urls.get(os_name()).or_else(|| u.urls.get("all")).cloned().unwrap_or_default()})
    });
    json!({
        "launcher_version": LAUNCHER_VERSION,
        "os": os_name(),
        "lang": s.lang,
        "config": cfg,
        "config_loaded": cfg.is_some(),
        "steam_found": game::steam_installed(),
        "game_dir": dir,
        "game_found": dir.is_some(),
        "installed_version": installed.version,
        "installed_channel": installed.channel,
        "channel": channel,
        "target_version": target.as_ref().map(|t| t.version.clone()).unwrap_or_default(),
        "update_available": !installed.version.is_empty() && target.as_ref().is_some_and(|t| t.version != installed.version),
        "launcher_update": launcher_update,
        "busy": app.busy.load(Ordering::Relaxed),
        "last_error": app.last_error.lock().unwrap().clone(),
        "cache_mb": s.cache_mb.unwrap_or(DEFAULT_CACHE_MB),
        "extra_mirror": s.extra_mirror,
        "dismissed": s.dismissed,
        "mirror_count": mirrors(&s, cfg.as_ref()).count(),
        "components": cfgd.components.iter().map(|c| json!({
            "id": c.id, "name": c.name, "description": c.description, "latest": c.version,
            "enabled": s.components.get(&c.id).copied().unwrap_or(c.default),
            "conflicts": dir.as_ref().map(|d| found_conflicts(d, c)).unwrap_or_default(),
            "installed": dir.as_ref().map(|d| update::Game::component(d, &c.id).load_state().version).unwrap_or_default(),
            "options": c.options.iter().map(|o| json!({
                "id": o.id, "name": o.name,
                "on": s.comp_options.get(&format!("{}.{}", c.id, o.id)).copied().unwrap_or(o.default),
            })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
        "usage_stats": s.usage_stats.unwrap_or(true),
        "costs": app.costs.lock().unwrap().clone(),
        "prev_channel": s.prev_channel.filter(|p| p != &channel && cfgd.channels.iter().any(|c| &c.id == p)),
        "main_channel": cfgd.channels.iter().find(|c| c.default).map(|c| c.id.clone()).unwrap_or_default(),
    })
}

/// Fetch the newest launcher.json (falls back to the cached one when offline).
#[tauri::command]
async fn refresh(app: AppHandle) -> Result<Value, String> {
    let st = app.state::<App>();
    let s = st.settings.lock().unwrap().clone();
    let cached = st.config.lock().unwrap().clone();
    let res = tauri::async_runtime::spawn_blocking(move || {
        let m = mirrors(&s, cached.as_ref());
        update::fetch_config(&m).map(|r| (r, m.current_base()))
    })
    .await
    .map_err(|e| e.to_string())?;
    match res {
        Ok(((cfg, raw, sig), worked)) => {
            remember_mirror(&st, worked);
            let _ = std::fs::write(data_dir().join("launcher.json"), raw);
            let _ = std::fs::write(data_dir().join("launcher.json.sig"), sig);
            *st.config.lock().unwrap() = Some(cfg);
            st.last_error.lock().unwrap().clear();
        }
        Err(e) => *st.last_error.lock().unwrap() = format!("offline: {e}"),
    }
    let a2 = app.clone();
    let _ = tauri::async_runtime::spawn_blocking(move || compute_costs(&a2)).await;
    Ok(get_status(app.state::<App>()))
}

#[tauri::command]
fn set_channel(app: State<'_, App>, id: String) {
    let mut s = app.settings.lock().unwrap();
    s.channel = Some(id);
    save_settings(&s);
}

/// One click: remember where we came from, pick the version and install it right away.
#[tauri::command]
fn switch_to(app: AppHandle, id: String) -> Result<(), String> {
    {
        let st = app.state::<App>();
        if st.busy.load(Ordering::Relaxed) {
            return Err("already running".into());
        }
        let cfg = st.config.lock().unwrap().clone().unwrap_or_default();
        let installed = game_dir(&st).map(|d| update::Game::new(d).load_state()).unwrap_or_default();
        let cur = current_channel(&st, &cfg, &installed);
        let mut s = st.settings.lock().unwrap();
        if !installed.version.is_empty() && cur != id {
            s.prev_channel = Some(cur);
        }
        s.channel = Some(id);
        save_settings(&s);
    }
    start_update(app, false, None)
}

#[tauri::command]
fn set_setting(app: State<'_, App>, key: String, value: Value) {
    let mut s = app.settings.lock().unwrap();
    match key.as_str() {
        "lang" => s.lang = value.as_str().unwrap_or("").to_string(),
        "extra_mirror" => s.extra_mirror = value.as_str().unwrap_or("").to_string(),
        "cache_mb" => s.cache_mb = value.as_u64(),
        "usage_stats" => s.usage_stats = value.as_bool(),
        "dismiss" => {
            if let Some(id) = value.as_str() {
                s.dismissed.push(id.to_string());
            }
        }
        _ => {}
    }
    save_settings(&s);
}

#[tauri::command]
fn set_component(app: State<'_, App>, id: String, on: bool) {
    let mut s = app.settings.lock().unwrap();
    s.components.insert(id, on);
    save_settings(&s);
}

/// Write every component option to its ini file (only for installed components).
fn apply_options(dir: &std::path::Path, cfg: &LauncherConfig, s: &Settings) {
    for c in &cfg.components {
        if update::Game::component(dir, &c.id).load_state().version.is_empty() { continue; }
        for o in &c.options {
            if o.ini.is_empty() || o.ini.contains("..") { continue; }
            let on = s.comp_options.get(&format!("{}.{}", c.id, o.id)).copied().unwrap_or(o.default);
            let _ = elite::set_ini_value(&dir.join(&o.ini), &o.section, &o.key, if on { "1" } else { "0" });
        }
    }
}

/// Tick/untick an option of an add-on: saved and written to the game folder right away (takes effect on next game start).
#[tauri::command]
fn set_comp_option(app: State<'_, App>, comp: String, opt: String, on: bool) {
    let s = {
        let mut s = app.settings.lock().unwrap();
        s.comp_options.insert(format!("{comp}.{opt}"), on);
        save_settings(&s);
        s.clone()
    };
    if let (Some(dir), Some(cfg)) = (game_dir(&app), app.config.lock().unwrap().clone()) {
        apply_options(&dir, &cfg, &s);
    }
}

/// Hand-installed copies of a component that are in the game folder right now (see Component::conflicts).
fn found_conflicts(dir: &std::path::Path, c: &elite_core::config::Component) -> Vec<String> {
    c.conflicts.iter().filter(|f| !f.contains("..") && update::Game::new(dir).file(f).is_file()).cloned().collect()
}

/// Component wanted by the player and not replaced by a hand-installed copy.
fn comp_on(dir: &std::path::Path, c: &elite_core::config::Component, s: &Settings) -> bool {
    s.components.get(&c.id).copied().unwrap_or(c.default) && found_conflicts(dir, c).is_empty()
}

/// Turn off launcher copies of components that now have a hand-installed copy (checked before every game start).
fn drop_conflicting(dir: &std::path::Path, cfg: &LauncherConfig) {
    let main = update::Game::new(dir);
    for c in &cfg.components {
        if found_conflicts(dir, c).is_empty() { continue; }
        let g = update::Game::component(dir, &c.id);
        if !g.load_state().version.is_empty() { let _ = g.uninstall(); }
        if !c.inject.is_empty() {
            if let Ok(names) = elite::set_injected_all(dir, &c.inject, false) {
                for name in names { main.accept_local_edit(&name); }
            }
        }
    }
}

/// Delete the hand-installed copy of a component: its files, their .ini next to them, and the stand-alone
/// package's loader (winmm.dll + readmes) once its dow2-patches folder has no DLLs left.
#[tauri::command]
fn remove_conflicts(app: State<'_, App>, id: String) -> Result<Vec<String>, String> {
    let dir = game_dir(&app).ok_or("game folder not found")?;
    let cfg = app.config.lock().unwrap().clone().unwrap_or_default();
    let c = cfg.components.iter().find(|c| c.id == id).ok_or("unknown add-on")?;
    let g = update::Game::new(&dir);
    let mut removed = Vec::new();
    for f in found_conflicts(&dir, c) {
        let p = g.file(&f);
        std::fs::remove_file(&p).map_err(|e| format!("{f}: {e}"))?;
        removed.push(f.clone());
        let ini = p.with_extension("ini");
        if ini.is_file() && std::fs::remove_file(&ini).is_ok() {
            removed.push(f.rsplit_once('.').map(|(a, _)| format!("{a}.ini")).unwrap_or_default());
        }
    }
    let pd = dir.join("dow2-patches");
    let dlls_left = std::fs::read_dir(&pd).map(|rd| rd.flatten().any(|e| e.file_name().to_string_lossy().to_ascii_lowercase().ends_with(".dll"))).unwrap_or(false);
    if !dlls_left {
        // the loader names its folder in UTF-16, the readmes in plain text
        let wide: Vec<u8> = "dow2-patches".encode_utf16().flat_map(|u| u.to_le_bytes()).collect();
        let ours = |name: &str| std::fs::read(dir.join(name))
            .map(|b| b.windows(12).any(|w| w == b"dow2-patches") || b.windows(wide.len()).any(|w| w == wide.as_slice()))
            .unwrap_or(false);
        for name in ["winmm.dll", "README_EN.txt", "README_RU.txt"] {
            if ours(name) && std::fs::remove_file(dir.join(name)).is_ok() { removed.push(name.to_string()); }
        }
        if pd.is_dir() && std::fs::read_dir(&pd).map(|mut rd| rd.next().is_none()).unwrap_or(false) {
            let _ = std::fs::remove_dir(&pd);
        }
    }
    Ok(removed)
}

/// Install ticked components, remove unticked ones, keep Elite.launchconfig in sync.
fn sync_components(dir: &std::path::Path, cfg: &LauncherConfig, s: &Settings, m: &Mirrors, progress: &mut dyn FnMut(&update::Progress)) -> Result<(), update::Error> {
    let main = update::Game::new(dir);
    if main.load_state().version.is_empty() {
        return Ok(()); // Elite itself is not installed yet
    }
    for c in &cfg.components {
        let on = comp_on(dir, c, s);
        let g = update::Game::component(dir, &c.id);
        if on {
            let man = update::load_manifest(&g, m, &c.version)?;
            update::apply(&g, m, &man, &update::Options { verify_all: false, cache_limit: s.cache_mb.unwrap_or(DEFAULT_CACHE_MB) << 20, channel: c.id.clone() }, progress)?;
            if c.id == "lobbystats" {
                elite::clean_lobbystats_ini(dir).map_err(|e| update::Error::Io(e.to_string()))?;
            }
        } else if !g.load_state().version.is_empty() {
            g.uninstall()?;
        }
        if !c.inject.is_empty() {
            for name in elite::set_injected_all(dir, &c.inject, on).map_err(|e| update::Error::Io(e.to_string()))? {
                main.accept_local_edit(&name);
            }
        }
    }
    apply_options(dir, cfg, s);
    Ok(())
}

#[tauri::command]
async fn pick_game_dir(app: AppHandle) -> Result<Value, String> {
    let picked = app.dialog().file().blocking_pick_folder();
    let st = app.state::<App>();
    if let Some(p) = picked.and_then(|p| p.into_path().ok()) {
        let marker = st.config.lock().unwrap().clone().unwrap_or_default().game.marker;
        if !game::is_game_dir(&p, &marker) {
            return Err(format!("{marker} not found in this folder"));
        }
        let mut s = st.settings.lock().unwrap();
        s.game_dir = Some(p);
        save_settings(&s);
    }
    Ok(get_status(app.state::<App>()))
}

#[tauri::command]
fn open_url(app: AppHandle, url: String) -> Result<(), String> {
    let ok = url.starts_with("https://") || url.starts_with("http://") || url.starts_with("steam://");
    if !ok {
        return Err("unsupported link".into());
    }
    app.opener().open_url(url, None::<&str>).map_err(|e| e.to_string())
}

#[tauri::command]
fn open_game_dir(app: State<'_, App>, handle: AppHandle) -> Result<(), String> {
    let d = game_dir(&app).ok_or("game folder not found")?;
    handle.opener().open_path(d.to_string_lossy(), None::<&str>).map_err(|e| e.to_string())
}

/// Install / update / switch version / repair in a background thread; progress goes to the window as events.
#[tauri::command]
fn start_update(app: AppHandle, repair: bool, components_only: Option<bool>) -> Result<(), String> {
    let components_only = components_only.unwrap_or(false);
    let st = app.state::<App>();
    if st.busy.swap(true, Ordering::SeqCst) {
        return Err("already running".into());
    }
    let cfg = st.config.lock().unwrap().clone();
    let Some(cfg) = cfg else {
        st.busy.store(false, Ordering::SeqCst);
        return Err("no connection to the update server yet".into());
    };
    let Some(dir) = game_dir(&st) else {
        st.busy.store(false, Ordering::SeqCst);
        return Err("game folder not found".into());
    };
    let g = update::Game::new(&dir);
    let channel = current_channel(&st, &cfg, &g.load_state());
    let version = cfg.channels.iter().find(|c| c.id == channel).map(|c| c.version.clone()).unwrap_or_default();
    let s = st.settings.lock().unwrap().clone();
    let m = mirrors(&s, Some(&cfg));
    *st.cancel.lock().unwrap() = Some(m.cancel.clone());
    st.last_error.lock().unwrap().clear();
    let app2 = app.clone();
    std::thread::spawn(move || {
        let mut last = Instant::now() - Duration::from_secs(1);
        let mut emit = |p: &update::Progress| {
            if p.phase == "done" || last.elapsed() > Duration::from_millis(120) {
                let _ = app2.emit("progress", p);
                last = Instant::now();
            }
        };
        let r = (if components_only {
            Ok(update::Summary::default())
        } else {
            update::load_manifest(&g, &m, &version).and_then(|man| {
                update::apply(&g, &m, &man, &update::Options { verify_all: repair, cache_limit: s.cache_mb.unwrap_or(DEFAULT_CACHE_MB) << 20, channel: channel.clone() }, &mut emit)
            })
        })
        .and_then(|sum| sync_components(&dir, &cfg, &s, &m, &mut emit).map(|_| sum));
        compute_costs(&app2);
        let st = app2.state::<App>();
        if r.is_ok() { remember_mirror(&st, m.current_base()); }
        st.busy.store(false, Ordering::SeqCst);
        let payload = match r {
            Ok(sum) => json!({"ok": true, "summary": sum}),
            Err(e) => {
                *st.last_error.lock().unwrap() = e.to_string();
                json!({"ok": false, "error": e.to_string(), "cancelled": matches!(e, update::Error::Cancelled)})
            }
        };
        let _ = app2.emit("update_done", payload);
    });
    Ok(())
}

#[tauri::command]
fn cancel_update(app: State<'_, App>) {
    if let Some(c) = app.cancel.lock().unwrap().as_ref() {
        c.store(true, Ordering::SeqCst);
    }
}

/// «Отправить логи»: логи наших DLL (Elite_DLLs) и свежие логи игры из «Документов» уходят на сервер,
/// оттуда — разработчику. Каждый файл обрезается до последних 3 МБ.
#[tauri::command]
async fn send_logs(app: AppHandle, who: String) -> Result<String, String> {
    let st = app.state::<App>();
    let dir = game_dir(&st);
    let installed = dir.as_ref().map(|d| update::Game::new(d).load_state()).unwrap_or_default();
    let comps: Vec<String> = st.config.lock().unwrap().clone().unwrap_or_default().components.iter().map(|c| c.version.clone()).collect();
    tauri::async_runtime::spawn_blocking(move || {
        let mut out = format!("Elite Launcher {} · {} · Elite {} ({}) · add-ons {:?}\ngame: {}\n\n",
            LAUNCHER_VERSION, os_name(), installed.version, installed.channel, comps,
            dir.as_ref().map(|d| d.display().to_string()).unwrap_or_default());
        // всё, что может помочь понять краш: логи дополнений, логи игры, дампы, отчёты Windows об ошибках
        let mut files: Vec<PathBuf> = Vec::new();
        let recent = |p: &std::path::Path, days: u64| std::fs::metadata(p).and_then(|m| m.modified()).ok()
            .and_then(|m| m.elapsed().ok()).map(|e| e.as_secs() < days * 86400).unwrap_or(false);
        let ext_of = |p: &std::path::Path| p.extension().and_then(|x| x.to_str()).unwrap_or("").to_ascii_lowercase();
        // файлы каталога (с подкаталогами до depth), новые сначала
        fn walk(root: &std::path::Path, depth: u32, out: &mut Vec<(std::time::SystemTime, PathBuf)>) {
            let Ok(rd) = std::fs::read_dir(root) else { return };
            for e in rd.flatten() {
                let p = e.path(); let Ok(md) = e.metadata() else { continue };
                if md.is_dir() { if depth > 0 { walk(&p, depth - 1, out); } continue; }
                out.push((md.modified().unwrap_or(std::time::UNIX_EPOCH), p));
            }
        }
        let mut take = |root: PathBuf, depth: u32, exts: &[&str], days: u64, max: usize, files: &mut Vec<PathBuf>| {
            let mut v = Vec::new(); walk(&root, depth, &mut v);
            v.retain(|(_, p)| exts.contains(&ext_of(p).as_str()) && recent(p, days));
            v.sort_by(|a, b| b.0.cmp(&a.0));
            files.extend(v.into_iter().take(max).map(|x| x.1));
        };
        if let Some(d) = &dir {
            take(d.join("Elite_DLLs"), 0, &["log", "ini", "txt", "dmp"], 30, 40, &mut files);
            take(d.clone(), 0, &["log", "txt", "dmp", "mdmp", "launchconfig"], 7, 20, &mut files);
        }
        if let Some(doc) = dirs::document_dir() {
            take(doc.join("My Games").join("Dawn of War II - Retribution"), 3, &["txt", "log", "dmp", "mdmp", "xml", "ini"], 3, 40, &mut files);
        }
        // дампы и отчёты Windows (WER) о падениях игры
        let is_game = |p: &std::path::Path| { let s = p.to_string_lossy().to_ascii_lowercase(); s.contains("dow2") || s.contains("elite") };
        let mut wer_roots: Vec<PathBuf> = Vec::new();
        if let Some(la) = dirs::data_local_dir() {
            let mut v = Vec::new(); walk(&la.join("CrashDumps"), 0, &mut v);
            v.retain(|(_, p)| is_game(p) && recent(p, 7)); v.sort_by(|a, b| b.0.cmp(&a.0));
            files.extend(v.into_iter().take(2).map(|x| x.1));
            wer_roots.push(la.join("Microsoft").join("Windows").join("WER"));
        }
        wer_roots.push(PathBuf::from(std::env::var("ProgramData").unwrap_or_else(|_| "C:\\ProgramData".into())).join("Microsoft").join("Windows").join("WER"));
        for w in wer_roots {
            for q in ["ReportArchive", "ReportQueue"] {
                let mut v = Vec::new(); walk(&w.join(q), 1, &mut v);
                v.retain(|(_, p)| is_game(p.parent().unwrap_or(p)) && recent(p, 7) && ["wer", "txt", "xml", "csv"].contains(&ext_of(p).as_str()));
                v.sort_by(|a, b| b.0.cmp(&a.0));
                files.extend(v.into_iter().take(12).map(|x| x.1));
            }
        }
        let mut n = 0; let mut total = 0usize;
        for f in files {
            let Ok(b) = std::fs::read(&f) else { continue };
            if total > 30 << 20 { out += &format!("=== {} ({} bytes) — skipped, upload is full ===\n", f.display(), b.len()); continue; }
            let ext = ext_of(&f);
            if ext == "dmp" || ext == "mdmp" {
                use base64::Engine;
                if b.len() > 12 << 20 { out += &format!("=== {} ({} bytes) — dump too big, skipped ===\n", f.display(), b.len()); continue; }
                out += &format!("=== {} ({} bytes, base64) ===\n", f.display(), b.len());
                out += &base64::engine::general_purpose::STANDARD.encode(&b);
                total += b.len() * 4 / 3;
            } else {
                // отчёты Windows бывают в UTF-16
                let text = if b.len() >= 2 && b[0] == 0xFF && b[1] == 0xFE {
                    let u: Vec<u16> = b[2..].chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
                    String::from_utf16_lossy(&u)
                } else { String::from_utf8_lossy(&b).into_owned() };
                let tail = if text.len() > 4 << 20 { let mut k = text.len() - (4 << 20); while !text.is_char_boundary(k) { k += 1; } &text[k..] } else { &text[..] };
                out += &format!("=== {} ({} bytes) ===\n", f.display(), b.len());
                out += tail; total += tail.len();
            }
            out += "\n\n"; n += 1;
        }
        // журнал Windows: падения игры (Application Error / WER)
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            let q = "*[System[(EventID=1000 or EventID=1001 or EventID=1002 or EventID=1026) and TimeCreated[timediff(@SystemTime) <= 604800000]]]";
            if let Ok(o) = std::process::Command::new("wevtutil").args(["qe", "Application", "/c:60", "/rd:true", "/f:text", &format!("/q:{q}")])
                .creation_flags(0x0800_0000).output() {
                let t = String::from_utf8_lossy(&o.stdout);
                let ev: Vec<&str> = t.split("Event[").filter(|e| { let l = e.to_ascii_lowercase(); l.contains("dow2") || l.contains("elite") }).collect();
                out += &format!("=== Windows event log: game crashes, last 7 days ({} events) ===\n", ev.len());
                for e in ev { out += "Event["; out += e; }
                out += "\n\n"; n += 1;
            }
        }
        if n == 0 { return Err("no log files found yet — play once with the add-ons on, then send".into()); }
        let who: String = who.chars().filter(|c| c.is_alphanumeric() || "._-".contains(*c)).take(32).collect();
        let who = if who.is_empty() { "anon".to_string() } else { who };
        let mut last = String::new();
        // куда слать логи — задаётся при сборке: ELITE_LOG_HOSTS="https://a;https://b" (в репозитории адресов нет)
        let hosts: Vec<&str> = option_env!("ELITE_LOG_HOSTS").unwrap_or("").split(';').map(str::trim).filter(|x| !x.is_empty()).collect();
        if hosts.is_empty() { return Err("log upload is not configured in this build".into()); }
        for host in hosts {
            match ureq::post(&format!("{host}/stats/api/logs?who={who}")).set("Content-Type", "text/plain; charset=utf-8").send_bytes(out.as_bytes()) {
                Ok(r) => {
                    let v: Value = serde_json::from_str(&r.into_string().unwrap_or_default()).unwrap_or_default();
                    return Ok(format!("{} ({} files)", v["id"].as_str().unwrap_or("ok"), n));
                }
                Err(e) => last = e.to_string(),
            }
        }
        Err(format!("could not send logs: {last}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Start the game. Never blocked by a pending update.
#[tauri::command]
fn play(app: State<'_, App>) -> Result<(), String> {
    if app.busy.load(Ordering::Relaxed) {
        return Err("files are being updated, wait a few seconds".into());
    }
    let dir = game_dir(&app).ok_or("game folder not found")?;
    let cfg = app.config.lock().unwrap().clone().unwrap_or_default();
    drop_conflicting(&dir, &cfg);
    let mut cmd = match os_name() {
        "windows" => cfg.launch.windows.clone(),
        "macos" => cfg.launch.macos.clone(),
        _ => cfg.launch.linux.clone(),
    };
    // у версии может быть свой exe (публичный тест 3.2 — отдельный мод Elite_Test рядом с основным)
    let installed = update::Game::new(&dir).load_state();
    if let Some(ch) = cfg.channels.iter().find(|c| c.id == installed.channel && c.version == installed.version) {
        if !ch.exe.is_empty() && os_name() == "windows" {
            cmd.program = format!("{{game}}/{}", ch.exe);
        }
        // старые версии (2.3, 2.9.3) — без своего Elite.exe, запускаются как мод: DOW2.exe -modname Elite
        if !ch.args.is_empty() && os_name() == "windows" {
            cmd.args = ch.args.clone();
        }
    }
    // путь к игре без «длинного» префикса \\?\ (canonicalize на Windows его добавляет, а с ним Windows не принимает «/»)
    let game = dir.to_string_lossy().to_string();
    let game = game.strip_prefix(r"\\?\").unwrap_or(&game).to_string();
    let sub = |x: &str| {
        let v = x.replace("{game}", &game);
        if cfg!(windows) { v.replace('/', "\\") } else { v }
    };
    let program = sub(&cmd.program);
    let mut c = std::process::Command::new(&program);
    c.args(cmd.args.iter().map(|a| sub(a))).current_dir(&game);
    match c.spawn() {
        Ok(_) => Ok(()),
        // 740 = программа требует прав администратора (Elite.exe так помечен). Обычный запуск такого не умеет,
        // а оболочка Windows умеет: `start` показывает обычный запрос UAC и задаёт рабочую папку.
        #[cfg(windows)]
        Err(e) if e.raw_os_error() == Some(740) => {
            use std::os::windows::process::CommandExt;
            let mut sh = std::process::Command::new("cmd");
            sh.args(["/C", "start", "", "/D", &game, &program]).args(cmd.args.iter().map(|a| sub(a)));
            sh.creation_flags(0x0800_0000); // CREATE_NO_WINDOW — без мигающей консоли
            sh.spawn().map(|_| ()).map_err(|e2| format!("cannot start {program} as administrator: {e2}"))
        }
        Err(e) => Err(format!("cannot start {program}: {e}")),
    }
}

/// Anonymous usage count: at most one request per day, sent to the log hosts, carrying only
/// sha256(random local secret + day number) cut to 128 bits, the launcher version, the OS and the installed Elite version.
/// The hash changes every day, so even the server can't tell whether two days came from the same install.
/// No hardware, Steam or account data, no game folder, no names. Off in Settings.
fn usage_ping(app: &AppHandle) {
    use sha2::{Digest, Sha256};
    let st = app.state::<App>();
    let day = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() / 86400).unwrap_or(0);
    let secret = {
        let mut s = st.settings.lock().unwrap();
        if !s.usage_stats.unwrap_or(true) || s.ping_day == day || day == 0 { return; }
        if s.ping_secret.len() != 64 {
            let mut b = [0u8; 32];
            if getrandom::getrandom(&mut b).is_err() { return; }
            s.ping_secret = b.iter().map(|x| format!("{x:02x}")).collect();
            save_settings(&s);
        }
        s.ping_secret.clone()
    };
    let mut h = Sha256::new();
    h.update(secret.as_bytes());
    h.update(day.to_le_bytes());
    let hash: String = h.finalize().iter().take(16).map(|x| format!("{x:02x}")).collect();
    let elite = game_dir(&st).map(|d| update::Game::new(d).load_state().version).filter(|v| !v.is_empty()).unwrap_or_else(|| "none".into());
    let elite: String = elite.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '.' || *c == '-').take(16).collect();
    let hosts: Vec<&str> = option_env!("ELITE_LOG_HOSTS").unwrap_or("").split(';').map(str::trim).filter(|x| !x.is_empty()).collect();
    for host in hosts {
        let url = format!("{host}/stats/api/ping?h={hash}&v={LAUNCHER_VERSION}&os={}&elite={elite}", os_name());
        if ureq::get(&url).timeout(Duration::from_secs(15)).call().is_ok() {
            let mut s = st.settings.lock().unwrap();
            s.ping_day = day;
            save_settings(&s);
            return;
        }
    }
}

fn main() {
    let settings = load_settings();
    let config = cached_config();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(App {
            settings: Mutex::new(settings),
            config: Mutex::new(config),
            busy: AtomicBool::new(false),
            cancel: Mutex::new(None),
            last_error: Mutex::new(String::new()),
            costs: Mutex::new(BTreeMap::new()),
        })
        .setup(|app| {
            let h = app.handle().clone();
            // once now (after the window is up) and then every few hours for launchers left open over midnight
            std::thread::spawn(move || loop {
                std::thread::sleep(Duration::from_secs(30));
                usage_ping(&h);
                std::thread::sleep(Duration::from_secs(4 * 3600));
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_status, refresh, set_channel, switch_to, send_logs, set_comp_option, set_setting, set_component, remove_conflicts, pick_game_dir, open_url, open_game_dir, start_update, cancel_update, play
        ])
        .run(tauri::generate_context!())
        .expect("error while running Elite Launcher");
}
