//! `launcher.json` — everything the Elite team can change without shipping a new launcher.
//!
//! Every field is optional and unknown fields are ignored, so old launchers keep working
//! with newer configs for years. Texts can be a plain string or a map by language:
//! `"text": "Hello"` or `"text": {"en": "Hello", "ru": "Привет"}`.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Text {
    #[default]
    None,
    Plain(String),
    ByLang(BTreeMap<String, String>),
}

impl Text {
    pub fn get(&self, lang: &str) -> String {
        match self {
            Text::None => String::new(),
            Text::Plain(s) => s.clone(),
            Text::ByLang(m) => m.get(lang).or_else(|| m.get("en")).or_else(|| m.values().next()).cloned().unwrap_or_default(),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct LauncherConfig {
    pub schema: u32,
    /// Shown in the window header, e.g. "Elite Mod".
    pub title: Text,
    pub subtitle: Text,
    pub channels: Vec<Channel>,
    /// Banner when a newer version of the player's channel exists. `{version}` and `{name}` are replaced.
    pub update_text: Text,
    pub news: Vec<NewsItem>,
    /// Button under the news list, e.g. the ModDB news page.
    pub news_more: Option<Link>,
    pub announcements: Vec<Announcement>,
    pub links: Vec<Link>,
    /// Extra mirrors (base URLs ending with '/'); tried after the built-in ones that answered.
    pub mirrors: Vec<String>,
    pub game: GameInfo,
    pub launch: Launch,
    pub launcher_update: Option<LauncherUpdate>,
    /// Optional add-ons the player can tick (e.g. LobbyStats). Each is a separately published version.
    pub components: Vec<Component>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Component {
    pub id: String,
    pub name: Text,
    pub description: Text,
    /// Version id on the mirrors (`<version>.manifest`), published like any build.
    pub version: String,
    /// Installed unless the player unticks it.
    pub default: bool,
    /// DLL to add to `InjectedFiles=` in Elite.launchconfig while the component is on.
    pub inject: String,
    /// Switches the player can tick under the component; each one is written to an ini file in the game folder.
    pub options: Vec<ComponentOption>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ComponentOption {
    pub id: String,
    pub name: Text,
    /// ini file relative to the game folder, e.g. "Elite_DLLs/LobbyStats.ini"
    pub ini: String,
    pub section: String,
    pub key: String,
    pub default: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Channel {
    pub id: String,
    pub name: Text,
    /// Version id on the mirrors: `<version>.manifest`.
    pub version: String,
    pub description: Text,
    /// Channel selected for new installs.
    pub default: bool,
    /// Small label next to the name, e.g. "BETA".
    pub badge: Text,
    /// Old version: shown in the collapsed "Archive" list instead of the main list.
    pub archive: bool,
    /// Program to start on Windows for this version, relative to the game folder (e.g. "Elite_Test.exe" for the
    /// public test that installs as a separate mod next to the main one). Empty = the default launch command.
    pub exe: String,
    /// Command-line arguments for this version on Windows. Old Elite (2.3, 2.9.3) has no launcher of its own and runs
    /// as a plain mod: `exe = "DOW2.exe"`, `args = ["-modname", "Elite"]`. Empty = the default arguments.
    pub args: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct NewsItem {
    pub date: String,
    pub title: Text,
    pub text: Text,
    pub url: String,
    pub image: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Announcement {
    pub id: String,
    pub text: Text,
    pub url: String,
    /// "info" | "accent" | "warning"
    pub style: String,
    /// ISO date (YYYY-MM-DD) after which it is hidden.
    pub until: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Link {
    pub title: Text,
    pub url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct GameInfo {
    pub steam_appid: u32,
    pub name: String,
    /// Folder name inside steamapps/common.
    pub folder: String,
    /// File that must exist in the game folder.
    pub marker: String,
    pub store_url: String,
    pub web_store_url: String,
    /// Shown when the game is missing.
    pub requirements: Text,
}

impl Default for GameInfo {
    fn default() -> Self {
        Self {
            steam_appid: 56400,
            name: "Warhammer 40,000: Dawn of War II - Retribution".into(),
            folder: "Dawn of War II - Retribution".into(),
            marker: "DOW2.exe".into(),
            store_url: "steam://store/56400".into(),
            web_store_url: "https://store.steampowered.com/app/56400/".into(),
            requirements: Text::None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Launch {
    pub windows: LaunchCmd,
    pub linux: LaunchCmd,
    pub macos: LaunchCmd,
}

impl Default for Launch {
    fn default() -> Self {
        Self {
            windows: LaunchCmd { program: "{game}/Elite.exe".into(), args: vec![], hint: Text::None },
            linux: LaunchCmd { program: "steam".into(), args: vec!["-applaunch".into(), "56400".into()], hint: Text::None },
            macos: LaunchCmd { program: "open".into(), args: vec!["steam://rungameid/56400".into()], hint: Text::None },
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct LaunchCmd {
    /// `{game}` is replaced with the game folder.
    pub program: String,
    pub args: Vec<String>,
    /// Shown under the Play button on this OS (e.g. Steam launch options for Proton).
    pub hint: Text,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct LauncherUpdate {
    pub version: String,
    pub text: Text,
    /// Download page per OS: "windows", "linux", "macos" (or "all").
    pub urls: BTreeMap<String, String>,
}

/// Compare dotted versions ("1.2.10" > "1.2.9").
pub fn version_newer(a: &str, b: &str) -> bool {
    let p = |s: &str| s.split(|c: char| !c.is_ascii_digit()).filter(|x| !x.is_empty()).map(|x| x.parse::<u64>().unwrap_or(0)).collect::<Vec<_>>();
    p(a) > p(b)
}
