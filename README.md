# Elite Launcher

Install, update and launch **Elite Mod** for *Dawn of War II: Retribution* — with a news feed,
announcements, stable/beta versions and small patches.

For players it is two clicks: **Install Elite** once, then **Play**. A new version is offered with a
banner (*"Elite 3.1 is out! Update now — the lobbies have already moved to it"*), but the game can
always be started, update or not.

## What it does

- **Finds the game** in every Steam library (Windows, Linux, macOS). If the game or Steam is
  missing, it says so and opens the Steam store page (`steam://store/56400`) or lets the player pick a folder.
- **Installs Elite** from your mirrors. Files are checked with SHA-256.
- **Small updates.** Every file is split into content-defined chunks (~16 KB) when you publish a
  build. The launcher keeps the chunks that didn't change in the player's files and downloads only
  the new ones. No unpacking or repacking on the player's PC — the result is the exact same file.
  A balance change inside `Elite_attrib.sga` is around 1 MB instead of the full installer.
- **Stable / beta / any number of channels.** Switching between versions the player already had is
  instant (replaced files are kept in `<game>/.elite_cache`, only changed files, size limit in settings).
- **Repair** button: re-checks every file and downloads only the damaged parts. This also brings
  back files that Steam's "Verify integrity" replaces (e.g. the sound-bug fix DLLs).
- **News, announcements, links** shown on the home screen, all coming from the server.
- **Add-ons** (`components` in launcher.json) the player ticks on/off, e.g. *LobbyStats* (Relic rating
  in the lobby + host auto-kick filter). An add-on is published like any version; the launcher installs
  it, adds its DLL to `InjectedFiles=` in `Elite.launchconfig` and re-adds it after every Elite update.
  The host filter itself is set in the game: the host types `!filter 1500-2400 wr55 h500 live` in lobby
  chat, the bot answers for everyone and stores the rules in `Elite_DLLs/LobbyStats.ini` (the launcher
  only creates a default file, never overwrites it).
- **Several mirrors** with automatic fail-over, including resuming a broken range on another mirror.
- **Signed data.** `launcher.json` and every version manifest are signed with Ed25519; the public
  key is compiled into the launcher. A hacked or untrusted mirror cannot push modified files or texts.
- **Works offline** with the last verified config; Play never needs the server.

## Repository

| Path | What |
|---|---|
| `crates/elite-core` | the engine: chunking, manifests, signatures, mirrors, updates, game detection |
| `crates/elite-publish` | the tool for the Elite team: keys, publishing a build, signing `launcher.json` |
| `crates/elite-cli` | the same engine in a console program (servers, scripts, tests) |
| `app` | the launcher window (Tauri 2: Rust + the system web view, ~5 MB) |
| `app/ui` | the interface: plain HTML/CSS/JS, no build step |
| `keys/public.key` | public key compiled into the launcher |
| `.github/workflows/build.yml` | builds Windows / Linux / macOS packages on GitHub |

## Publishing a version (Elite team)

```bash
# once: create your own keys and put public.key into keys/ before building the launcher
elite-publish keygen ~/elite-keys            # private.key stays with you, never commit it

# every release: publish the installed mod files of the new build
elite-publish version <build_folder> <store> 3.2 ~/elite-keys/private.key files.txt
#   <build_folder> — a game folder (or just the Elite files) of the new version
#   files.txt      — which files belong to Elite, one path per line, relative to the game folder
#                    (Elite/…, Elite_bin/…, Elite_DLLs/…, Elite.*, Profiler.dll, Shark.dll, fmodex.dll, …)

# point a channel to it in <store>/launcher.json, then sign the config
elite-publish sign <store>/launcher.json ~/elite-keys/private.key
elite-publish verify <store>                 # check all signatures before uploading
```

Upload `<store>` to every mirror as-is. Any static hosting with HTTP Range support works
(Cloudflare R2, S3, GitHub Pages/Releases, nginx…). Identical files across versions are stored once
(`blobs/<sha256>`), so a new version only adds what changed. Old versions can stay available for
rollback or be deleted together with blobs no manifest uses.

## launcher.json

Everything here can change without releasing a new launcher. All fields are optional; unknown
fields are ignored, so old launchers keep working with newer configs. Texts are either a string or
`{"en": "...", "ru": "..."}`.

