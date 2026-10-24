//! Elite Mod launcher core.
//!
//! Everything that can change lives on the server as data (`launcher.json`, signed);
//! the launcher binary only carries the engine: chunked updates, signature checks,
//! mirror fail-over and game detection. See README.md at the repository root.
pub mod cdc;
pub mod config;
pub mod elite;
pub mod game;
pub mod manifest;
pub mod net;
pub mod sign;
pub mod update;

/// Version of this launcher build (compared with `min_launcher` / `launcher_update` in launcher.json).
pub const LAUNCHER_VERSION: &str = env!("CARGO_PKG_VERSION");