```jsonc
{
  "schema": 1,
  "title": "Elite Mod",
  "subtitle": "Dawn of War II · Retribution",
  "channels": [
    {"id": "stable", "name": "Elite 3.1", "version": "3.1", "default": true,
     "description": "Main version — the lobbies play this"},
    {"id": "beta", "name": "Elite 3.2 Public Test", "version": "3.2-ptb", "badge": "BETA",
     "description": "Public test build: may be unstable"}
  ],
  "update_text": "{name} is out! Update now — the lobbies have already moved to it.",
  "news": [{"date": "2026-10-07", "title": "...", "text": "...", "url": "https://www.moddb.com/..."}],
  "news_more": {"title": "All news on ModDB", "url": "https://www.moddb.com/mods/dawn-of-war-2-retribution-elite-mod/news"},
  "announcements": [{"id": "motya2026", "style": "accent", "until": "2026-12-31",
                     "text": "Vote for Elite — Mod of the Year!", "url": "https://www.moddb.com/..."}],
  "links": [{"title": "ModDB", "url": "..."}, {"title": "Discord", "url": "..."}],
  "mirrors": ["https://mirror2.example.org/elite/"],
  "game": {"requirements": "Elite needs the Steam version of Dawn of War II: Retribution."},
  "launch": {
    "windows": {"program": "{game}/Elite.exe"},
    "linux": {"program": "steam", "args": ["-applaunch", "56400"], "hint": "Set Steam launch options to run Elite.exe through Proton."}
  },
  "components": [{"id": "lobbystats", "version": "lobbystats-0.4", "default": true, "inject": "LobbyStats.dll",
                  "name": "Lobby rating & host filter", "description": "Relic rating in the lobby; optional auto-kick rules when you host"}],
  "launcher_update": {"version": "1.1.0", "urls": {"windows": "https://...", "linux": "https://...", "macos": "https://..."}}
}
```

- `update_text` — banner when the player's channel points to a newer version. `{name}`/`{version}` are replaced.
- `announcements` — dismissable bars on top; `until` hides them after a date.
- `launcher_update` — offers (never forces) a newer launcher.
- Built-in mirrors are in `app/src/main.rs` (`BUILTIN_MIRRORS`). Put hosts from different
  providers/domains there, so the launcher can start even if one is blocked; add more later through `mirrors`.

## Building

```bash
cargo install tauri-cli --version "^2" --locked
cd app && cargo tauri build                 # installer for the current OS
cargo build --release -p elite-publish -p elite-cli
```

Linux needs `libwebkit2gtk-4.1-dev libsoup-3.0-dev librsvg2-dev`. Windows packages can also be
cross-built on Linux: `cargo tauri build --target x86_64-pc-windows-gnu --bundles nsis` (needs `mingw-w64`
and `nsis`). macOS needs a Mac or the GitHub workflow. Without code signing Windows SmartScreen shows a
warning on first start; signing is configured in `app/tauri.conf.json` (`bundle > windows`).

## Versions and releases

The version lives only in `Cargo.toml` (`[workspace.package] version`); Tauri takes it from there. To ship one:
raise the version, add its entry at the top of [CHANGELOG.md](CHANGELOG.md) and merge to `main`.

`.github/workflows/build.yml` runs on every push and pull request:

| Job | What |
|---|---|
| `version` | reads the version, fails when CHANGELOG.md has no entry for it |
| `core` | builds and tests `elite-core`, `elite-publish`, `elite-cli` |
| `launcher` | installers for Windows, Linux, macOS (artifacts, kept 7 days) |
| `publisher` | `elite-publish` and `elite-cli` for each OS |
| `publish` | dry run: publishes the installers into a temporary store with a one-off key, signs a `launcher.json`, verifies all signatures, then deletes it all; the upload to the mirrors is skipped for now |
| `release` | only on `main`: tags `vX.Y.Z` if it is new; for `X.Y.0` also a GitHub Release with the installers and the CHANGELOG entries since the previous `X.Y.0` |

Tags are set by CI, not by hand.

## Formats (stable — changing them needs a new launcher)

- Chunking: gear hash, min/avg/max 4/16/64 KB, see `crates/elite-core/src/cdc.rs`.
- Manifest `ELUPD001`: version, files (path, size, SHA-256, chunk list); `<version>.manifest` + `.sig`.
- Blobs: `blobs/<sha256 of the file>`, fetched with HTTP Range.
- Signatures: Ed25519, base64 in `<file>.sig`.

License: MIT.

## Code signing

Windows builds are signed for free by SignPath Foundation, see [CODE_SIGNING.md](CODE_SIGNING.md) (includes the privacy policy).
